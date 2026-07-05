//! Minimal real ACP agent for Phase 2-4 integration tests. Speaks JSON-RPC
//! 2.0 over ndjson stdio (matching acpx's `test/mock-agent.ts` role, scoped
//! down to what these phases' tests exercise: `initialize`, `session/new`,
//! `session/prompt` (with a live `session/update` notification), and
//! `session/resume` (Phase 4's reconnect-after-crash test).
//!
//! Behavior toggles via env var (kept as one process, not a CLI flag parser,
//! since these are test-only knobs):
//! - `ACP_FAKE_AGENT_IGNORE_SIGTERM=1`: installs a `SIG_IGN` handler for
//!   SIGTERM before serving, so the SIGTERM->SIGKILL escalation test can
//!   observe the escalation actually happening.
//! - `ACP_FAKE_AGENT_EXIT_AFTER_MS=<n>`: exits the process (simulating a
//!   crash, no graceful shutdown) `n` milliseconds after startup. Set via
//!   `AcpRuntimeEnsureInput.session_options.env` (persisted onto the
//!   session record) so Phase 4's reconnect-after-crash integration test
//!   can make a *specific* spawned agent die without the test needing raw
//!   pid access to the process the runtime spawned internally.
//! - `ACP_FAKE_AGENT_PROMPT_DELAY_MS=<n>`: sleeps `n` milliseconds on the
//!   request-serving thread before responding to `session/prompt` (the
//!   live `session/update` notification is still emitted immediately, so a
//!   test can observe "a turn is in flight" before the delay elapses).
//!   Phase 6's queueing tests use this, same env-var-toggle pattern as
//!   above, to (a) prove two *different* sessions' turns overlap in time
//!   (each session gets its own spawned agent process, so one slow agent
//!   does not block another), and (b) prove a second `start_turn` on the
//!   *same* session does not begin until the first's delay has elapsed.
//! - `ACP_FAKE_AGENT_PROMPT_UPDATE_COUNT=<n>`: emits `n` sequential
//!   `session/update` notifications (`chunk-0`, `chunk-1`, ...) instead of
//!   the default single `"hello from fake agent"` chunk. Defaults to `1`
//!   (unset), preserving every existing test's assumption of exactly one
//!   text delta. Phase 6's notification-ordering test sets this to observe
//!   several updates and assert they are processed in the order sent.
//! - `session/new` returns a session id suffixed with this process's pid
//!   (`fake-session-<pid>`), so a test can tell whether a second
//!   `ensure_session` call resumed the original backend session
//!   (`session/resume`, id unchanged) or silently created a new one
//!   (`session/new` again, id would change to the new process's pid).
//!
//! The request-serving loop runs on a background thread; the main thread
//! parks indefinitely instead of returning when stdin reaches EOF. A real
//! ACP agent's process lifetime is controlled by signals (this is exactly
//! what `client/shutdown.rs` exercises), not by "my stdin pipe closed" — the
//! shutdown sequence closes stdin *before* sending SIGTERM (mirroring
//! acpx's `client-process.ts`), so a process that exits on stdin-EOF would
//! make the SIGTERM/SIGKILL escalation untestable. Phase 4's crash-recovery
//! test instead sends SIGKILL directly to the recorded pid, simulating an
//! unclean death the shutdown sequence never runs for.

use std::io::{BufRead, Write};
use std::sync::Mutex;

use serde_json::{Value, json};

fn main() {
    #[cfg(unix)]
    if std::env::var("ACP_FAKE_AGENT_IGNORE_SIGTERM").as_deref() == Ok("1") {
        unsafe {
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
    }

    if let Ok(ms) = std::env::var("ACP_FAKE_AGENT_EXIT_AFTER_MS") {
        if let Ok(ms) = ms.parse::<u64>() {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                std::process::exit(1);
            });
        }
    }

    std::thread::spawn(serve_stdio);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

fn serve_stdio() {
    let stdin = std::io::stdin();
    let stdout = Mutex::new(std::io::stdout());

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        handle_message(&request, &stdout);
    }
}

fn write_line(stdout: &Mutex<std::io::Stdout>, payload: &Value) {
    let mut stdout = stdout.lock().expect("stdout mutex poisoned");
    let _ = writeln!(stdout, "{payload}");
    let _ = stdout.flush();
}

/// Handles one JSON-RPC message (request or notification). Requests with an
/// `id` always get exactly one response line; `session/prompt` additionally
/// emits a `session/update` notification first, so integration tests have a
/// live event to drain before the terminal response arrives.
fn handle_message(request: &Value, stdout: &Mutex<std::io::Stdout>) {
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return;
    };
    let id = request.get("id").cloned();

    if method == "session/prompt" {
        let session_id = request
            .get("params")
            .and_then(|p| p.get("sessionId"))
            .cloned()
            .unwrap_or(json!("fake-session-1"));
        let update_count: u32 = std::env::var("ACP_FAKE_AGENT_PROMPT_UPDATE_COUNT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        for i in 0..update_count.max(1) {
            let text = if update_count <= 1 {
                "hello from fake agent".to_string()
            } else {
                format!("chunk-{i}")
            };
            write_line(
                stdout,
                &json!({
                    "jsonrpc": "2.0",
                    "method": "session/update",
                    "params": {
                        "sessionId": session_id,
                        "update": {
                            "sessionUpdate": "agent_message_chunk",
                            "content": {"type": "text", "text": text},
                        },
                    },
                }),
            );
        }
        if let Ok(ms) = std::env::var("ACP_FAKE_AGENT_PROMPT_DELAY_MS") {
            if let Ok(ms) = ms.parse::<u64>() {
                std::thread::sleep(std::time::Duration::from_millis(ms));
            }
        }
    }

    let Some(id) = id else {
        // A notification (e.g. `session/cancel`) — nothing to respond to.
        return;
    };

    let result = match method {
        "initialize" => json!({
            "protocolVersion": 1,
            "agentCapabilities": {
                "loadSession": false,
                "sessionCapabilities": {
                    "resume": {},
                },
            },
            "authMethods": [],
        }),
        "session/new" => json!({
            "sessionId": format!("fake-session-{}", std::process::id()),
        }),
        "session/resume" => json!({}),
        "session/prompt" => json!({
            "stopReason": "end_turn",
        }),
        "session/set_mode" => json!({}),
        "session/set_config_option" => json!({}),
        _ => json!({}),
    };

    write_line(
        stdout,
        &json!({"jsonrpc": "2.0", "id": id, "result": result}),
    );
}
