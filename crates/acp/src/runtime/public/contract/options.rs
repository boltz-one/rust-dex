//! `AcpRuntimeOptions` (the runtime's construction-time configuration) plus
//! the two free functions that bridge it and [`super::turn`]'s types to the
//! rest of `runtime::public`: turn-attachment encoding and the legacy
//! terminal-event compatibility shim.

use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::schema::v1::McpServer;

use crate::permissions::PermissionRequestHandler;
use crate::runtime::public::errors::{AcpRuntimeError, AcpRuntimeErrorCode};
use crate::runtime::public::events::AcpRuntimeEvent;
use crate::types::{NonInteractivePermissionPolicy, PermissionMode};

use super::registry::{AcpAgentRegistry, AcpSessionStore};
use super::types::{AcpRuntimeTurnAttachment, AcpRuntimeTurnResult};

/// Ports `AcpRuntimeOptions`, plus one documented addition: `terminal`.
/// acpx threads a terminal-capability flag through `AcpClient`'s
/// constructor options instead of the runtime-level options object; this
/// port has no separate per-call client-construction API (each session's
/// `AcpClient` is spawned internally by the engine), so the flag lives
/// here instead. `session_store`/`agent_registry` are trait objects per
/// contract.ts's own `AcpSessionStore`/`AcpAgentRegistry` interfaces —
/// [`super::registry::BuiltInAgentRegistry`] is a ready-to-use
/// `agent_registry`, and a file-backed `session_store` lives at
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

/// Ports `legacyTerminalEventFromTurnResult`, used by
/// [`crate::runtime::engine::manager::AcpRuntime::run_turn`]'s compatibility
/// shim.
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
