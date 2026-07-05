//! `AcpRuntime` status/control surface: capabilities, status snapshot,
//! `session/set_mode` and `session/set_config_option`, persistence, and
//! `doctor`. Split out of `manager/mod.rs` per the workspace's per-file
//! line convention — see that module's docs for the split rationale.

use std::sync::Arc;

use super::AcpRuntime;
use crate::runtime::engine::connected_session::ConnectedSession;
use crate::runtime::engine::manager_support::{runtime_status_from_record, wrap_err};
use crate::runtime::public::contract::{
    AcpRuntimeCapabilities, AcpRuntimeControl, AcpRuntimeDoctorReport, AcpRuntimeHandle,
    AcpRuntimeStatus,
};
use crate::runtime::public::errors::{AcpRuntimeError, AcpRuntimeErrorCode};
use crate::runtime::public::probe::probe_runtime;
use crate::session::mode_preference::{set_desired_config_option, set_desired_mode_id};

impl AcpRuntime {
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
}
