//! Runtime truncation of a [`SessionConversation`], applied after every
//! mutation so persisted files don't grow unboundedly.
//!
//! Ports `trimConversationForRuntime` and its helpers from
//! `others/acpx/src/session/conversation-model.ts`. Truncation order
//! matches acpx exactly (phase-05 Risk Assessment calls this out
//! explicitly): message-count trim first (drop oldest messages), *then*
//! per-field char-limit trim applied to whatever messages remain, then the
//! `request_token_usage` map is capped last.

use super::conversation::SessionConversation;
use super::limits::{
    MAX_RUNTIME_AGENT_TEXT_CHARS, MAX_RUNTIME_MESSAGES, MAX_RUNTIME_REQUEST_TOKEN_USAGE,
    MAX_RUNTIME_THINKING_CHARS, MAX_RUNTIME_TOOL_IO_CHARS,
};
use super::message::{
    SessionAgentContent, SessionMessage, SessionToolResultContent, SessionUserContent,
};

/// Ports `trimRuntimeText`: truncates to `max_chars`, appending `...` when
/// truncation actually happened (matching acpx's `slice(0, max - 3) + "..."`
/// exactly, including its behavior for `max_chars < 3`).
pub fn trim_runtime_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let keep = max_chars.saturating_sub(3);
    let truncated: String = value.chars().take(keep).collect();
    format!("{truncated}...")
}

/// Ports `trimConversationForRuntime`.
pub fn trim_conversation_for_runtime(conversation: &mut SessionConversation) {
    if conversation.messages.len() > MAX_RUNTIME_MESSAGES {
        let drop = conversation.messages.len() - MAX_RUNTIME_MESSAGES;
        conversation.messages.drain(0..drop);
    }

    for message in &mut conversation.messages {
        trim_runtime_message(message);
    }

    if conversation.request_token_usage.len() > MAX_RUNTIME_REQUEST_TOKEN_USAGE {
        // acpx keeps the *last* N entries in insertion order
        // (`Object.entries(...).slice(-N)`); a `HashMap` has no stable
        // insertion order, so this port approximates it by keeping an
        // arbitrary N entries. Phase 4/6 (which own live request tracking)
        // should prefer an ordered map if exact "most recent" semantics
        // become load-bearing.
        let keep: Vec<_> = conversation
            .request_token_usage
            .drain()
            .take(MAX_RUNTIME_REQUEST_TOKEN_USAGE)
            .collect();
        conversation.request_token_usage = keep.into_iter().collect();
    }
}

fn trim_runtime_message(message: &mut SessionMessage) {
    match message {
        SessionMessage::User(user) => {
            for content in &mut user.content {
                if let SessionUserContent::Text(text) = content {
                    *text = trim_runtime_text(text, MAX_RUNTIME_AGENT_TEXT_CHARS);
                }
            }
        }
        SessionMessage::Agent(agent) => {
            for content in &mut agent.content {
                trim_runtime_agent_content(content);
            }
            for result in agent.tool_results.values_mut() {
                if let SessionToolResultContent::Text(text) = &mut result.content {
                    *text = trim_runtime_text(text, MAX_RUNTIME_TOOL_IO_CHARS);
                }
            }
        }
        SessionMessage::Resume => {}
    }
}

fn trim_runtime_agent_content(content: &mut SessionAgentContent) {
    match content {
        SessionAgentContent::Text(text) => {
            *text = trim_runtime_text(text, MAX_RUNTIME_AGENT_TEXT_CHARS);
        }
        SessionAgentContent::Thinking { text, .. } => {
            *text = trim_runtime_text(text, MAX_RUNTIME_THINKING_CHARS);
        }
        SessionAgentContent::ToolUse(tool_use) => {
            tool_use.raw_input = trim_runtime_text(&tool_use.raw_input, MAX_RUNTIME_TOOL_IO_CHARS);
        }
        SessionAgentContent::RedactedThinking(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::conversation_model::conversation::create_session_conversation;
    use crate::session::conversation_model::message::{SessionAgentMessage, SessionUserMessage};

    #[test]
    fn trim_runtime_text_truncates_and_appends_ellipsis() {
        let long = "a".repeat(10);
        assert_eq!(trim_runtime_text(&long, 5), "aa...");
        assert_eq!(trim_runtime_text("short", 10), "short");
    }

    #[test]
    fn caps_message_count_at_200_dropping_oldest() {
        let mut conversation = create_session_conversation(None);
        for i in 0..250 {
            conversation
                .messages
                .push(SessionMessage::User(SessionUserMessage {
                    id: format!("m{i}"),
                    content: vec![SessionUserContent::Text(format!("msg-{i}"))],
                }));
        }

        trim_conversation_for_runtime(&mut conversation);

        assert_eq!(conversation.messages.len(), MAX_RUNTIME_MESSAGES);
        // Oldest 50 messages (m0..m49) were dropped; m50 now leads.
        let first = conversation.messages.first().unwrap().as_user().unwrap();
        assert_eq!(first.id, "m50");
        let last = conversation.messages.last().unwrap().as_user().unwrap();
        assert_eq!(last.id, "m249");
    }

    #[test]
    fn trims_oversized_agent_text_message() {
        let mut conversation = create_session_conversation(None);
        let oversized = "x".repeat(MAX_RUNTIME_AGENT_TEXT_CHARS + 500);
        conversation
            .messages
            .push(SessionMessage::Agent(SessionAgentMessage {
                content: vec![SessionAgentContent::Text(oversized)],
                ..Default::default()
            }));

        trim_conversation_for_runtime(&mut conversation);

        let agent = conversation.messages[0].as_agent().unwrap();
        let SessionAgentContent::Text(text) = &agent.content[0] else {
            panic!("expected Text content");
        };
        assert_eq!(text.chars().count(), MAX_RUNTIME_AGENT_TEXT_CHARS);
        assert!(text.ends_with("..."));
    }
}
