//! Ports `others/acpx/src/runtime/engine/session-options.ts`: the live
//! per-session agent options (`SessionAgentOptions`) applied when a fresh
//! ACP session is created, and their persisted form on
//! [`crate::session::acpx_state::SessionOptions`].

use std::collections::HashMap;

use crate::session::acpx_state::{SessionOptions, SystemPromptOption};
use crate::session::record::SessionRecord;

/// Ports `SessionAgentOptions`. Threaded into a fresh `session/new` request
/// (system prompt / env) and persisted onto the new record so a later
/// reconnect can recreate an equivalent session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionAgentOptions {
    pub model: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_turns: Option<u32>,
    pub system_prompt: Option<SystemPromptOption>,
    pub env: Option<HashMap<String, String>>,
}

fn merge_env(
    fallback: Option<&HashMap<String, String>>,
    preferred: Option<&HashMap<String, String>>,
) -> Option<HashMap<String, String>> {
    if fallback.is_none() && preferred.is_none() {
        return None;
    }
    let mut merged = fallback.cloned().unwrap_or_default();
    if let Some(preferred) = preferred {
        merged.extend(preferred.clone());
    }
    Some(merged)
}

/// Ports `mergeSessionOptions`: `preferred` fields win over `fallback`,
/// `env` maps are shallow-merged (`preferred` wins per key).
pub fn merge_session_options(
    preferred: Option<&SessionAgentOptions>,
    fallback: Option<&SessionAgentOptions>,
) -> Option<SessionAgentOptions> {
    let merged = SessionAgentOptions {
        model: preferred
            .and_then(|p| p.model.clone())
            .or_else(|| fallback.and_then(|f| f.model.clone())),
        allowed_tools: preferred
            .and_then(|p| p.allowed_tools.clone())
            .or_else(|| fallback.and_then(|f| f.allowed_tools.clone())),
        max_turns: preferred
            .and_then(|p| p.max_turns)
            .or_else(|| fallback.and_then(|f| f.max_turns)),
        system_prompt: preferred
            .and_then(|p| p.system_prompt.clone())
            .or_else(|| fallback.and_then(|f| f.system_prompt.clone())),
        env: merge_env(
            fallback.and_then(|f| f.env.as_ref()),
            preferred.and_then(|p| p.env.as_ref()),
        ),
    };
    has_any_option(&merged).then_some(merged)
}

fn has_any_option(options: &SessionAgentOptions) -> bool {
    options.model.is_some()
        || options.allowed_tools.is_some()
        || options.max_turns.is_some()
        || options.system_prompt.is_some()
        || options.env.is_some()
}

fn persisted_session_options(options: &SessionAgentOptions) -> Option<SessionOptions> {
    let persisted = SessionOptions {
        model: options.model.clone(),
        allowed_tools: options.allowed_tools.clone(),
        max_turns: options.max_turns,
        system_prompt: options.system_prompt.clone(),
        env: options.env.clone(),
    };
    has_stored_session_options(&persisted).then_some(persisted)
}

fn has_stored_session_options(options: &SessionOptions) -> bool {
    options.model.is_some()
        || options.allowed_tools.is_some()
        || options.max_turns.is_some()
        || options.system_prompt.is_some()
        || options.env.is_some()
}

/// Ports `persistSessionOptions`.
pub fn persist_session_options(record: &mut SessionRecord, options: Option<&SessionAgentOptions>) {
    let next = options.and_then(persisted_session_options);
    if let Some(next) = next {
        let mut acpx = record.acpx.take().unwrap_or_default();
        acpx.session_options = Some(next);
        record.acpx = Some(acpx);
        return;
    }

    if let Some(acpx) = &mut record.acpx {
        acpx.session_options = None;
    }
}

/// Ports `sessionOptionsFromRecord`.
pub fn session_options_from_record(record: &SessionRecord) -> Option<SessionAgentOptions> {
    let stored = record.acpx.as_ref()?.session_options.as_ref()?;
    let options = SessionAgentOptions {
        model: stored.model.clone(),
        allowed_tools: stored.allowed_tools.clone(),
        max_turns: stored.max_turns,
        system_prompt: stored.system_prompt.clone(),
        env: stored.env.clone(),
    };
    has_any_option(&options).then_some(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::persistence::serialize::test_support::sample_session_record;

    #[test]
    fn merge_prefers_preferred_over_fallback() {
        let fallback = SessionAgentOptions {
            model: Some("fallback-model".into()),
            max_turns: Some(5),
            ..Default::default()
        };
        let preferred = SessionAgentOptions {
            model: Some("preferred-model".into()),
            ..Default::default()
        };
        let merged = merge_session_options(Some(&preferred), Some(&fallback)).unwrap();
        assert_eq!(merged.model.as_deref(), Some("preferred-model"));
        assert_eq!(merged.max_turns, Some(5));
    }

    #[test]
    fn persist_and_read_round_trips() {
        let mut record = sample_session_record();
        let options = SessionAgentOptions {
            model: Some("gpt-5".into()),
            ..Default::default()
        };
        persist_session_options(&mut record, Some(&options));
        let read_back = session_options_from_record(&record).unwrap();
        assert_eq!(read_back.model.as_deref(), Some("gpt-5"));
    }

    #[test]
    fn persist_none_clears_existing_options() {
        let mut record = sample_session_record();
        let options = SessionAgentOptions {
            model: Some("gpt-5".into()),
            ..Default::default()
        };
        persist_session_options(&mut record, Some(&options));
        persist_session_options(&mut record, None);
        assert!(session_options_from_record(&record).is_none());
    }
}
