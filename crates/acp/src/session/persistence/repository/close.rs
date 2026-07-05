//! Ports `closeSession`: best-effort terminate the recorded pid, mark the
//! record closed, and persist it.

use crate::error::Result;
use crate::session::conversation_model::iso_now;
use crate::session::record::SessionRecord;
use crate::session::store_options::AcpFileSessionStoreOptions;

use super::resolve::resolve_session_record;
use super::write::write_session_record;
use crate::session::persistence::index::rebuild_session_index;

#[cfg(unix)]
fn best_effort_terminate(pid: u32) {
    // SAFETY: sends standard termination signals to a pid recorded in a
    // session file. Not necessarily a child of this process (the agent may
    // have outlived a previous run of this crate's host), so this uses
    // `libc::kill` (single pid) rather than `libc::killpg` (process group) —
    // ports acpx's plain `process.kill(pid, signal)` calls in `closeSession`.
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGTERM);
        libc::kill(pid as libc::pid_t, libc::SIGKILL);
    }
}

#[cfg(windows)]
fn best_effort_terminate(pid: u32) {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(handle, 1);
            let _ = CloseHandle(handle);
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn best_effort_terminate(_pid: u32) {}

/// Ports `closeSession`.
pub fn close_session(options: &AcpFileSessionStoreOptions, id: &str) -> Result<SessionRecord> {
    let mut record = resolve_session_record(options, id)?;
    let now = iso_now();

    if let Some(pid) = record.pid {
        best_effort_terminate(pid);
    }

    record.closed = true;
    record.closed_at = Some(now.clone());
    record.pid = None;
    record.last_used_at = now.clone();
    record.last_prompt_at.get_or_insert(now);

    write_session_record(options, &record)?;
    let _ = rebuild_session_index(options);
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::persistence::serialize::test_support::sample_session_record;

    fn options(dir: &tempfile::TempDir) -> AcpFileSessionStoreOptions {
        AcpFileSessionStoreOptions::new(dir.path())
    }

    #[test]
    fn close_session_marks_closed_and_clears_pid() {
        let dir = tempfile::tempdir().unwrap();
        let options = options(&dir);
        let record = sample_session_record();
        write_session_record(&options, &record).unwrap();

        let closed = close_session(&options, &record.acpx_record_id).unwrap();
        assert!(closed.closed);
        assert!(closed.pid.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn close_session_terminates_a_live_recorded_pid() {
        let dir = tempfile::tempdir().unwrap();
        let options = options(&dir);
        let mut record = sample_session_record();

        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("failed to spawn sleep");
        record.pid = Some(child.id());
        write_session_record(&options, &record).unwrap();

        let closed = close_session(&options, &record.acpx_record_id).unwrap();
        assert!(closed.pid.is_none());

        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = child.try_wait();
        assert!(!crate::platform::is_process_alive(child.id()));
    }
}
