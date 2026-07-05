//! Ports `others/acpx/src/runtime/engine/connected-session.ts`: owns one
//! live [`AcpClient`] + backend session id pairing, plus the mutable
//! session-scoped state ([`crate::session::record::SessionRecord`], the
//! in-memory [`SessionConversation`], and the raw `session/update`
//! notification feed) a running turn or a `get_status`/`set_mode` call
//! reads or updates.
//!
//! Per Requirement 6 and this file's module docs in the phase plan,
//! `has_active_prompt`/`request_cancel_active_prompt` here describe *this
//! session's* in-flight turn only (an `AtomicBool` this struct owns), not a
//! client-global single-flight lock — Phase 6's per-session queue builds on
//! top of this, it does not replace it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use agent_client_protocol::schema::v1::{
    McpServer, SessionConfigId, SessionConfigValueId, SessionId, SessionModeId,
    SessionNotification, SetSessionConfigOptionResponse,
};
use parking_lot::Mutex;

use crate::client::AcpClient;
use crate::error::Result;
use crate::queue::SessionPromptQueue;
use crate::session::conversation_model::SessionConversation;
use crate::session::model_state::advertised_model_state;
use crate::session::record::SessionRecord;

/// One connected ACP session: the live client, the backend session id
/// (mutable — reconnect may create a fresh one), and the record/conversation
/// state a turn mutates as it runs.
pub struct ConnectedSession {
    pub client: AcpClient,
    session_id: Mutex<SessionId>,
    pub record: Mutex<SessionRecord>,
    pub conversation: Mutex<SessionConversation>,
    /// Fed by the client's `session/update` notification handler (see
    /// `crate::client::handlers::ClientRequestHandlers`); cloned into
    /// whichever turn is currently draining it (see module docs — this
    /// phase assumes at most one active turn per session in practice).
    pub notifications: smol::channel::Receiver<SessionNotification>,
    pub mcp_servers: Vec<McpServer>,
    active_prompt: AtomicBool,
    /// Phase 6 (ADR-4): this session's bounded FIFO prompt queue. Public so
    /// `runtime::engine::manager::AcpRuntime::start_turn` can enqueue
    /// through it directly instead of calling `prompt_turn::start_turn`
    /// itself (that call is now `crate::queue::dispatcher`'s job).
    pub prompt_queue: SessionPromptQueue,
    /// Serializes `session/update` notification application in arrival
    /// order (Requirement 4; ports acpx's `sessionUpdateChain` guarantee at
    /// session granularity). Use [`Self::with_ordered_update`] rather than
    /// reaching in directly.
    ///
    /// This is a SEPARATE lock from `prompt_queue`'s internal admission
    /// lock (Risk Assessment: "deadlock risk between the per-session
    /// update-ordering lock and the prompt-queue lock") — code in this
    /// crate must never hold both at once. In practice `prompt_queue` never
    /// touches `update_order` and vice versa (neither module even imports
    /// the other's lock type), so the invariant holds by construction, not
    /// just by convention; keep it that way in any future change.
    update_order: Mutex<()>,
}

impl ConnectedSession {
    pub fn new(
        client: AcpClient,
        session_id: SessionId,
        record: SessionRecord,
        conversation: SessionConversation,
        notifications: smol::channel::Receiver<SessionNotification>,
        mcp_servers: Vec<McpServer>,
        prompt_queue_capacity: Option<usize>,
    ) -> Arc<Self> {
        Arc::new(Self {
            client,
            session_id: Mutex::new(session_id),
            record: Mutex::new(record),
            conversation: Mutex::new(conversation),
            notifications,
            mcp_servers,
            active_prompt: AtomicBool::new(false),
            prompt_queue: match prompt_queue_capacity {
                Some(capacity) => SessionPromptQueue::with_capacity(capacity),
                None => SessionPromptQueue::new(),
            },
            update_order: Mutex::new(()),
        })
    }

    /// Runs `f` (expected to be a quick, non-blocking append/forward — see
    /// Step 5's design note) under the `session/update` ordering lock. The
    /// caller's per-session single `notifications` consumer loop already
    /// only ever runs one `f` at a time in arrival order; this lock makes
    /// that guarantee explicit and future-proof rather than an incidental
    /// side effect of today's single-consumer structure.
    pub(crate) fn with_ordered_update<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = self.update_order.lock();
        f()
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id.lock().clone()
    }

    pub fn set_session_id(&self, session_id: SessionId) {
        *self.session_id.lock() = session_id;
    }

    /// Ports `hasActivePrompt` (session-scoped, see module docs).
    pub fn has_active_prompt(&self) -> bool {
        self.active_prompt.load(Ordering::SeqCst)
    }

    pub(crate) fn set_active_prompt(&self, active: bool) {
        self.active_prompt.store(active, Ordering::SeqCst);
    }

    /// Ports `requestCancelActivePrompt`: sends `session/cancel` only if a
    /// prompt is actually in flight, returning whether it did.
    pub fn request_cancel_active_prompt(&self) -> Result<bool> {
        if !self.has_active_prompt() {
            return Ok(false);
        }
        self.client.cancel_session(self.session_id())?;
        Ok(true)
    }

    /// Ports `setSessionMode`.
    pub async fn set_session_mode(&self, mode_id: &str) -> Result<()> {
        self.client
            .set_session_mode(self.session_id(), SessionModeId::new(mode_id))
            .await?;
        Ok(())
    }

    /// Ports `setSessionModel`: resolves the advertised model config id from
    /// the record's `acpx` state, then delegates to `session/set_config_option`.
    pub async fn set_session_model(
        &self,
        model_id: &str,
    ) -> Result<SetSessionConfigOptionResponse> {
        let config_id = {
            let record = self.record.lock();
            advertised_model_state(record.acpx.as_ref()).and_then(|models| models.config_id)
        };
        let config_id = config_id.unwrap_or_else(|| "model".to_string());
        self.client
            .set_session_config_option(
                self.session_id(),
                SessionConfigId::new(config_id),
                SessionConfigValueId::new(model_id),
            )
            .await
    }

    /// Ports `setSessionConfigOption`.
    pub async fn set_session_config_option(
        &self,
        config_id: &str,
        value: &str,
    ) -> Result<SetSessionConfigOptionResponse> {
        self.client
            .set_session_config_option(
                self.session_id(),
                SessionConfigId::new(config_id),
                SessionConfigValueId::new(value),
            )
            .await
    }
}
