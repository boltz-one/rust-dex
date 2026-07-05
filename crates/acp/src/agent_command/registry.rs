//! Ports the name -> command-line half of `others/acpx/src/agent-registry.ts`.
//!
//! Deliberately **not** ported: `resolveInstalledBuiltInAgentLaunch` /
//! `resolvePackageExecBuiltInAgentLaunch` / `BUILT_IN_AGENT_PACKAGES`. Those
//! functions search acpx's own `node_modules` tree (and shell to
//! `npm exec`) to avoid `npx`'s cold-start latency for a few first-party
//! adapters — an optimization specific to acpx shipping as an npm package.
//! This crate has no `node_modules` tree to search; the registry's `npx ...`
//! command strings below are exactly what acpx itself falls back to when
//! that optimization doesn't apply, so this is a straightforward substitute,
//! not a scope reduction.

use std::collections::HashMap;

/// Default agent when none is specified. Mirrors acpx's `DEFAULT_AGENT_NAME`.
pub const DEFAULT_AGENT_NAME: &str = "codex";

/// Built-in agent name -> shell command line, ported verbatim from acpx's
/// `AGENT_REGISTRY` (including its pinned adapter package version ranges).
pub fn built_in_agent_registry() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("pi", "npx pi-acp@^0.0.26"),
        ("openclaw", "openclaw acp"),
        ("codex", "npx -y @agentclientprotocol/codex-acp@^0.0.44"),
        (
            "claude",
            "npx -y @agentclientprotocol/claude-agent-acp@^0.37.0",
        ),
        ("gemini", "gemini --acp"),
        ("cursor", "cursor-agent acp"),
        ("copilot", "copilot --acp --stdio"),
        ("droid", "droid exec --output-format acp"),
        ("fast-agent", "uvx fast-agent-mcp acp"),
        ("grok-build", "grok agent stdio"),
        ("iflow", "iflow --experimental-acp"),
        ("kilocode", "npx -y @kilocode/cli acp"),
        ("kimi", "kimi acp"),
        ("kiro", "kiro-cli-chat acp"),
        ("mux", "npx -y mux@^0.27.0 acp"),
        ("opencode", "npx -y opencode-ai acp"),
        ("qoder", "qodercli --acp"),
        ("qwen", "qwen --acp"),
        ("trae", "traecli acp serve"),
    ])
}

/// Aliases resolved to a canonical registry key before lookup. Ports
/// acpx's `AGENT_ALIASES`.
fn agent_aliases() -> HashMap<&'static str, &'static str> {
    HashMap::from([("factory-droid", "droid"), ("factorydroid", "droid")])
}

/// Ports `normalizeAgentName`.
pub fn normalize_agent_name(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Ports `mergeAgentRegistry`: built-ins plus caller overrides (trimmed,
/// name-normalized, empty entries skipped).
pub fn merge_agent_registry(
    overrides: Option<&HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut merged: HashMap<String, String> = built_in_agent_registry()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let Some(overrides) = overrides else {
        return merged;
    };
    for (name, command) in overrides {
        let normalized = normalize_agent_name(name);
        let trimmed_command = command.trim();
        if normalized.is_empty() || trimmed_command.is_empty() {
            continue;
        }
        merged.insert(normalized, trimmed_command.to_string());
    }
    merged
}

/// Ports `resolveAgentCommand`: resolves a named agent (or alias) to its
/// registered command line, falling back to treating `agent_name` itself as
/// a raw command line when it isn't a known name.
pub fn resolve_agent_command(
    agent_name: &str,
    overrides: Option<&HashMap<String, String>>,
) -> String {
    let normalized = normalize_agent_name(agent_name);
    let registry = merge_agent_registry(overrides);
    if let Some(command) = registry.get(&normalized) {
        return command.clone();
    }
    let aliases = agent_aliases();
    if let Some(canonical) = aliases.get(normalized.as_str())
        && let Some(command) = registry.get(*canonical)
    {
        return command.clone();
    }
    agent_name.to_string()
}

/// Ports `listBuiltInAgents`.
pub fn list_built_in_agents(overrides: Option<&HashMap<String, String>>) -> Vec<String> {
    let mut names: Vec<String> = merge_agent_registry(overrides).into_keys().collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_known_agent_name() {
        assert_eq!(resolve_agent_command("Cursor", None), "cursor-agent acp");
    }

    #[test]
    fn resolves_alias() {
        assert_eq!(
            resolve_agent_command("factory-droid", None),
            "droid exec --output-format acp"
        );
    }

    #[test]
    fn unknown_name_passes_through_as_raw_command() {
        assert_eq!(
            resolve_agent_command("./my-agent --flag", None),
            "./my-agent --flag"
        );
    }

    #[test]
    fn overrides_win_over_built_ins() {
        let overrides = HashMap::from([("codex".to_string(), "my-codex-override".to_string())]);
        assert_eq!(
            resolve_agent_command("codex", Some(&overrides)),
            "my-codex-override"
        );
    }

    #[test]
    fn default_agent_name_is_registered() {
        assert!(built_in_agent_registry().contains_key(DEFAULT_AGENT_NAME));
    }
}
