//! `AcpRuntime`: the thin coordinator wiring `engine::{connected_session,
//! reconnect, prompt_turn, reuse_policy, session_options}` behind the
//! `public::contract` API. Ports the orchestration half of
//! `others/acpx/src/runtime/engine/manager.ts` (`AcpRuntimeManager`) —
//! `manager.ts`'s 1445 lines are dominated by inline helpers this port
//! already split into their own modules (Key Insight in the phase file);
//! this file is deliberately just the coordination glue.
//!
//! Split (per the workspace's <200-line file guideline) by phase of
//! `AcpRuntime`'s API rather than by type: this module keeps the struct
//! definition plus session-acquisition (`new`/`ensure_session`/friends),
//! [`turn`] holds turn execution (`start_turn`/`run_turn`), [`status`]
//! holds status/mode/config-option/doctor calls, and [`queue_control`]
//! holds cancellation, queue introspection, and `close`. All are `impl
//! AcpRuntime` blocks on the one struct defined here — Rust allows an
//! `impl` in any descendant module of the type's defining module, and each
//! submodule only needs the private fields this file declares.

mod queue_control;
mod status;
mod turn;

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;

use super::connected_session::ConnectedSession;
use super::manager_support::{
    create_initial_record, record_id_for, resume_policy_for_mode, wrap_err,
};
use super::reuse_policy::{ReuseCandidate, should_reuse_existing_record};
use super::session_options::persist_session_options;

use crate::runtime::public::contract::{
    AcpRuntimeEnsureInput, AcpRuntimeHandle, AcpRuntimeOptions,
};
use crate::runtime::public::errors::{AcpRuntimeError, AcpRuntimeErrorCode};
use crate::session::record::SessionRecord;
use crate::types::SessionResumePolicy;

/// The concrete, embeddable ACP runtime. Ports `AcpRuntimeManager` /
/// `AcpRuntime` (see `public::contract`'s module docs for the
/// trait-vs-struct decision).
pub struct AcpRuntime {
    options: AcpRuntimeOptions,
    sessions: Mutex<HashMap<String, Arc<ConnectedSession>>>,
}

impl AcpRuntime {
    pub fn new(options: AcpRuntimeOptions) -> Self {
        Self {
            options,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    fn handle_for(
        &self,
        session_key: &str,
        backend: &str,
        connected: &ConnectedSession,
    ) -> AcpRuntimeHandle {
        let record = connected.record.lock();
        AcpRuntimeHandle {
            session_key: session_key.to_string(),
            backend: backend.to_string(),
            runtime_session_name: session_key.to_string(),
            cwd: Some(record.cwd.clone()),
            acpx_record_id: Some(record.acpx_record_id.clone()),
            backend_session_id: Some(record.acp_session_id.clone()),
            agent_session_id: record.agent_session_id.clone(),
        }
    }

    /// Ports `ensureSession`.
    pub async fn ensure_session(
        &self,
        input: AcpRuntimeEnsureInput,
    ) -> Result<AcpRuntimeHandle, AcpRuntimeError> {
        let agent_command = self.options.agent_registry.resolve(&input.agent);
        let record_id = record_id_for(&input.session_key, input.mode);
        let cwd = input
            .cwd
            .clone()
            .unwrap_or_else(|| self.options.cwd.clone());
        let cwd_string = cwd.to_string_lossy().into_owned();

        if let Some(connected) = self.sessions.lock().get(&input.session_key).cloned() {
            // A live in-memory entry is only actually reusable if its agent
            // process is still alive — otherwise this is exactly the
            // "agent crashed while the app kept running" case the
            // reconnect state machine (`engine::reconnect`) exists for, and
            // we must fall through to the persisted-record path below
            // rather than handing back a handle to a dead connection.
            let process_alive = connected
                .client
                .state()
                .last_known_pid
                .is_some_and(crate::platform::is_process_alive);
            let candidate = ReuseCandidate {
                cwd: &cwd,
                agent_command: &agent_command,
                resume_session_id: input.resume_session_id.as_deref(),
            };
            let reusable =
                process_alive && should_reuse_existing_record(&connected.record.lock(), &candidate);
            if reusable {
                return Ok(self.handle_for(&input.session_key, &agent_command, &connected));
            }
            self.sessions.lock().remove(&input.session_key);
        }

        let loaded = self
            .options
            .session_store
            .load(record_id.clone())
            .await
            .map_err(|err| {
                wrap_err(
                    AcpRuntimeErrorCode::SessionInitFailed,
                    "failed to load session record",
                    err,
                )
            })?;

        let candidate = ReuseCandidate {
            cwd: &cwd,
            agent_command: &agent_command,
            resume_session_id: input.resume_session_id.as_deref(),
        };
        let record = match loaded {
            Some(record) if should_reuse_existing_record(&record, &candidate) => record,
            _ => {
                let mut fresh = create_initial_record(&record_id, &agent_command, &cwd_string);
                persist_session_options(&mut fresh, input.session_options.as_ref());
                fresh
            }
        };

        let resume_policy = resume_policy_for_mode(input.mode);
        let connected = self.spawn_connected_session(record, resume_policy).await?;
        let handle = self.handle_for(&input.session_key, &agent_command, &connected);
        self.sessions
            .lock()
            .insert(input.session_key.clone(), connected);
        Ok(handle)
    }

    async fn spawn_connected_session(
        &self,
        record: SessionRecord,
        resume_policy: SessionResumePolicy,
    ) -> Result<Arc<ConnectedSession>, AcpRuntimeError> {
        super::manager_spawn::spawn_connected_session(&self.options, record, resume_policy).await
    }

    fn connected(
        &self,
        handle: &AcpRuntimeHandle,
    ) -> Result<Arc<ConnectedSession>, AcpRuntimeError> {
        self.sessions
            .lock()
            .get(&handle.session_key)
            .cloned()
            .ok_or_else(|| {
                AcpRuntimeError::new(
                    AcpRuntimeErrorCode::BackendUnavailable,
                    format!(
                        "no connected ACP session for session key {}",
                        handle.session_key
                    ),
                )
            })
    }
}
