//! `AcpRuntime`: the thin coordinator wiring `engine::{connected_session,
//! reconnect, prompt_turn, reuse_policy, session_options}` behind the
//! `public::contract` API. Ports the orchestration half of
//! `others/acpx/src/runtime/engine/manager.ts` (`AcpRuntimeManager`) —
//! `manager.ts`'s 1445 lines are dominated by inline helpers this port
//! already split into their own modules (Key Insight in the phase file);
//! this file is deliberately just the coordination glue.

use std::collections::HashMap;
use std::sync::Arc;

use futures::StreamExt;
use futures::stream::BoxStream;
use parking_lot::Mutex;

use crate::session::conversation_model::iso_now;
use crate::session::mode_preference::{set_desired_config_option, set_desired_mode_id};
use crate::session::record::SessionRecord;
use crate::types::SessionResumePolicy;

use super::connected_session::ConnectedSession;
use super::manager_support::{
    create_initial_record, failed_turn, record_id_for, resume_policy_for_mode,
    runtime_status_from_record, wrap_err,
};
use super::reuse_policy::{ReuseCandidate, should_reuse_existing_record};
use super::session_options::persist_session_options;

use crate::runtime::public::contract::{
    AcpRuntimeCapabilities, AcpRuntimeControl, AcpRuntimeDoctorReport, AcpRuntimeEnsureInput,
    AcpRuntimeHandle, AcpRuntimeOptions, AcpRuntimeStatus, AcpRuntimeTurn, AcpRuntimeTurnInput,
    legacy_terminal_event_from_turn_result,
};
use crate::runtime::public::errors::{AcpRuntimeError, AcpRuntimeErrorCode};
use crate::runtime::public::events::AcpRuntimeEvent;
use crate::runtime::public::probe::probe_runtime;

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

    /// Ports `startTurn` (ADR-7). Phase 6 (ADR-4) adjustment: this is now
    /// `async` — it enqueues onto the session's `SessionPromptQueue`
    /// (`connected_session.rs`'s `prompt_queue` field) rather than calling
    /// `engine::prompt_turn::start_turn` directly, and only resolves once
    /// it's this request's turn to actually run (immediately if the
    /// session was idle, matching acpx's synchronous-looking `startTurn`
    /// shape; after earlier same-session requests complete otherwise). See
    /// `crate::queue`'s module docs for the full design. This is the
    /// sanctioned Phase 4 API adjustment flagged in this phase's Risk
    /// Assessment ("if Phase 4 ships `prompt_turn::run_turn` as the only
    /// entry point... the public `AcpRuntime::start_turn` signature needs a
    /// follow-up change") — `prompt_turn::start_turn` is now an
    /// implementation detail `crate::queue::dispatcher` calls, not
    /// something this method invokes directly.
    pub async fn start_turn(&self, input: AcpRuntimeTurnInput) -> AcpRuntimeTurn {
        let request_id = input.request_id.clone();
        let connected = match self.connected(&input.handle) {
            Ok(connected) => connected,
            Err(err) => return failed_turn(request_id, err),
        };
        let timeout_ms = self.options.timeout_ms;
        let session_store = self.options.session_store.clone();
        match connected
            .prompt_queue
            .enqueue(connected.clone(), session_store, input, timeout_ms)
            .await
        {
            Ok(turn) => turn,
            Err(err) => failed_turn(request_id, err.into()),
        }
    }

    /// Ports `runTurn`: the compatibility adapter that folds the terminal
    /// result back into the event stream as a `done`/`error` event. Prefer
    /// [`Self::start_turn`]. `async` for the same reason as `start_turn`.
    pub async fn run_turn(
        &self,
        input: AcpRuntimeTurnInput,
    ) -> BoxStream<'static, AcpRuntimeEvent> {
        let mut turn = self.start_turn(input).await;
        let events = turn.events();
        let result = turn.result();
        let terminal =
            futures::stream::once(
                async move { legacy_terminal_event_from_turn_result(&result.await) },
            );
        Box::pin(events.chain(terminal))
    }

    /// Ports `getCapabilities`.
    pub fn get_capabilities(&self) -> AcpRuntimeCapabilities {
        AcpRuntimeCapabilities {
            controls: vec![
                AcpRuntimeControl::SetMode,
                AcpRuntimeControl::SetConfigOption,
                AcpRuntimeControl::Status,
            ],
            config_option_keys: None,
        }
    }

    /// Ports `getStatus`.
    pub async fn get_status(
        &self,
        handle: &AcpRuntimeHandle,
    ) -> Result<AcpRuntimeStatus, AcpRuntimeError> {
        let connected = self.connected(handle)?;
        let record = connected.record.lock();
        Ok(runtime_status_from_record(&record))
    }

    /// Ports `setMode`.
    pub async fn set_mode(
        &self,
        handle: &AcpRuntimeHandle,
        mode: &str,
    ) -> Result<(), AcpRuntimeError> {
        let connected = self.connected(handle)?;
        connected.set_session_mode(mode).await.map_err(|err| {
            wrap_err(
                AcpRuntimeErrorCode::BackendUnsupportedControl,
                "session/set_mode failed",
                err,
            )
        })?;
        {
            let mut record = connected.record.lock();
            set_desired_mode_id(&mut record, Some(mode));
        }
        self.persist(&connected).await
    }

    /// Ports `setConfigOption`.
    pub async fn set_config_option(
        &self,
        handle: &AcpRuntimeHandle,
        key: &str,
        value: &str,
    ) -> Result<(), AcpRuntimeError> {
        let connected = self.connected(handle)?;
        connected
            .set_session_config_option(key, value)
            .await
            .map_err(|err| {
                wrap_err(
                    AcpRuntimeErrorCode::BackendUnsupportedControl,
                    "session/set_config_option failed",
                    err,
                )
            })?;
        {
            let mut record = connected.record.lock();
            set_desired_config_option(&mut record, key, Some(value));
        }
        self.persist(&connected).await
    }

    async fn persist(&self, connected: &Arc<ConnectedSession>) -> Result<(), AcpRuntimeError> {
        let snapshot = connected.record.lock().clone();
        self.options
            .session_store
            .save(snapshot)
            .await
            .map_err(|err| {
                wrap_err(
                    AcpRuntimeErrorCode::SessionInitFailed,
                    "failed to persist session record",
                    err,
                )
            })
    }

    /// Ports `doctor`.
    pub async fn doctor(&self) -> AcpRuntimeDoctorReport {
        let report = probe_runtime(&self.options).await;
        AcpRuntimeDoctorReport {
            ok: report.ok,
            code: None,
            message: report.message,
            install_command: None,
            details: report.details,
        }
    }

    /// Ports `cancel`.
    pub async fn cancel(
        &self,
        handle: &AcpRuntimeHandle,
        reason: Option<&str>,
    ) -> Result<(), AcpRuntimeError> {
        let connected = self.connected(handle)?;
        if let Some(reason) = reason {
            log::info!(
                "[acp] cancelling active prompt on {} ({reason})",
                handle.session_key
            );
        }
        connected.request_cancel_active_prompt().map_err(|err| {
            wrap_err(
                AcpRuntimeErrorCode::TurnFailed,
                "failed to cancel active prompt",
                err,
            )
        })?;
        Ok(())
    }

    /// Number of prompt requests currently queued (not counting whichever
    /// one is running, if any) for a session. Mostly useful for
    /// diagnostics/tests; the Architecture section of Phase 6's plan calls
    /// this out as part of the intended public surface alongside
    /// `enqueue`/`cancel_active`.
    pub fn queue_len(&self, handle: &AcpRuntimeHandle) -> Result<usize, AcpRuntimeError> {
        let connected = self.connected(handle)?;
        Ok(connected.prompt_queue.queue_len())
    }

    /// Phase 6 (Requirement 3/Step 6): drops queued-but-not-yet-started
    /// prompt requests for a session, without touching whatever turn is
    /// currently running. This is deliberately *not* what [`Self::cancel`]
    /// does by default (least-surprise: cancelling "this" turn shouldn't
    /// silently discard a user's already-submitted next message) — call
    /// this explicitly, or [`Self::cancel_active_and_clear`] for the
    /// combined "stop everything" action. Returns how many requests were
    /// cleared.
    pub fn clear_queue(&self, handle: &AcpRuntimeHandle) -> Result<usize, AcpRuntimeError> {
        let connected = self.connected(handle)?;
        Ok(connected.prompt_queue.clear_queue())
    }

    /// Ports the "stop everything" UI action Step 6 calls out: cancels the
    /// active turn (if any) AND drops any queued-but-not-started requests
    /// for the session, as one combined call. Returns how many queued
    /// requests were cleared.
    pub async fn cancel_active_and_clear(
        &self,
        handle: &AcpRuntimeHandle,
        reason: Option<&str>,
    ) -> Result<usize, AcpRuntimeError> {
        self.cancel(handle, reason).await?;
        self.clear_queue(handle)
    }

    /// Ports `close`.
    pub async fn close(
        &self,
        handle: &AcpRuntimeHandle,
        reason: &str,
        discard_persistent_state: bool,
    ) -> Result<(), AcpRuntimeError> {
        log::info!("[acp] closing session {} ({reason})", handle.session_key);
        let removed = self.sessions.lock().remove(&handle.session_key);
        let Some(connected) = removed else {
            return Ok(());
        };

        if discard_persistent_state {
            // Best-effort: the record simply stops being reachable via
            // `ensure_session` again once removed from the live map; actual
            // deletion of the on-disk file is a repository-level operation
            // (`session::persistence::repository`) this trait-based
            // `AcpSessionStore` doesn't expose a `delete` for (acpx's own
            // `AcpSessionStore` interface doesn't either — only `load`/`save`).
        } else {
            let mut record = connected.record.lock().clone();
            record.closed = true;
            record.closed_at = Some(iso_now());
            record.last_used_at = iso_now();
            if let Err(err) = self.options.session_store.save(record).await {
                log::warn!("[acp] failed to persist closed session record: {err}");
            }
        }

        match Arc::try_unwrap(connected) {
            Ok(connected) => {
                connected.client.shutdown().await;
            }
            Err(_) => {
                // Still referenced (e.g. an in-flight turn's background
                // task holds a clone); it will finish and drop its own
                // reference. Cancel any active prompt so it doesn't linger
                // indefinitely.
            }
        }
        Ok(())
    }
}
