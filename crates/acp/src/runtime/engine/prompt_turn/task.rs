//! The detached task [`super::start_turn`] spawns: races `session/prompt`
//! against an optional cancel signal while a sibling task drains live
//! `session/update` notifications, then persists the result. Split out of
//! `prompt_turn/mod.rs` (setup vs. execution) per the workspace's per-file
//! line convention.

use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::schema::v1::{ContentBlock, PromptRequest};
use futures::channel::oneshot;
use futures::future::{Either, select};
use futures::pin_mut;

use crate::control::with_timeout;
use crate::error::AcpError;
use crate::runtime::engine::connected_session::ConnectedSession;
use crate::runtime::engine::lifecycle::apply_conversation;
use crate::runtime::public::contract::{AcpRuntimeTurnResult, AcpSessionStore};
use crate::runtime::public::events::{AcpRuntimeEvent, parse_session_update};

use super::turn_result::{
    turn_result_from_rpc_error, turn_result_from_stop_reason, turn_result_from_timeout,
};
use super::update_mapping::session_update_input;
use crate::session::conversation_model::session_update::record_session_update;

#[allow(clippy::too_many_arguments)]
pub(super) async fn run_turn_task(
    connected: Arc<ConnectedSession>,
    session_store: Arc<dyn AcpSessionStore>,
    session_id: agent_client_protocol::schema::v1::SessionId,
    content_blocks: Vec<ContentBlock>,
    timeout: Option<Duration>,
    event_tx: smol::channel::Sender<AcpRuntimeEvent>,
    cancel_rx: oneshot::Receiver<Option<String>>,
    result_tx: oneshot::Sender<AcpRuntimeTurnResult>,
    on_slot_freed: Option<oneshot::Sender<()>>,
) {
    let drain_task = smol::spawn(drain_notifications(
        connected.clone(),
        session_id.clone(),
        event_tx.clone(),
    ));

    let request = PromptRequest::new(session_id.clone(), content_blocks);
    let prompt_future = connected
        .client
        .connection()
        .send_request(request)
        .block_task();
    pin_mut!(prompt_future);

    // Cloning the `Arc` (not `AcpClient` itself, which isn't `Clone`) so the
    // `async move` block below can own a handle to send `session/cancel`
    // without taking `connected` away from the rest of this function.
    let race_connected = connected.clone();
    let race_session_id = session_id.clone();
    let raced = async move {
        match select(prompt_future.as_mut(), cancel_rx).await {
            Either::Left((response, _cancel_rx)) => response,
            Either::Right((_cancel_result, _prompt_future)) => {
                let _ = race_connected.client.cancel_session(race_session_id);
                prompt_future.await
            }
        }
    };

    let outcome = match timeout {
        Some(timeout) => match with_timeout(raced, Some(timeout)).await {
            Ok(result) => result.map_err(turn_result_from_rpc_error),
            Err(AcpError::Timeout(d)) => Err(turn_result_from_timeout(d)),
            Err(_) => Err(turn_result_from_timeout(timeout)),
        },
        None => raced.await.map_err(turn_result_from_rpc_error),
    };

    // Stop draining live updates before persisting: `drain_task` holds the
    // same `record`/`conversation` locks this function is about to take.
    drain_task.cancel().await;

    connected.set_active_prompt(false);
    if let Some(tx) = on_slot_freed {
        // Best-effort: the queue dispatcher always awaits this (see
        // `crate::queue::dispatcher::dispatch`), but a send failure here
        // (nobody listening) must never fail the turn itself.
        let _ = tx.send(());
    }

    let result = match outcome {
        Ok(response) => turn_result_from_stop_reason(response.stop_reason),
        Err(result) => result,
    };

    {
        let mut record = connected.record.lock();
        let conversation = connected.conversation.lock();
        apply_conversation(&mut record, &conversation);
        let now = crate::session::conversation_model::conversation::iso_now();
        record.last_prompt_at = Some(now.clone());
        record.last_used_at = now;
    }
    let snapshot = connected.record.lock().clone();
    let _ = session_store.save(snapshot).await;

    let terminal_event = match &result {
        AcpRuntimeTurnResult::Completed { stop_reason }
        | AcpRuntimeTurnResult::Cancelled { stop_reason } => AcpRuntimeEvent::Done {
            stop_reason: stop_reason.clone(),
        },
        AcpRuntimeTurnResult::Failed { error } => AcpRuntimeEvent::Error {
            message: error.message.clone(),
            code: error.code.clone(),
            detail_code: error.detail_code.clone(),
            retryable: error.retryable,
        },
    };
    let _ = event_tx.try_send(terminal_event);
    let _ = result_tx.send(result);
}

async fn drain_notifications(
    connected: Arc<ConnectedSession>,
    session_id: agent_client_protocol::schema::v1::SessionId,
    event_tx: smol::channel::Sender<AcpRuntimeEvent>,
) {
    while let Ok(notification) = connected.notifications.recv().await {
        if notification.session_id != session_id {
            continue;
        }
        // Requirement 4: apply this notification's live-event forward +
        // persisted-history fold under the session's update-ordering lock,
        // making "processed in arrival order" an explicit invariant rather
        // than an incidental consequence of this being a single-consumer
        // loop. See `ConnectedSession::update_order`'s doc comment for why
        // this must stay a separate lock from the prompt queue's.
        connected.with_ordered_update(|| {
            if let Some(event) = parse_session_update(&notification.update) {
                let _ = event_tx.try_send(event);
            }
            if let Some(update_input) = session_update_input(&notification.update) {
                let mut conversation = connected.conversation.lock();
                let mut record = connected.record.lock();
                let acpx = record.acpx.get_or_insert_with(Default::default);
                record_session_update(&mut conversation, acpx, update_input, None);
            }
        });
    }
}
