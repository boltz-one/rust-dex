//! Agent Client Protocol (ACP) client/runtime, embeddable in the GPUI
//! desktop app.
//!
//! This crate ports the "Core + session persistence + queueing" slice of
//! `others/acpx` (a TypeScript ACP CLI) into Rust. CLI/commander surface,
//! the cross-process IPC queue daemon, and the flows DSL are explicitly out
//! of scope. See `plans/20260705-1718-acpx-to-acp-crate-port/plan.md` for
//! the full scope, phase breakdown, and architecture decisions.

pub mod agent_command;
pub mod agent_session_id;
pub mod auth_env;
pub mod client;
pub mod control;
pub mod error;
pub mod error_normalization;
pub mod error_shapes;
pub mod filesystem;
pub mod jsonrpc_gap;
pub mod mcp_servers;
pub mod permissions;
mod platform;
pub mod queue;
pub mod runtime;
pub mod session;
pub mod session_control_errors;
pub mod terminal;
pub mod types;
pub mod version;

pub use error::{AcpError, Result};
