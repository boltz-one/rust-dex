#![cfg(feature = "test-support")]
//! Phase 4 integration tests: drive the real `AcpRuntime` public contract
//! end to end against the real fake ACP agent binary (extended in this
//! phase to support `session/prompt` + `session/update` and
//! `session/resume`) — no mocks, per Success Criteria.

use std::collections::HashMap;

use boltz_acp::runtime::engine::session_options::SessionAgentOptions;
use boltz_acp::runtime::public::{
    AcpRuntime, AcpRuntimeEnsureInput, AcpRuntimeEvent, AcpRuntimeOptions, AcpRuntimePromptMode,
    AcpRuntimeSessionMode, AcpRuntimeTurnInput, AcpRuntimeTurnResult, BuiltInAgentRegistry,
};
use boltz_acp::session::persistence::FileAcpSessionStore;
use boltz_acp::session::store_options::AcpFileSessionStoreOptions;
use boltz_acp::types::{NonInteractivePermissionPolicy, PermissionMode};
use futures::StreamExt;

fn fake_agent_path() -> &'static str {
    env!("CARGO_BIN_EXE_acp-fake-agent")
}

fn test_runtime(state_dir: &std::path::Path, cwd: &std::path::Path) -> AcpRuntime {
    let overrides = HashMap::from([("test-agent".to_string(), fake_agent_path().to_string())]);
    let options = AcpRuntimeOptions {
        cwd: cwd.to_path_buf(),
        session_store: FileAcpSessionStore::new(AcpFileSessionStoreOptions::new(state_dir)),
        agent_registry: std::sync::Arc::new(BuiltInAgentRegistry::new(Some(overrides))),
        mcp_servers: Vec::new(),
        permission_mode: PermissionMode::ApproveAll,
        non_interactive_permissions: NonInteractivePermissionPolicy::Deny,
        timeout_ms: Some(5_000),
        probe_agent: None,
        verbose: false,
        terminal: false,
        on_permission_request: None,
        prompt_queue_capacity: None,
    };
    AcpRuntime::new(options)
}

/// Success Criteria #1: ensure_session -> start_turn -> drain events to
/// completion -> await result -> see `Completed`.
#[test]
fn full_lifecycle_ensure_session_and_prompt_turn() {
    smol::block_on(async {
        let state_dir = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        let runtime = test_runtime(state_dir.path(), cwd.path());

        let handle = runtime
            .ensure_session(AcpRuntimeEnsureInput {
                session_key: "full-lifecycle".to_string(),
                agent: "test-agent".to_string(),
                mode: AcpRuntimeSessionMode::Persistent,
                resume_session_id: None,
                cwd: None,
                session_options: None,
            })
            .await
            .expect("ensure_session should succeed against the real fake agent");

        assert!(handle.backend_session_id.is_some());

        let mut turn = runtime
            .start_turn(AcpRuntimeTurnInput {
                handle,
                text: "hello".to_string(),
                attachments: Vec::new(),
                mode: AcpRuntimePromptMode::Prompt,
                request_id: "req-1".to_string(),
                timeout_ms: None,
            })
            .await;

        let mut events = turn.events();
        let mut saw_text_delta = false;
        while let Some(event) = events.next().await {
            if let AcpRuntimeEvent::TextDelta { text, .. } = &event {
                assert_eq!(text, "hello from fake agent");
                saw_text_delta = true;
            }
        }
        assert!(
            saw_text_delta,
            "expected a text_delta event from the live session/update"
        );

        let result = turn.result().await;
        assert!(
            matches!(result, AcpRuntimeTurnResult::Completed { .. }),
            "expected Completed, got {result:?}"
        );
    });
}

/// Success Criteria #2: kill the fake agent process, `ensure_session` again
/// with the same key, confirm the runtime transparently reconnects (backend
/// session id unchanged, proving `session/resume` was used rather than a
/// silent fresh `session/new`) rather than a silent state loss or panic.
#[test]
fn reconnect_after_agent_crash_resumes_backend_session() {
    smol::block_on(async {
        let state_dir = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        let runtime = test_runtime(state_dir.path(), cwd.path());

        let session_options = SessionAgentOptions {
            env: Some(HashMap::from([(
                "ACP_FAKE_AGENT_EXIT_AFTER_MS".to_string(),
                "150".to_string(),
            )])),
            ..Default::default()
        };

        let first_handle = runtime
            .ensure_session(AcpRuntimeEnsureInput {
                session_key: "crash-recovery".to_string(),
                agent: "test-agent".to_string(),
                mode: AcpRuntimeSessionMode::Persistent,
                resume_session_id: None,
                cwd: None,
                session_options: Some(session_options),
            })
            .await
            .expect("initial ensure_session should succeed");
        let original_backend_session_id = first_handle
            .backend_session_id
            .clone()
            .expect("handle should carry a backend session id");

        // Let the fake agent's self-destruct timer fire (simulating a crash
        // while this process — the embedding app — keeps running).
        smol::Timer::after(std::time::Duration::from_millis(500)).await;

        let second_handle = runtime
            .ensure_session(AcpRuntimeEnsureInput {
                session_key: "crash-recovery".to_string(),
                agent: "test-agent".to_string(),
                mode: AcpRuntimeSessionMode::Persistent,
                resume_session_id: None,
                cwd: None,
                session_options: None,
            })
            .await
            .expect("ensure_session after a crash must reconnect, not fail silently");

        assert_eq!(
            second_handle.backend_session_id.as_deref(),
            Some(original_backend_session_id.as_str()),
            "expected the reconnected handle to resume the original backend session id \
             (proves session/resume was used, not a silent fresh session/new)"
        );
    });
}

/// Exercises `get_status`, `set_mode`, and `cancel` against the real fake
/// agent — not called out as a separate Success Criterion, but cheap
/// insurance that the rest of the public contract's methods (not just
/// `ensure_session`/`start_turn`) are wired correctly end to end.
#[test]
fn status_mode_and_cancel_round_trip_against_real_agent() {
    smol::block_on(async {
        let state_dir = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        let runtime = test_runtime(state_dir.path(), cwd.path());

        let handle = runtime
            .ensure_session(AcpRuntimeEnsureInput {
                session_key: "status-mode-cancel".to_string(),
                agent: "test-agent".to_string(),
                mode: AcpRuntimeSessionMode::Persistent,
                resume_session_id: None,
                cwd: None,
                session_options: None,
            })
            .await
            .expect("ensure_session should succeed");

        let status = runtime
            .get_status(&handle)
            .await
            .expect("get_status should succeed for a connected session");
        assert_eq!(
            status.backend_session_id.as_deref(),
            handle.backend_session_id.as_deref()
        );

        runtime
            .set_mode(&handle, "plan")
            .await
            .expect("the fake agent accepts any session/set_mode call");

        // No prompt is in flight, so `cancel` should report success as a
        // no-op rather than erroring.
        runtime
            .cancel(&handle, Some("test cleanup"))
            .await
            .expect("cancel with no active prompt should not error");

        let capabilities = runtime.get_capabilities();
        assert!(!capabilities.controls.is_empty());
    });
}
