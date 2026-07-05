//! `spawn_connected_session`: builds the filesystem/terminal/permission
//! handlers for one session, spawns its `AcpClient`, runs the reconnect
//! state machine, and persists the result. Split out of `manager.rs` purely
//! for the per-file line convention — takes `&AcpRuntimeOptions` rather than
//! `&AcpRuntime` since that's all it actually needs.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::agent_command::resolve_agent_program;
use crate::auth_env::build_agent_environment;
use crate::client::handlers::{ClientRequestHandlers, PermissionRequestWiring};
use crate::client::{AcpClient, SpawnAgentOptions};
use crate::filesystem::FilesystemHandlers;
use crate::session::conversation_model::iso_now;
use crate::session::record::SessionRecord;
use crate::terminal::{TerminalManager, TerminalManagerOptions};
use crate::types::SessionResumePolicy;

use super::connected_session::ConnectedSession;
use super::manager_support::{conversation_from_record, wrap_err};
use super::reconnect::connect_and_load_session;
use super::session_options::session_options_from_record;

use crate::runtime::public::contract::AcpRuntimeOptions;
use crate::runtime::public::errors::{AcpRuntimeError, AcpRuntimeErrorCode};

/// Ports the `AcpClient` construction + `connectAndLoadSession` half of
/// acpx's `withConnectedSession`/manager.ts session setup.
pub(super) async fn spawn_connected_session(
    options: &AcpRuntimeOptions,
    mut record: SessionRecord,
    resume_policy: SessionResumePolicy,
) -> Result<Arc<ConnectedSession>, AcpRuntimeError> {
    let agent_command = record.agent_command.clone();
    let parts = resolve_agent_program(&agent_command, None).map_err(|err| {
        wrap_err(
            AcpRuntimeErrorCode::BackendMissing,
            "failed to resolve agent command",
            err,
        )
    })?;
    let cwd = PathBuf::from(&record.cwd);
    let session_env = session_options_from_record(&record).and_then(|o| o.env);
    let env = build_agent_environment(std::env::vars(), None, session_env.as_ref(), cfg!(windows));

    let (notifications_tx, notifications_rx) = smol::channel::unbounded();
    let filesystem = Arc::new(
        FilesystemHandlers::new(
            &cwd,
            options.permission_mode,
            options.non_interactive_permissions,
            options.on_permission_request.clone(),
        )
        .map_err(|err| {
            wrap_err(
                AcpRuntimeErrorCode::SessionInitFailed,
                "failed to sandbox session cwd",
                err,
            )
        })?,
    );
    let terminal = Arc::new(TerminalManager::new(TerminalManagerOptions {
        cwd: cwd.clone(),
        permission_mode: options.permission_mode,
        non_interactive_policy: options.non_interactive_permissions,
        handler: options.on_permission_request.clone(),
        kill_grace: None,
    }));
    let handlers = ClientRequestHandlers {
        filesystem: Some(filesystem),
        terminal: Some(terminal),
        permission: PermissionRequestWiring {
            mode: options.permission_mode,
            non_interactive_policy: options.non_interactive_permissions,
            handler: options.on_permission_request.clone(),
        },
        notifications: Some(notifications_tx),
    };

    let client = AcpClient::spawn(SpawnAgentOptions {
        program: &parts.command,
        args: &parts.args,
        cwd: &cwd,
        env: &env,
        client_name: "boltz-acp".to_string(),
        terminal: options.terminal,
        handlers,
    })
    .await
    .map_err(|err| {
        wrap_err(
            AcpRuntimeErrorCode::SessionInitFailed,
            "failed to spawn ACP agent",
            err,
        )
    })?;

    record.protocol_version = Some(i64::from(client.init_response().protocol_version.as_u16()));
    record.agent_capabilities = Some(client.init_response().agent_capabilities.clone());

    let conversation = conversation_from_record(&record);

    let connect_result = connect_and_load_session(
        &client,
        &mut record,
        &conversation,
        &options.mcp_servers,
        resume_policy,
        options.timeout_ms.map(Duration::from_millis),
    )
    .await
    .map_err(|err| {
        wrap_err(
            AcpRuntimeErrorCode::SessionInitFailed,
            "failed to connect ACP session",
            err,
        )
    })?;

    record.pid = client.state().last_known_pid;
    record.agent_started_at = Some(client.state().agent_started_at.to_rfc3339());
    record.closed = false;
    record.closed_at = None;
    record.last_used_at = iso_now();

    options
        .session_store
        .save(record.clone())
        .await
        .map_err(|err| {
            wrap_err(
                AcpRuntimeErrorCode::SessionInitFailed,
                "failed to persist session record",
                err,
            )
        })?;

    Ok(ConnectedSession::new(
        client,
        connect_result.session_id,
        record,
        conversation,
        notifications_rx,
        options.mcp_servers.clone(),
        options.prompt_queue_capacity,
    ))
}
