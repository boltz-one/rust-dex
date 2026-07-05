//! Verifying an imported archive's agent identity against the caller's
//! expectation.
//!
//! Ports `assertExpectedAgentCommand`/`agentCommandMatchesExpected` from
//! `others/acpx/src/session/import.ts`. acpx's `commandLooksLikeBuiltInAgent`
//! regex-matches specific npm package names
//! (`@agentclientprotocol/codex-acp`, etc.) against the archived command
//! string; this port uses a simpler match against this crate's own
//! `crate::agent_command::normalize_agent_name` instead, since acpx's
//! npm-package-name heuristics don't apply to this port's own command
//! resolution conventions.

use crate::error::Result;
use crate::session::export::{ExportedSession, normalize_agent_identity};
use crate::session::record::SessionRecord;

use super::{ImportSessionOptions, import_error};

fn command_looks_like_built_in_agent(command: &str, agent_name: &str) -> bool {
    crate::agent_command::normalize_agent_name(command)
        == crate::agent_command::normalize_agent_name(agent_name)
}

fn agent_command_matches_expected(
    archived_command: &str,
    expected_agent_command: &str,
    expected_agent_name: Option<&str>,
) -> bool {
    if archived_command == expected_agent_command {
        return true;
    }
    expected_agent_name
        .map(|name| command_looks_like_built_in_agent(archived_command, name))
        .unwrap_or(false)
}

/// Ports `assertExpectedAgentCommand`.
pub(super) fn assert_expected_agent_command(
    archive: &ExportedSession,
    source_record: &mut SessionRecord,
    options: &ImportSessionOptions,
) -> Result<()> {
    let Some(expected_agent_command) = &options.expected_agent_command else {
        return Ok(());
    };
    let expected_agent_name = normalize_agent_identity(options.expected_agent_name.as_deref());
    let archive_command_matches = agent_command_matches_expected(
        &archive.session.agent,
        expected_agent_command,
        expected_agent_name.as_deref(),
    );
    let state_command_matches = agent_command_matches_expected(
        &source_record.agent_command,
        expected_agent_command,
        expected_agent_name.as_deref(),
    );

    if archive_command_matches && state_command_matches {
        source_record.agent_command = expected_agent_command.clone();
        return Ok(());
    }
    Err(import_error(
        "Session export archive agent does not match the requested agent",
    ))
}
