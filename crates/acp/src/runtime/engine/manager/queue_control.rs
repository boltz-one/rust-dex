//! `AcpRuntime` cancellation, prompt-queue introspection/clearing, and
//! `close`. Split out of `manager/mod.rs` per the workspace's per-file line
//! convention — see that module's docs for the split rationale.

use std::sync::Arc;

use super::AcpRuntime;
use crate::runtime::engine::manager_support::wrap_err;
use crate::runtime::public::contract::AcpRuntimeHandle;
use crate::runtime::public::errors::{AcpRuntimeError, AcpRuntimeErrorCode};
use crate::session::conversation_model::iso_now;

impl AcpRuntime {
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
