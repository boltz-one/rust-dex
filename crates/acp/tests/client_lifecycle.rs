#![cfg(feature = "test-support")]
//! Phase 2 integration tests: spawn the real fake ACP agent binary
//! (`tests/fixtures/fake_agent/main.rs`) as a subprocess and drive the
//! handshake/shutdown lifecycle end to end over real ndjson stdio. No
//! mocks — run with `cargo test -p boltz-acp --features test-support`.

use std::collections::HashMap;
use std::path::Path;

use boltz_acp::client::{AcpClient, SpawnAgentOptions};

fn fake_agent_path() -> &'static str {
    env!("CARGO_BIN_EXE_acp-fake-agent")
}

#[test]
fn spawn_handshake_and_shutdown() {
    smol::block_on(async {
        let env = HashMap::new();
        let client = AcpClient::spawn(SpawnAgentOptions {
            program: fake_agent_path(),
            args: &[],
            cwd: Path::new("/tmp"),
            env: &env,
            client_name: "boltz-acp-test".to_string(),
            terminal: true,
            handlers: Default::default(),
        })
        .await
        .expect("spawn+handshake should succeed");

        assert!(!client.init_response().agent_capabilities.load_session);

        let info = client.shutdown().await;
        assert_eq!(info.reason, "sigterm");
    });
}

#[test]
fn sigterm_ignoring_agent_escalates_to_sigkill() {
    smol::block_on(async {
        let mut env = HashMap::new();
        env.insert("ACP_FAKE_AGENT_IGNORE_SIGTERM".to_string(), "1".to_string());
        let client = AcpClient::spawn(SpawnAgentOptions {
            program: fake_agent_path(),
            args: &[],
            cwd: Path::new("/tmp"),
            env: &env,
            client_name: "boltz-acp-test".to_string(),
            terminal: false,
            handlers: Default::default(),
        })
        .await
        .expect("spawn+handshake should succeed");

        let info = client.shutdown().await;
        assert_eq!(info.reason, "sigkill");
    });
}

#[test]
fn session_new_returns_typed_response() {
    smol::block_on(async {
        let env = HashMap::new();
        let client = AcpClient::spawn(SpawnAgentOptions {
            program: fake_agent_path(),
            args: &[],
            cwd: Path::new("/tmp"),
            env: &env,
            client_name: "boltz-acp-test".to_string(),
            terminal: false,
            handlers: Default::default(),
        })
        .await
        .expect("spawn+handshake should succeed");

        let response = client
            .session_new(Path::new("/tmp").to_path_buf(), vec![])
            .await
            .expect("session/new should succeed");
        // Phase 4 changed the fake agent to suffix the session id with its
        // own pid (see `tests/fixtures/fake_agent/main.rs`'s module docs),
        // so a reconnect-after-crash test can tell resumed sessions (id
        // unchanged) apart from freshly created ones (new pid suffix).
        assert!(response.session_id.0.starts_with("fake-session-"));

        client.shutdown().await;
    });
}
