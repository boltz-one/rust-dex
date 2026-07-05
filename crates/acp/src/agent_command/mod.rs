//! Agent command resolution: named-agent -> `(program, args)`, per-agent CLI
//! quirks, and spawn-time command-line construction. Ports
//! `others/acpx/src/acp/agent-command.ts`, `others/acpx/src/agent-registry.ts`,
//! `others/acpx/src/acp/model-support.ts`, `others/acpx/src/acp/codex-compat.ts`,
//! and `others/acpx/src/spawn-command-options.ts`.
//!
//! Per the phase's open question on agent-specific coverage: Claude, Cursor,
//! and Codex detection/quirks are ported (acpx has dedicated compat files
//! for these three). Gemini, Copilot, and Devin detection predicates are
//! ported (cheap and needed for command classification), but their
//! additional runtime quirks (Gemini CLI version probing and
//! `--acp`/`--experimental-acp` rewriting, Copilot `--help` capability
//! probing, Devin's Windsurf client-identity spoofing) are deferred — they
//! require spawning an extra probe subprocess or a compatibility shim that
//! doesn't affect this phase's handshake/shutdown/registry surface.

pub mod agent_detect;
pub mod codex_compat;
pub mod command_args;
pub mod model_request;
pub mod model_support;
pub mod registry;
pub mod spawn_options;

pub use agent_detect::{
    is_claude_acp_command, is_copilot_acp_command, is_cursor_acp_command, is_devin_acp_command,
    is_gemini_acp_command,
};
pub use codex_compat::{is_codex_acp_command, is_legacy_zed_codex_acp_invocation};
pub use command_args::{
    CommandParts, basename_token, resolve_agent_close_after_stdin_end_ms,
    should_ignore_non_json_agent_output_line, split_command_line,
};
pub use model_request::{
    RequestedModelUnsupportedError, RequestedModelUnsupportedReason,
    assert_requested_model_supported, resolve_requested_model_id,
    supports_legacy_claude_code_model_metadata,
};
pub use model_support::{
    AvailableModel, SessionModelState, format_available_model_ids, model_state_from_config_options,
    model_state_from_legacy_response, model_state_from_session_response,
};
pub use registry::{
    DEFAULT_AGENT_NAME, built_in_agent_registry, list_built_in_agents, merge_agent_registry,
    normalize_agent_name, resolve_agent_command,
};

/// Resolves a `--agent` value (a registered name, alias, or raw command
/// line) all the way to `(program, args)`, combining [`registry`]'s
/// name resolution with [`command_args`]'s quoting-aware splitting. This is
/// the single entry point `client/spawn.rs` uses.
pub fn resolve_agent_program(
    agent_name_or_command: &str,
    overrides: Option<&std::collections::HashMap<String, String>>,
) -> crate::error::Result<CommandParts> {
    let command_line = resolve_agent_command(agent_name_or_command, overrides);
    split_command_line(&command_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_named_agent_to_program_and_args() {
        let parts = resolve_agent_program("cursor", None).unwrap();
        assert_eq!(parts.command, "cursor-agent");
        assert_eq!(parts.args, vec!["acp"]);
    }

    #[test]
    fn resolves_raw_command_line_directly() {
        let parts = resolve_agent_program("/usr/local/bin/my-agent --flag", None).unwrap();
        assert_eq!(parts.command, "/usr/local/bin/my-agent");
        assert_eq!(parts.args, vec!["--flag"]);
    }
}
