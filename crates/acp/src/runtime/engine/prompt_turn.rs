//! Ports `others/acpx/src/runtime/engine/prompt-turn.ts` plus the
//! turn-execution half of `manager.ts`'s `startTurn`: runs exactly one
//! turn given "permission already granted". **Deliberately queue-agnostic**
//! per Requirement 6 — this module does not check "is another turn already
//! running on this session"; that single-flight gate is Phase 6's
//! per-session queue (ADR-4), layered *around* [`start_turn`], not inside
//! it.
//!
//! ## Turn lifecycle
//!
//! 1. Build the ACP `ContentBlock`s from the turn input, record the user
//!    message into the session's in-memory conversation model.
//! 2. Spawn a background task that: (a) drains the connected session's raw
//!    `session/update` notification feed for the lifetime of the turn,
//!    forwarding parsed [`AcpRuntimeEvent`]s to the caller's stream and
//!    folding each update into the conversation model; (b) sends
//!    `session/prompt` and races it against an optional cancel signal —
//!    cancelling sends `session/cancel` and then keeps awaiting the
//!    agent's (now-cancelled) response, per ACP's cooperative-cancellation
//!    contract, rather than abandoning the request.
//! 3. On completion (success, cancellation, or failure), merge the
//!    conversation model back into the record and persist it.
//!
//! Token-usage breakdowns from `UsageUpdate._meta.usage` are surfaced live
//! (see `runtime::public::events`) but not additionally persisted into
//! `request_token_usage` here — this build has no `PromptResponse.usage`
//! field (the `unstable_end_turn_token_usage` schema feature isn't
//! enabled), so there is no terminal-response usage payload to record via
//! [`crate::session::conversation_model::record_prompt_response_usage`];
//! only the live `cumulative_cost` from `UsageUpdate.cost` is folded in.

use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::schema::v1::{
    AvailableCommandsUpdate, ContentBlock, CurrentModeUpdate, PromptRequest, SessionUpdate,
    StopReason, ToolCall, ToolCallStatus, ToolCallUpdate, ToolKind, UsageUpdate,
};
use futures::channel::oneshot;
use futures::future::{Either, select};
use futures::pin_mut;
use futures::stream::BoxStream;

use crate::control::with_timeout;
use crate::error::AcpError;
use crate::error_normalization::is_retryable_prompt_error;
use crate::runtime::engine::connected_session::ConnectedSession;
use crate::runtime::engine::lifecycle::apply_conversation;
use crate::runtime::public::contract::{
    AcpRuntimeTurn, AcpRuntimeTurnInput, AcpRuntimeTurnResult, AcpRuntimeTurnResultError,
    AcpSessionStore, attachment_content_blocks,
};
use crate::runtime::public::errors::{AcpRuntimeError, AcpRuntimeErrorCode};
use crate::runtime::public::events::{AcpRuntimeEvent, parse_session_update};
use crate::session::acpx_state::SessionAvailableCommand;
use crate::session::conversation_model::agent_content::InboundContent;
use crate::session::conversation_model::conversation::SessionUsageCost;
use crate::session::conversation_model::record::record_prompt_submission;
use crate::session::conversation_model::session_update::{
    SessionUpdateInput, record_session_update,
};
use crate::session::conversation_model::tool_call::ToolCallUpdateInput;

const EVENT_CHANNEL_CAPACITY: usize = 512;

fn text_from_content_block(block: &ContentBlock) -> Option<&str> {
    match block {
        ContentBlock::Text(text) => Some(text.text.as_str()),
        _ => None,
    }
}

fn kind_str(kind: ToolKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "other".to_string())
}

fn status_str(status: ToolCallStatus) -> String {
    serde_json::to_value(status)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "pending".to_string())
}

fn tool_call_update_input_from_call(call: &ToolCall) -> ToolCallUpdateInput {
    ToolCallUpdateInput {
        tool_call_id: call.tool_call_id.0.to_string(),
        title: Some(call.title.clone()),
        kind: Some(kind_str(call.kind)),
        status: Some(status_str(call.status)),
        raw_input: call.raw_input.clone(),
        raw_output_present: call.raw_output.is_some(),
        raw_output: call.raw_output.clone(),
    }
}

fn tool_call_update_input_from_update(update: &ToolCallUpdate) -> ToolCallUpdateInput {
    ToolCallUpdateInput {
        tool_call_id: update.tool_call_id.0.to_string(),
        title: update.fields.title.clone(),
        kind: update.fields.kind.map(kind_str),
        status: update.fields.status.map(status_str),
        raw_input: update.fields.raw_input.clone(),
        raw_output_present: update.fields.raw_output.is_some(),
        raw_output: update.fields.raw_output.clone(),
    }
}

fn available_command_from_update(update: &AvailableCommandsUpdate) -> Vec<SessionAvailableCommand> {
    update
        .available_commands
        .iter()
        .map(|command| SessionAvailableCommand {
            name: command.name.clone(),
            description: (!command.description.trim().is_empty())
                .then(|| command.description.clone()),
            has_input: Some(command.input.is_some()),
        })
        .collect()
}

fn current_mode_update_input(update: &CurrentModeUpdate) -> SessionUpdateInput {
    SessionUpdateInput::CurrentModeUpdate(update.current_mode_id.0.to_string())
}

fn usage_update_input(update: &UsageUpdate) -> SessionUpdateInput {
    SessionUpdateInput::UsageUpdate {
        usage: None,
        cost: update.cost.as_ref().map(|cost| SessionUsageCost {
            amount: Some(cost.amount),
            currency: Some(cost.currency.clone()),
        }),
    }
}

/// Maps a live typed `SessionUpdate` onto Phase 5's protocol-agnostic
/// [`SessionUpdateInput`] so [`record_session_update`] can fold it into the
/// persisted conversation model. Companion to
/// `runtime::public::events::parse_session_update` (same source, different
/// destination — one produces the live UI event, this produces the
/// persisted-history delta).
fn session_update_input(update: &SessionUpdate) -> Option<SessionUpdateInput> {
    match update {
        SessionUpdate::UserMessageChunk(chunk) => {
            text_from_content_block(&chunk.content).map(|text| {
                SessionUpdateInput::UserMessageChunk(InboundContent::Text(text.to_string()))
            })
        }
        SessionUpdate::AgentMessageChunk(chunk) => text_from_content_block(&chunk.content)
            .map(|text| SessionUpdateInput::AgentMessageChunk(text.to_string())),
        SessionUpdate::AgentThoughtChunk(chunk) => text_from_content_block(&chunk.content)
            .map(|text| SessionUpdateInput::AgentThoughtChunk(text.to_string())),
        SessionUpdate::ToolCall(call) => Some(SessionUpdateInput::ToolCall(
            tool_call_update_input_from_call(call),
        )),
        SessionUpdate::ToolCallUpdate(update) => Some(SessionUpdateInput::ToolCall(
            tool_call_update_input_from_update(update),
        )),
        SessionUpdate::AvailableCommandsUpdate(update) => Some(
            SessionUpdateInput::AvailableCommandsUpdate(available_command_from_update(update)),
        ),
        SessionUpdate::CurrentModeUpdate(update) => Some(current_mode_update_input(update)),
        SessionUpdate::ConfigOptionUpdate(update) => Some(SessionUpdateInput::ConfigOptionUpdate(
            update.config_options.clone(),
        )),
        SessionUpdate::SessionInfoUpdate(update) => Some(SessionUpdateInput::SessionInfoUpdate {
            title: match &update.title {
                agent_client_protocol::schema::MaybeUndefined::Undefined => None,
                agent_client_protocol::schema::MaybeUndefined::Null => Some(None),
                agent_client_protocol::schema::MaybeUndefined::Value(v) => Some(Some(v.clone())),
            },
            updated_at: update.updated_at.clone().take(),
        }),
        SessionUpdate::UsageUpdate(update) => Some(usage_update_input(update)),
        _ => None,
    }
}

fn turn_result_from_stop_reason(stop_reason: StopReason) -> AcpRuntimeTurnResult {
    let text = match stop_reason {
        StopReason::EndTurn => "end_turn",
        StopReason::MaxTokens => "max_tokens",
        StopReason::MaxTurnRequests => "max_turn_requests",
        StopReason::Refusal => "refusal",
        StopReason::Cancelled => "cancelled",
        // `StopReason` is `#[non_exhaustive]`; treat any future variant as
        // a normal completion rather than failing to compile against a
        // schema point release.
        _ => "unknown",
    };
    if matches!(stop_reason, StopReason::Cancelled) {
        AcpRuntimeTurnResult::Cancelled {
            stop_reason: Some(text.to_string()),
        }
    } else {
        AcpRuntimeTurnResult::Completed {
            stop_reason: Some(text.to_string()),
        }
    }
}

fn turn_result_from_timeout(timeout: Duration) -> AcpRuntimeTurnResult {
    AcpRuntimeTurnResult::Failed {
        error: AcpRuntimeTurnResultError {
            message: format!("prompt timed out after {timeout:?}"),
            code: Some("TIMEOUT".to_string()),
            detail_code: None,
            retryable: Some(true),
        },
    }
}

fn turn_result_from_rpc_error(error: agent_client_protocol::Error) -> AcpRuntimeTurnResult {
    let retryable = is_retryable_prompt_error(&error);
    AcpRuntimeTurnResult::Failed {
        error: AcpRuntimeTurnResultError {
            message: error.message.clone(),
            code: Some(i32::from(error.code).to_string()),
            detail_code: None,
            retryable: Some(retryable),
        },
    }
}

/// Ports `startTurn`'s turn-execution body (queue-agnostic, see module
/// docs). Returns immediately with a live [`AcpRuntimeTurn`]; the actual
/// RPC work runs on a detached `smol` task.
///
/// `on_slot_freed` is Phase 6's hook (`crate::queue::dispatcher`): fired
/// exactly once, right when this turn's slot becomes reusable (success,
/// RPC error, timeout, or cancellation all count — same "done" moment as
/// `connected.set_active_prompt(false)` below), so the per-session queue
/// knows when to dispatch the next pending request. `None` when called
/// outside the queue (there is currently no such caller, but the parameter
/// stays optional so this function doesn't *require* a queue to exist).
pub fn start_turn(
    connected: Arc<ConnectedSession>,
    session_store: Arc<dyn AcpSessionStore>,
    input: AcpRuntimeTurnInput,
    default_timeout_ms: Option<u64>,
    on_slot_freed: Option<oneshot::Sender<()>>,
) -> Result<AcpRuntimeTurn, AcpRuntimeError> {
    let content_blocks = attachment_content_blocks(&input.text, &input.attachments)
        .map_err(|err| AcpRuntimeError::new(AcpRuntimeErrorCode::TurnFailed, err.message))?;
    let request_id = input.request_id.clone();
    let timeout = input
        .timeout_ms
        .or(default_timeout_ms)
        .map(Duration::from_millis);

    {
        let mut conversation = connected.conversation.lock();
        record_prompt_submission(
            &mut conversation,
            &[InboundContent::Text(input.text.clone())],
            None,
        );
    }

    let (event_tx, event_rx) = smol::channel::bounded::<AcpRuntimeEvent>(EVENT_CHANNEL_CAPACITY);
    let (cancel_tx, cancel_rx) = oneshot::channel::<Option<String>>();
    let (result_tx, result_rx) = oneshot::channel::<AcpRuntimeTurnResult>();

    let session_id = connected.session_id();
    connected.set_active_prompt(true);

    smol::spawn(run_turn_task(
        connected,
        session_store,
        session_id,
        content_blocks,
        timeout,
        event_tx,
        cancel_rx,
        result_tx,
        on_slot_freed,
    ))
    .detach();

    let events: BoxStream<'static, AcpRuntimeEvent> = Box::pin(event_rx);
    let result = Box::pin(async move {
        result_rx.await.unwrap_or(AcpRuntimeTurnResult::Failed {
            error: AcpRuntimeTurnResultError {
                message: "turn task ended without reporting a result".to_string(),
                code: None,
                detail_code: None,
                retryable: Some(false),
            },
        })
    });
    Ok(AcpRuntimeTurn::new(request_id, events, result, cancel_tx))
}

#[allow(clippy::too_many_arguments)]
async fn run_turn_task(
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
