//! The public embeddable contract. Ports
//! `others/acpx/src/runtime/public/contract.ts` — the actual API surface the
//! GPUI app calls. Per the phase brief, shape fidelity to this file matters
//! more than internal implementation fidelity elsewhere in `runtime/`.
//!
//! ## ADR-7 recap (ensureSession/startTurn shape)
//!
//! acpx's `AcpRuntime` is a TS *interface* implemented by exactly one
//! concrete runtime; nothing in this crate's scope needs genuine
//! substitutability (no test double stands in for the runtime — Success
//! Criteria call for exercising the *real* fake-agent subprocess). This
//! port is therefore a concrete `AcpRuntime` struct, not a trait, per
//! Requirement 1's decision guidance. `AcpSessionStore`/`AcpAgentRegistry`
//! *are* traits (contract.ts declares them as such, and a GPUI app
//! plausibly wants to substitute its own storage/registry), and
//! `PermissionRequestHandler` is reused as-is from Phase 3 (ADR-6 already
//! solved "async, non-blocking decision callback" — redefining an
//! equivalent callback type here would duplicate it).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::schema::v1::McpServer;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::permissions::PermissionRequestHandler;
use crate::runtime::engine::session_options::SessionAgentOptions;
use crate::session::record::SessionRecord;
use crate::types::{NonInteractivePermissionPolicy, PermissionMode};

use super::errors::AcpRuntimeError;
use super::events::{
    AcpRuntimeAvailableCommand, AcpRuntimeEvent, AcpRuntimeUsageBreakdown, AcpRuntimeUsageCost,
};

/// Ports `AcpRuntimePromptMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpRuntimePromptMode {
    Prompt,
    Steer,
}

/// Ports `AcpRuntimeSessionMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpRuntimeSessionMode {
    Persistent,
    Oneshot,
}

/// Ports `AcpRuntimeControl`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpRuntimeControl {
    SetMode,
    SetConfigOption,
    Status,
}

impl AcpRuntimeControl {
    pub fn as_str(self) -> &'static str {
        match self {
            AcpRuntimeControl::SetMode => "session/set_mode",
            AcpRuntimeControl::SetConfigOption => "session/set_config_option",
            AcpRuntimeControl::Status => "session/status",
        }
    }
}

/// Ports `AcpRuntimeHandle`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpRuntimeHandle {
    pub session_key: String,
    pub backend: String,
    pub runtime_session_name: String,
    pub cwd: Option<String>,
    pub acpx_record_id: Option<String>,
    pub backend_session_id: Option<String>,
    pub agent_session_id: Option<String>,
}

/// Ports `AcpRuntimeEnsureInput`.
pub struct AcpRuntimeEnsureInput {
    pub session_key: String,
    pub agent: String,
    pub mode: AcpRuntimeSessionMode,
    pub resume_session_id: Option<String>,
    pub cwd: Option<PathBuf>,
    pub session_options: Option<SessionAgentOptions>,
}

/// Ports `AcpRuntimeTurnAttachment`.
#[derive(Debug, Clone)]
pub struct AcpRuntimeTurnAttachment {
    pub media_type: String,
    pub data: String,
}

/// Ports `AcpRuntimeTurnInput`.
pub struct AcpRuntimeTurnInput {
    pub handle: AcpRuntimeHandle,
    pub text: String,
    pub attachments: Vec<AcpRuntimeTurnAttachment>,
    pub mode: AcpRuntimePromptMode,
    pub request_id: String,
    pub timeout_ms: Option<u64>,
}

/// Ports `AcpRuntimeCapabilities`.
#[derive(Debug, Clone)]
pub struct AcpRuntimeCapabilities {
    pub controls: Vec<AcpRuntimeControl>,
    pub config_option_keys: Option<Vec<String>>,
}

/// Ports `AcpRuntimeSessionModels`.
#[derive(Debug, Clone, Default)]
pub struct AcpRuntimeSessionModels {
    pub current_model_id: Option<String>,
    pub available_model_ids: Vec<String>,
}

/// Ports `AcpRuntimeSessionUsage`.
#[derive(Debug, Clone, Default)]
pub struct AcpRuntimeSessionUsage {
    pub cumulative: Option<AcpRuntimeUsageBreakdown>,
    pub cost: Option<AcpRuntimeUsageCost>,
    pub per_request: HashMap<String, AcpRuntimeUsageBreakdown>,
}

/// Ports `AcpRuntimeStatus`.
#[derive(Debug, Clone, Default)]
pub struct AcpRuntimeStatus {
    pub summary: Option<String>,
    pub acpx_record_id: Option<String>,
    pub backend_session_id: Option<String>,
    pub agent_session_id: Option<String>,
    pub models: Option<AcpRuntimeSessionModels>,
    pub usage: Option<AcpRuntimeSessionUsage>,
    pub available_commands: Option<Vec<AcpRuntimeAvailableCommand>>,
}

/// Ports `AcpRuntimeDoctorReport`.
#[derive(Debug, Clone)]
pub struct AcpRuntimeDoctorReport {
    pub ok: bool,
    pub code: Option<String>,
    pub message: String,
    pub install_command: Option<String>,
    pub details: Vec<String>,
}

/// Ports `AcpRuntimeTurnResultError`.
#[derive(Debug, Clone)]
pub struct AcpRuntimeTurnResultError {
    pub message: String,
    pub code: Option<String>,
    pub detail_code: Option<String>,
    pub retryable: Option<bool>,
}

/// Ports `AcpRuntimeTurnResult`.
#[derive(Debug, Clone)]
pub enum AcpRuntimeTurnResult {
    Completed { stop_reason: Option<String> },
    Cancelled { stop_reason: Option<String> },
    Failed { error: AcpRuntimeTurnResultError },
}

/// Ports `AcpRuntimeTurn` (ADR-7): `events` is a pull-based `Stream` rather
/// than acpx's `AsyncIterable`, and `result` is a boxed future settled
/// exactly once the turn reaches a terminal state, kept separate from the
/// live event stream per acpx's own already-improved `startTurn` shape (see
/// the `done`/`error` doc comments on [`AcpRuntimeEvent`]).
pub struct AcpRuntimeTurn {
    pub request_id: String,
    events: Option<BoxStream<'static, AcpRuntimeEvent>>,
    result: BoxFuture<'static, AcpRuntimeTurnResult>,
    cancel_tx: Option<futures::channel::oneshot::Sender<Option<String>>>,
}

impl AcpRuntimeTurn {
    pub(crate) fn new(
        request_id: String,
        events: BoxStream<'static, AcpRuntimeEvent>,
        result: BoxFuture<'static, AcpRuntimeTurnResult>,
        cancel_tx: futures::channel::oneshot::Sender<Option<String>>,
    ) -> Self {
        Self {
            request_id,
            events: Some(events),
            result,
            cancel_tx: Some(cancel_tx),
        }
    }

    /// Takes the live event stream. Ports the `events` getter; a `take`
    /// rather than a `&self` borrow because `BoxStream` isn't `Clone` and
    /// acpx's own consumer only ever drains it once.
    pub fn events(&mut self) -> BoxStream<'static, AcpRuntimeEvent> {
        self.events
            .take()
            .unwrap_or_else(|| Box::pin(futures::stream::empty()))
    }

    /// Awaits the turn's terminal result. Ports the `result` getter.
    pub fn result(self) -> BoxFuture<'static, AcpRuntimeTurnResult> {
        self.result
    }

    /// Ports `cancel(...)`: requests cancellation; the turn's task sends
    /// `session/cancel` and keeps awaiting the agent's (now-cancelled)
    /// response rather than tearing down early, per ACP's cancellation
    /// protocol (the agent MUST still reply with `stopReason: cancelled`).
    pub fn cancel(&mut self, reason: Option<String>) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(reason);
        }
    }

    /// Ports `closeStream(...)`: stops draining live events early (e.g. the
    /// UI navigated away) without affecting the in-flight turn itself. The
    /// producer side uses a bounded, best-effort `try_send` (see
    /// `engine::prompt_turn`), so it never blocks even if nothing drains
    /// the stream after this is called.
    pub fn close_stream(&mut self) {
        self.events = None;
    }
}

/// Ports `AcpSessionStore`. `load`/`save` are modeled as fallible (acpx's
/// `Promise<T>` can reject too; Rust just makes that explicit in the
/// signature) over this crate's own [`SessionRecord`].
pub trait AcpSessionStore: Send + Sync {
    fn load(
        &self,
        session_id: String,
    ) -> BoxFuture<'static, crate::error::Result<Option<SessionRecord>>>;
    fn save(&self, record: SessionRecord) -> BoxFuture<'static, crate::error::Result<()>>;
}

/// Ports `AcpAgentRegistry`.
pub trait AcpAgentRegistry: Send + Sync {
    fn resolve(&self, agent_name: &str) -> String;
    fn list(&self) -> Vec<String>;
}

/// A ready-to-use [`AcpAgentRegistry`] over [`crate::agent_command::registry`]'s
/// built-in table, with optional caller overrides layered on top.
pub struct BuiltInAgentRegistry {
    overrides: Option<HashMap<String, String>>,
}

impl BuiltInAgentRegistry {
    pub fn new(overrides: Option<HashMap<String, String>>) -> Self {
        Self { overrides }
    }
}

impl Default for BuiltInAgentRegistry {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AcpAgentRegistry for BuiltInAgentRegistry {
    fn resolve(&self, agent_name: &str) -> String {
        crate::agent_command::resolve_agent_command(agent_name, self.overrides.as_ref())
    }

    fn list(&self) -> Vec<String> {
        crate::agent_command::list_built_in_agents(self.overrides.as_ref())
    }
}

/// Ports `AcpRuntimeOptions`, plus one documented addition: `terminal`.
/// acpx threads a terminal-capability flag through `AcpClient`'s
/// constructor options instead of the runtime-level options object; this
/// port has no separate per-call client-construction API (each session's
/// `AcpClient` is spawned internally by the engine), so the flag lives
/// here instead. `session_store`/`agent_registry` are trait objects per
/// contract.ts's own `AcpSessionStore`/`AcpAgentRegistry` interfaces —
/// [`BuiltInAgentRegistry`] is a ready-to-use `agent_registry`, and a
/// file-backed `session_store` lives at
/// `crate::session::persistence::file_session_store::FileAcpSessionStore`.
pub struct AcpRuntimeOptions {
    pub cwd: PathBuf,
    pub session_store: Arc<dyn AcpSessionStore>,
    pub agent_registry: Arc<dyn AcpAgentRegistry>,
    pub mcp_servers: Vec<McpServer>,
    pub permission_mode: PermissionMode,
    pub non_interactive_permissions: NonInteractivePermissionPolicy,
    pub timeout_ms: Option<u64>,
    pub probe_agent: Option<String>,
    pub verbose: bool,
    pub terminal: bool,
    pub on_permission_request: Option<Arc<dyn PermissionRequestHandler>>,
    /// Phase 6 addition (ADR-4): bound on each session's pending-prompt
    /// FIFO (`crate::queue::SessionPromptQueue`). `None` uses
    /// `crate::queue::DEFAULT_QUEUE_CAPACITY`. Not part of acpx's
    /// `AcpRuntimeOptions` (acpx has no in-process multi-item queue to
    /// bound — see `queue`'s module docs) but exposed here rather than
    /// hardcoded per Requirement 1/Step 3, so the embedding GPUI app can
    /// tune it.
    pub prompt_queue_capacity: Option<usize>,
}

/// Ports `AcpFileSessionStoreOptions`; re-exported for convenience so
/// callers don't need to reach into `session::store_options` separately
/// when they only want the file-backed store.
pub type AcpFileSessionStoreOptions = crate::session::store_options::AcpFileSessionStoreOptions;

pub(crate) fn attachment_content_blocks(
    text: &str,
    attachments: &[AcpRuntimeTurnAttachment],
) -> Result<Vec<agent_client_protocol::schema::v1::ContentBlock>, AcpRuntimeError> {
    use super::errors::AcpRuntimeErrorCode;
    use agent_client_protocol::schema::v1::{
        AudioContent, ContentBlock, ImageContent, TextContent,
    };

    if attachments.is_empty() {
        return Ok(vec![ContentBlock::Text(TextContent::new(text))]);
    }

    let mut blocks = Vec::with_capacity(attachments.len() + 1);
    if !text.is_empty() {
        blocks.push(ContentBlock::Text(TextContent::new(text)));
    }
    for attachment in attachments {
        if let Some(mime) = attachment.media_type.strip_prefix("image/") {
            blocks.push(ContentBlock::Image(ImageContent::new(
                format!("image/{mime}"),
                attachment.data.clone(),
            )));
            continue;
        }
        if let Some(mime) = attachment.media_type.strip_prefix("audio/") {
            blocks.push(ContentBlock::Audio(AudioContent::new(
                format!("audio/{mime}"),
                attachment.data.clone(),
            )));
            continue;
        }
        return Err(AcpRuntimeError::new(
            AcpRuntimeErrorCode::TurnFailed,
            format!(
                "Unsupported ACP runtime attachment media type: {}",
                attachment.media_type
            ),
        ));
    }
    Ok(blocks)
}

/// Ports `legacyTerminalEventFromTurnResult`, used by [`AcpRuntime::run_turn`]'s
/// compatibility shim.
pub(crate) fn legacy_terminal_event_from_turn_result(
    result: &AcpRuntimeTurnResult,
) -> AcpRuntimeEvent {
    match result {
        AcpRuntimeTurnResult::Failed { error } => AcpRuntimeEvent::Error {
            message: error.message.clone(),
            code: error.code.clone(),
            detail_code: error.detail_code.clone(),
            retryable: error.retryable,
        },
        AcpRuntimeTurnResult::Completed { stop_reason }
        | AcpRuntimeTurnResult::Cancelled { stop_reason } => AcpRuntimeEvent::Done {
            stop_reason: stop_reason.clone(),
        },
    }
}
