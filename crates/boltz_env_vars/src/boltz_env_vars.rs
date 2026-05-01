pub use env_var::{EnvVar, bool_env_var, env_var};
use std::sync::LazyLock;

/// Whether Boltz is running in stateless mode.
/// When true, Boltz will use in-memory databases instead of persistent storage.
pub static BOLTZ_STATELESS: LazyLock<bool> = bool_env_var!("BOLTZ_STATELESS");
