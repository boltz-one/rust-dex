//! Public `AcpClient` handle: ties together spawn, transport, handshake,
//! shutdown, and lifecycle state into the single object Phase 3/4/6 build
//! on. Ports the surface of `others/acpx/src/acp/client.ts` this phase owns
//! (handshake/session-new/shutdown) — prompt-queue state is Phase 6's.

pub mod handlers;
pub mod handshake;
pub mod shutdown;
pub mod spawn;
pub mod state;
pub mod transport;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use agent_client_protocol::schema::v1::{
    CancelNotification, InitializeResponse, LoadSessionRequest, LoadSessionResponse, McpServer,
    NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse, ResumeSessionRequest,
    ResumeSessionResponse, SessionConfigId, SessionConfigValueId, SessionId, SessionModeId,
    SetSessionConfigOptionRequest, SetSessionConfigOptionResponse, SetSessionModeRequest,
    SetSessionModeResponse,
};
use agent_client_protocol::{Agent, ConnectionTo, Error as AcpRpcError};

use crate::error::{AcpError, Result};
use crate::error_normalization::normalize_agent_error;
pub use handlers::ClientRequestHandlers;
use handshake::{RunningConnection, spawn_and_initialize};
use spawn::{SpawnOptions, spawn_agent_process};
use state::{AgentExitInfo, ClientState};
use transport::take_transport;

/// Everything needed to spawn one ACP agent and complete its handshake.
pub struct SpawnAgentOptions<'a> {
    pub program: &'a str,
    pub args: &'a [String],
    pub cwd: &'a Path,
    pub env: &'a HashMap<String, String>,
    pub client_name: String,
    pub terminal: bool,
    /// Whether `program`/`args` resolve to a Gemini CLI `--acp` invocation
    /// (see [`crate::agent_command::is_gemini_acp_command`]). When true,
    /// the `initialize` handshake races
    /// [`crate::agent_command::gemini_quirks::resolve_gemini_acp_startup_timeout_ms`]
    /// instead of waiting indefinitely, surfacing
    /// [`AcpError::GeminiAcpStartupTimeout`] on expiry (matching acpx's
    /// Gemini-specific startup-timeout handling).
    pub is_gemini: bool,
    /// Whether `program`/`args` resolve to a Devin ACP invocation (see
    /// [`crate::agent_command::is_devin_acp_command`]). When true, the
    /// advertised `clientInfo`/`clientCapabilities` are swapped for
    /// Devin's Windsurf compatibility identity (see
    /// `client::handshake`'s module docs).
    pub is_devin: bool,
    /// Phase 4's filesystem/terminal/permission handlers and `session/update`
    /// notification sink, registered on the connection before the
    /// `initialize` handshake runs. Defaults (all `None`/empty) are fine for
    /// a handshake-only client (e.g. [`crate::runtime::public::probe::probe_runtime`]).
    pub handlers: ClientRequestHandlers,
}

/// A running ACP agent subprocess plus its live connection handle. Owns no
/// prompt-queue state (see module docs) — that's [`crate::client`]'s
/// consumers in later phases, not this struct.
pub struct AcpClient {
    child: util::process::Child,
    connection: ConnectionTo<Agent>,
    init_response: InitializeResponse,
    state: ClientState,
    _task: smol::Task<std::result::Result<(), AcpRpcError>>,
    shutdown_tx: Option<futures::channel::oneshot::Sender<()>>,
}

impl AcpClient {
    /// Spawns the agent subprocess and performs the ACP `initialize`
    /// handshake. Ports the combined effect of acpx's `client-process.ts`
    /// spawn + `client.ts`'s `initializeAgentConnection`.
    pub async fn spawn(options: SpawnAgentOptions<'_>) -> Result<Self> {
        let mut child = spawn_agent_process(SpawnOptions {
            program: options.program,
            args: options.args,
            cwd: options.cwd,
            env: options.env,
        })?;
        let pid = child.id();
        let transport = take_transport(&mut child)?;

        let init_outcome = if options.is_gemini {
            let timeout = crate::agent_command::resolve_gemini_acp_startup_timeout_ms();
            match crate::control::with_timeout(
                spawn_and_initialize(
                    transport,
                    options.client_name.clone(),
                    options.terminal,
                    options.is_devin,
                    options.handlers,
                ),
                Some(timeout),
            )
            .await
            {
                Ok(inner) => inner,
                Err(AcpError::Timeout(_)) => {
                    // The agent hasn't finished `initialize` within Gemini's
                    // startup budget (usually stuck on interactive OAuth) —
                    // kill the hung subprocess rather than leaking it, then
                    // surface a Gemini-specific diagnostic.
                    let _ = child.kill();
                    let message = crate::agent_command::gemini_quirks::build_gemini_acp_startup_timeout_message(
                        options.program,
                    )
                    .await;
                    Err(AcpError::GeminiAcpStartupTimeout(message))
                }
                Err(other) => Err(other),
            }
        } else {
            spawn_and_initialize(
                transport,
                options.client_name.clone(),
                options.terminal,
                options.is_devin,
                options.handlers,
            )
            .await
        };

        let RunningConnection {
            connection,
            init_response,
            task,
            shutdown_tx,
        } = init_outcome.map_err(|err| attach_command(err, options.program))?;

        Ok(Self {
            child,
            connection,
            init_response,
            state: ClientState::new(pid),
            _task: task,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    pub fn init_response(&self) -> &InitializeResponse {
        &self.init_response
    }

    pub fn state(&self) -> &ClientState {
        &self.state
    }

    pub fn connection(&self) -> &ConnectionTo<Agent> {
        &self.connection
    }

    /// Ports the `session/new` half of acpx's `client.ts`.
    pub async fn session_new(
        &self,
        cwd: PathBuf,
        mcp_servers: Vec<McpServer>,
    ) -> Result<NewSessionResponse> {
        let request = NewSessionRequest::new(cwd).mcp_servers(mcp_servers);
        self.connection
            .send_request(request)
            .block_task()
            .await
            .map_err(|err| normalize_agent_error(err, String::new()))
    }

    /// Ports the `session/load` half of acpx's `client.ts` (used by
    /// [`crate::runtime::engine::reconnect`] when the agent advertises
    /// `agentCapabilities.loadSession`).
    pub async fn session_load(
        &self,
        session_id: SessionId,
        cwd: PathBuf,
        mcp_servers: Vec<McpServer>,
    ) -> Result<LoadSessionResponse> {
        let request = LoadSessionRequest::new(session_id, cwd).mcp_servers(mcp_servers);
        self.connection
            .send_request(request)
            .block_task()
            .await
            .map_err(|err| normalize_agent_error(err, String::new()))
    }

    /// Ports the `session/resume` half of acpx's `client.ts`.
    pub async fn session_resume(
        &self,
        session_id: SessionId,
        cwd: PathBuf,
        mcp_servers: Vec<McpServer>,
    ) -> Result<ResumeSessionResponse> {
        let request = ResumeSessionRequest::new(session_id, cwd).mcp_servers(mcp_servers);
        self.connection
            .send_request(request)
            .block_task()
            .await
            .map_err(|err| normalize_agent_error(err, String::new()))
    }

    /// Sends `session/prompt` and awaits the agent's terminal response.
    pub async fn prompt(&self, request: PromptRequest) -> Result<PromptResponse> {
        self.connection
            .send_request(request)
            .block_task()
            .await
            .map_err(|err| normalize_agent_error(err, String::new()))
    }

    /// Sends the `session/cancel` notification (fire-and-forget per ACP;
    /// the agent replies to the still-pending `session/prompt` request with
    /// `stopReason: cancelled`, it does not reply to this notification).
    pub fn cancel_session(&self, session_id: SessionId) -> Result<()> {
        self.connection
            .send_notification(CancelNotification::new(session_id))
            .map_err(|err| AcpError::Other(anyhow::anyhow!("failed to send session/cancel: {err}")))
    }

    /// Ports `AcpClient.setSessionMode`.
    pub async fn set_session_mode(
        &self,
        session_id: SessionId,
        mode_id: SessionModeId,
    ) -> Result<SetSessionModeResponse> {
        let request = SetSessionModeRequest::new(session_id, mode_id);
        crate::jsonrpc_gap::send_set_session_mode(&self.connection, request)
            .await
            .map_err(|err| normalize_agent_error(err, String::new()))
    }

    /// Ports `AcpClient.setSessionConfigOption`.
    pub async fn set_session_config_option(
        &self,
        session_id: SessionId,
        config_id: SessionConfigId,
        value: SessionConfigValueId,
    ) -> Result<SetSessionConfigOptionResponse> {
        let request = SetSessionConfigOptionRequest::new(session_id, config_id, value);
        crate::jsonrpc_gap::send_set_session_config_option(&self.connection, request)
            .await
            .map_err(|err| normalize_agent_error(err, String::new()))
    }

    /// Gracefully shuts down the agent process (SIGTERM grace, then
    /// SIGKILL escalation), tearing down the background connection task
    /// first so it doesn't race the process teardown.
    pub async fn shutdown(mut self) -> AgentExitInfo {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        let pid = self.state.last_known_pid.unwrap_or(0);
        let info = shutdown::shutdown_agent_process(&mut self.child, pid).await;
        self.state.record_exit(info.clone());
        info
    }
}

/// `spawn_and_initialize` doesn't know the resolved command string (only
/// `client/mod.rs` does); this fills it into `AgentStartup` errors for a
/// useful message.
fn attach_command(err: AcpError, command: &str) -> AcpError {
    match err {
        AcpError::AgentStartup {
            exit_code,
            signal,
            stderr_summary,
            ..
        } => AcpError::AgentStartup {
            command: command.to_string(),
            exit_code,
            signal,
            stderr_summary,
        },
        other => other,
    }
}
