//! `TerminalManager`: `terminal/create|output|kill|release|wait_for_exit`.
//! Ports `others/acpx/src/acp/terminal-manager.ts`'s public surface (the
//! 884-line TS file's process-group descendant tracking is deliberately
//! simplified away — see `terminal::tracking`'s module docs for why that's
//! safe here).
//!
//! Output capture is poll-style (`terminal/output` returns whatever's been
//! buffered so far) per Step 7's "implement polling first" guidance; the
//! return type is [`output::OutputSnapshot`] rather than a raw string so a
//! future streaming API can be added without breaking this one.

mod kill;
pub mod output;
pub mod spawn;
pub mod tracking;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::schema::v1::{
    CreateTerminalRequest, CreateTerminalResponse, KillTerminalRequest, KillTerminalResponse,
    ReleaseTerminalRequest, ReleaseTerminalResponse, TerminalId, TerminalOutputRequest,
    TerminalOutputResponse, WaitForTerminalExitRequest, WaitForTerminalExitResponse,
};
use parking_lot::Mutex;

pub use output::{DEFAULT_TERMINAL_OUTPUT_LIMIT_BYTES, OutputSnapshot};

use crate::error::{AcpError, Result};
use crate::permissions::{PermissionRequestHandler, confirm_action};
use crate::types::{NonInteractivePermissionPolicy, PermissionMode};
use output::OutputBuffer;
use spawn::spawn_terminal_process;

struct ManagedTerminal {
    child: smol::lock::Mutex<util::process::Child>,
    pid: u32,
    output: Arc<OutputBuffer>,
    // Kept alive for the terminal's lifetime; dropping cancels them.
    _readers: (smol::Task<()>, smol::Task<()>),
}

pub struct TerminalManagerOptions {
    pub cwd: PathBuf,
    pub permission_mode: PermissionMode,
    pub non_interactive_policy: NonInteractivePermissionPolicy,
    pub handler: Option<Arc<dyn PermissionRequestHandler>>,
    pub kill_grace: Option<Duration>,
}

/// Ports the `TerminalManager` class. All operations are gated by the same
/// `PermissionMode`/`NonInteractivePermissionPolicy` acpx enforces (see
/// `permissions::confirm_action`).
pub struct TerminalManager {
    cwd: PathBuf,
    permission_mode: PermissionMode,
    non_interactive_policy: NonInteractivePermissionPolicy,
    handler: Option<Arc<dyn PermissionRequestHandler>>,
    kill_grace: Duration,
    terminals: Mutex<HashMap<TerminalId, Arc<ManagedTerminal>>>,
}

impl TerminalManager {
    pub fn new(options: TerminalManagerOptions) -> Self {
        Self {
            cwd: options.cwd,
            permission_mode: options.permission_mode,
            non_interactive_policy: options.non_interactive_policy,
            handler: options.handler,
            kill_grace: options.kill_grace.unwrap_or(kill::DEFAULT_KILL_GRACE),
            terminals: Mutex::new(HashMap::new()),
        }
    }

    /// Ports `updatePermissionPolicy`.
    pub fn update_permission_policy(
        &mut self,
        permission_mode: PermissionMode,
        non_interactive_policy: NonInteractivePermissionPolicy,
    ) {
        self.permission_mode = permission_mode;
        self.non_interactive_policy = non_interactive_policy;
    }

    /// Ports `createTerminal`.
    pub async fn create_terminal(
        &self,
        params: CreateTerminalRequest,
    ) -> Result<CreateTerminalResponse> {
        let command_line = command_line_description(&params.command, &params.args);
        let approved = confirm_action(
            self.permission_mode,
            self.non_interactive_policy,
            self.handler.as_deref(),
            params.session_id.clone(),
            format!("Allow terminal command \"{command_line}\"?"),
        )
        .await?;
        if !approved {
            return Err(AcpError::PermissionDenied("terminal/create".to_string()));
        }

        let output_byte_limit = params
            .output_byte_limit
            .unwrap_or(output::DEFAULT_TERMINAL_OUTPUT_LIMIT_BYTES);
        let mut spawned = spawn_terminal_process(&params, &self.cwd)?;

        let output = Arc::new(OutputBuffer::new());
        let stdout_reader = output::spawn_reader(
            spawned.child.stdout.take(),
            output.clone(),
            output_byte_limit,
        );
        let stderr_reader = output::spawn_reader(
            spawned.child.stderr.take(),
            output.clone(),
            output_byte_limit,
        );

        let terminal = Arc::new(ManagedTerminal {
            child: smol::lock::Mutex::new(spawned.child),
            pid: spawned.pid,
            output,
            _readers: (stdout_reader, stderr_reader),
        });

        let terminal_id = TerminalId::new(uuid::Uuid::new_v4().to_string());
        self.terminals.lock().insert(terminal_id.clone(), terminal);
        Ok(CreateTerminalResponse::new(terminal_id))
    }

    /// Ports `terminalOutput`.
    pub async fn terminal_output(
        &self,
        params: TerminalOutputRequest,
    ) -> Result<TerminalOutputResponse> {
        let terminal = self.get_terminal(&params.terminal_id)?;
        let snapshot = terminal.output.snapshot();
        let exit_status = tracking::current_exit_status(&terminal.child).await;
        Ok(
            TerminalOutputResponse::new(snapshot.output, snapshot.truncated)
                .exit_status(exit_status),
        )
    }

    /// Ports `waitForTerminalExit`.
    pub async fn wait_for_terminal_exit(
        &self,
        params: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse> {
        let terminal = self.get_terminal(&params.terminal_id)?;
        let status = tracking::poll_exit_status(&terminal.child).await;
        Ok(WaitForTerminalExitResponse::new(status))
    }

    /// Ports `killTerminal`.
    pub async fn kill_terminal(&self, params: KillTerminalRequest) -> Result<KillTerminalResponse> {
        let terminal = self.get_terminal(&params.terminal_id)?;
        kill::kill_process(&terminal, self.kill_grace).await;
        Ok(KillTerminalResponse::new())
    }

    /// Ports `releaseTerminal`.
    pub async fn release_terminal(
        &self,
        params: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse> {
        let terminal = self.terminals.lock().remove(&params.terminal_id);
        let Some(terminal) = terminal else {
            return Ok(ReleaseTerminalResponse::new());
        };
        kill::kill_process(&terminal, self.kill_grace).await;
        terminal.output.clear();
        Ok(ReleaseTerminalResponse::new())
    }

    /// Ports `shutdown`: releases every outstanding terminal.
    pub async fn shutdown(&self) {
        let ids: Vec<TerminalId> = self.terminals.lock().keys().cloned().collect();
        for terminal_id in ids {
            let _ = self
                .release_terminal(ReleaseTerminalRequest::new("shutdown", terminal_id))
                .await;
        }
    }

    fn get_terminal(&self, terminal_id: &TerminalId) -> Result<Arc<ManagedTerminal>> {
        self.terminals
            .lock()
            .get(terminal_id)
            .cloned()
            .ok_or_else(|| AcpError::Other(anyhow::anyhow!("unknown terminal: {terminal_id}")))
    }
}

/// Ports `toCommandLine`, approximated: acpx JSON-quotes each arg, this uses
/// Rust's `Debug` string quoting — close enough for a human-readable
/// permission-prompt description, not a wire format.
fn command_line_description(command: &str, args: &[String]) -> String {
    if args.is_empty() {
        return command.to_string();
    }
    let rendered = args
        .iter()
        .map(|arg| format!("{arg:?}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{command} {rendered}")
}

#[cfg(test)]
impl TerminalManager {
    fn terminal_pid(&self, terminal_id: &TerminalId) -> u32 {
        self.terminals
            .lock()
            .get(terminal_id)
            .expect("terminal exists")
            .pid
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
