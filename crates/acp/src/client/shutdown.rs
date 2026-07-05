//! SIGTERM(1.5s)->SIGKILL(1s) grace-period shutdown. Ports the shutdown half
//! of `others/acpx/src/acp/client-process.ts`.
//!
//! `util::process::Child::kill` (per ADR-3) only sends SIGKILL
//! (`libc::killpg(pid, SIGKILL)`) — there is no signal-specific variant to
//! reuse for the initial graceful SIGTERM. Rather than extend `util` (a
//! near-universal workspace dependency) for one caller, this module sends
//! SIGTERM directly via `libc::killpg`, matching `util`'s own
//! whole-process-group semantics, then escalates to `util::process::Child::kill`
//! for the SIGKILL step so the final kill path stays shared.

use std::time::Duration;

use util::process::Child;

use crate::client::state::AgentExitInfo;
use crate::control::with_timeout;
use crate::platform::is_process_alive;

const SIGTERM_GRACE: Duration = Duration::from_millis(1500);
const SIGKILL_GRACE: Duration = Duration::from_secs(1);

/// Closes `child`'s stdin, sends SIGTERM to its process group, waits up to
/// [`SIGTERM_GRACE`] for it to exit, escalates to SIGKILL via `util`'s
/// [`Child::kill`] and waits up to [`SIGKILL_GRACE`], then reports how it
/// went down. Never returns an error: a shutdown sequence's job is to
/// guarantee the process is gone, not to propagate why waiting failed.
pub async fn shutdown_agent_process(child: &mut Child, pid: u32) -> AgentExitInfo {
    drop(child.stdin.take());

    // SAFETY: signals a process group this crate itself spawned (per
    // `util::process::Child::spawn`'s `set_pre_exec_to_start_new_session`).
    unsafe {
        libc::killpg(pid as libc::pid_t, libc::SIGTERM);
    }
    if with_timeout(wait_for_exit(child), Some(SIGTERM_GRACE))
        .await
        .is_ok()
    {
        return exit_info("sigterm", child).await;
    }

    if let Err(err) = child.kill() {
        log::warn!("failed to SIGKILL agent process {pid}: {err}");
    }
    if with_timeout(wait_for_exit(child), Some(SIGKILL_GRACE))
        .await
        .is_ok()
    {
        return exit_info("sigkill", child).await;
    }

    AgentExitInfo {
        reason: "sigkill_timeout".to_string(),
        exit_code: None,
        signal: is_process_alive(pid).then(|| "unknown".to_string()),
    }
}

async fn wait_for_exit(child: &mut Child) {
    let _ = child.status().await;
}

async fn exit_info(reason: &str, child: &mut Child) -> AgentExitInfo {
    let status = child.status().await.ok();
    AgentExitInfo {
        reason: reason.to_string(),
        exit_code: status.and_then(|s| s.code()),
        signal: signal_from_status(status),
    }
}

#[cfg(unix)]
fn signal_from_status(status: Option<std::process::ExitStatus>) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;
    status.and_then(|s| s.signal()).map(|s| s.to_string())
}

#[cfg(not(unix))]
fn signal_from_status(_status: Option<std::process::ExitStatus>) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::spawn::{SpawnOptions, spawn_agent_process};
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn graceful_process_exits_on_sigterm() {
        smol::block_on(async {
            let env = HashMap::new();
            let mut child = spawn_agent_process(SpawnOptions {
                program: "/bin/sleep",
                args: &["30".to_string()],
                cwd: Path::new("/tmp"),
                env: &env,
            })
            .expect("spawn should succeed");
            let pid = child.id();

            let info = shutdown_agent_process(&mut child, pid).await;
            assert_eq!(info.reason, "sigterm");
            assert!(!is_process_alive(pid));
        });
    }
}
