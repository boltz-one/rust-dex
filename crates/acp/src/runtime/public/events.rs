//! `AcpRuntimeEvent` + the `session/update` -> event parser.
//!
//! Ports `others/acpx/src/runtime/public/events.ts`'s *event shapes*
//! faithfully, but not its *parsing mechanism*: acpx parses raw NDJSON text
//! lines (`parsePromptEventLine(line: string)`) because its only view of a
//! `session/update` notification is whatever un-typed JSON its hand-rolled
//! transport handed it. This crate receives the real, typed
//! `agent_client_protocol_schema::v1::SessionUpdate` enum from the SDK (per
//! ADR-1 — reuse the SDK for everything it covers), so [`parse_session_update`]
//! pattern-matches that enum directly instead of re-deriving JSON field
//! sniffing. Per the phase's framing, shape fidelity to `contract.ts`'s
//! `AcpRuntimeEvent` matters more than internal parsing-implementation
//! fidelity.

use agent_client_protocol::schema::v1::{
    ContentBlock, Plan, SessionUpdate, ToolCall, ToolCallContent, ToolCallLocation, ToolCallUpdate,
    ToolKind,
};

/// Ports `AcpSessionUpdateTag`. acpx's version is an open string union
/// (`| (string & {})`); a plain `String` is the direct Rust analog since
/// this crate has no need to exhaustively match on it (tags are attached to
/// events purely for the UI's benefit).
pub type AcpSessionUpdateTag = String;

/// Which stream a `text_delta` event belongs to. Ports the `"output" |
/// "thought"` union on `AcpRuntimeEvent`'s `text_delta` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpRuntimeTextStream {
    Output,
    Thought,
}

/// Ports `AcpRuntimeUsageCost`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AcpRuntimeUsageCost {
    pub amount: Option<f64>,
    pub currency: Option<String>,
}

/// Ports `AcpRuntimeUsageBreakdown`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AcpRuntimeUsageBreakdown {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cached_read_tokens: Option<u64>,
    pub cached_write_tokens: Option<u64>,
    pub thought_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl AcpRuntimeUsageBreakdown {
    pub(crate) fn is_empty(&self) -> bool {
        self.input_tokens.is_none()
            && self.output_tokens.is_none()
            && self.cached_read_tokens.is_none()
            && self.cached_write_tokens.is_none()
            && self.thought_tokens.is_none()
            && self.total_tokens.is_none()
    }
}

/// Ports `AcpRuntimeAvailableCommand`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpRuntimeAvailableCommand {
    pub name: String,
    pub description: Option<String>,
    pub has_input: Option<bool>,
}

/// Ports `AcpRuntimeEvent`. The `done`/`error` compatibility-terminal
/// variants exist only for [`crate::runtime::public::contract::AcpRuntime::run_turn`]'s
/// legacy shim (see that method's docs) — [`parse_session_update`] never
/// produces them; they're built directly from an
/// [`crate::runtime::public::contract::AcpRuntimeTurnResult`] instead.
#[derive(Debug, Clone, PartialEq)]
pub enum AcpRuntimeEvent {
    TextDelta {
        text: String,
        stream: AcpRuntimeTextStream,
        tag: Option<AcpSessionUpdateTag>,
    },
    Status {
        text: String,
        tag: Option<AcpSessionUpdateTag>,
        used: Option<u64>,
        size: Option<u64>,
        cost: Option<AcpRuntimeUsageCost>,
        breakdown: Option<AcpRuntimeUsageBreakdown>,
        available_commands: Option<Vec<AcpRuntimeAvailableCommand>>,
    },
    ToolCall {
        text: String,
        tag: Option<AcpSessionUpdateTag>,
        tool_call_id: Option<String>,
        status: Option<String>,
        title: Option<String>,
        kind: Option<ToolKind>,
        locations: Vec<ToolCallLocation>,
        raw_input: Option<serde_json::Value>,
        raw_output: Option<serde_json::Value>,
        content: Vec<ToolCallContent>,
    },
    /// Compatibility terminal event; see the enum's module docs.
    Done { stop_reason: Option<String> },
    /// Compatibility failure event; see the enum's module docs.
    Error {
        message: String,
        code: Option<String>,
        detail_code: Option<String>,
        retryable: Option<bool>,
    },
}

fn text_from_content_block(block: &ContentBlock) -> Option<&str> {
    match block {
        ContentBlock::Text(text) => Some(text.text.as_str()),
        _ => None,
    }
}

fn text_delta(text: &str, stream: AcpRuntimeTextStream, tag: &str) -> Option<AcpRuntimeEvent> {
    if text.is_empty() {
        return None;
    }
    Some(AcpRuntimeEvent::TextDelta {
        text: text.to_string(),
        stream,
        tag: Some(tag.to_string()),
    })
}

fn tool_call_text(title: &str, status: Option<&str>) -> String {
    match status {
        Some(status) if !status.is_empty() => format!("{title} ({status})"),
        _ => title.to_string(),
    }
}

fn status_str(status: agent_client_protocol::schema::v1::ToolCallStatus) -> &'static str {
    use agent_client_protocol::schema::v1::ToolCallStatus;
    match status {
        ToolCallStatus::Pending => "pending",
        ToolCallStatus::InProgress => "in_progress",
        ToolCallStatus::Completed => "completed",
        ToolCallStatus::Failed => "failed",
        // `#[non_exhaustive]`; see `parse_session_update`'s docs.
        _ => "pending",
    }
}

fn tool_call_event(call: &ToolCall) -> AcpRuntimeEvent {
    let status = status_str(call.status);
    AcpRuntimeEvent::ToolCall {
        text: tool_call_text(&call.title, Some(status)),
        tag: Some("tool_call".to_string()),
        tool_call_id: Some(call.tool_call_id.0.to_string()),
        status: Some(status.to_string()),
        title: Some(call.title.clone()),
        kind: Some(call.kind),
        locations: call.locations.clone(),
        raw_input: call.raw_input.clone(),
        raw_output: call.raw_output.clone(),
        content: call.content.clone(),
    }
}

fn tool_call_update_event(update: &ToolCallUpdate) -> AcpRuntimeEvent {
    let title = update
        .fields
        .title
        .clone()
        .unwrap_or_else(|| "tool call".to_string());
    let status = update.fields.status.map(status_str);
    AcpRuntimeEvent::ToolCall {
        text: tool_call_text(&title, status),
        tag: Some("tool_call_update".to_string()),
        tool_call_id: Some(update.tool_call_id.0.to_string()),
        status: status.map(str::to_string),
        title: update.fields.title.clone(),
        kind: update.fields.kind,
        locations: update.fields.locations.clone().unwrap_or_default(),
        raw_input: update.fields.raw_input.clone(),
        raw_output: update.fields.raw_output.clone(),
        content: update.fields.content.clone().unwrap_or_default(),
    }
}

fn plan_status_text(plan: &Plan) -> Option<AcpRuntimeEvent> {
    let first = plan.entries.first()?;
    Some(AcpRuntimeEvent::Status {
        text: format!("plan: {}", first.content),
        tag: Some("plan".to_string()),
        used: None,
        size: None,
        cost: None,
        breakdown: None,
        available_commands: None,
    })
}

fn usage_cost(
    meta_cost: Option<&agent_client_protocol::schema::v1::Cost>,
) -> Option<AcpRuntimeUsageCost> {
    meta_cost.map(|cost| AcpRuntimeUsageCost {
        amount: Some(cost.amount),
        currency: Some(cost.currency.clone()),
    })
}

/// Reads a `_meta.usage` breakdown if the agent attached one (Claude Code
/// does this; not every adapter does). Ports the relevant half of
/// `normalizeUsageBreakdown`.
fn usage_breakdown_from_meta(
    meta: Option<&agent_client_protocol::schema::v1::Meta>,
) -> Option<AcpRuntimeUsageBreakdown> {
    let usage = meta?.get("usage")?.as_object()?;
    let read_u64 = |keys: &[&str]| -> Option<u64> {
        keys.iter()
            .find_map(|key| usage.get(*key))
            .and_then(|value| value.as_u64())
    };
    let breakdown = AcpRuntimeUsageBreakdown {
        input_tokens: read_u64(&["inputTokens", "input_tokens"]),
        output_tokens: read_u64(&["outputTokens", "output_tokens"]),
        cached_read_tokens: read_u64(&[
            "cachedReadTokens",
            "cacheReadInputTokens",
            "cache_read_input_tokens",
        ]),
        cached_write_tokens: read_u64(&[
            "cachedWriteTokens",
            "cacheCreationInputTokens",
            "cache_creation_input_tokens",
        ]),
        thought_tokens: read_u64(&["thoughtTokens", "thought_tokens"]),
        total_tokens: read_u64(&["totalTokens", "total_tokens"]),
    };
    (!breakdown.is_empty()).then_some(breakdown)
}

fn available_commands_event(
    update: &agent_client_protocol::schema::v1::AvailableCommandsUpdate,
) -> AcpRuntimeEvent {
    let available_commands: Vec<AcpRuntimeAvailableCommand> = update
        .available_commands
        .iter()
        .map(|command| AcpRuntimeAvailableCommand {
            name: command.name.clone(),
            description: (!command.description.trim().is_empty())
                .then(|| command.description.clone()),
            has_input: Some(command.input.is_some()),
        })
        .collect();
    let text = if available_commands.is_empty() {
        "available commands updated".to_string()
    } else {
        format!("available commands updated ({})", available_commands.len())
    };
    AcpRuntimeEvent::Status {
        text,
        tag: Some("available_commands_update".to_string()),
        used: None,
        size: None,
        cost: None,
        breakdown: None,
        available_commands: Some(available_commands),
    }
}

fn status_event(tag: &str, text: String) -> AcpRuntimeEvent {
    AcpRuntimeEvent::Status {
        text,
        tag: Some(tag.to_string()),
        used: None,
        size: None,
        cost: None,
        breakdown: None,
        available_commands: None,
    }
}

/// Ports `parsePromptEventLine`, adapted to operate on the typed
/// `SessionUpdate` this crate actually receives (see module docs).
/// `UserMessageChunk` deliberately yields `None`: acpx's own dispatch table
/// has no entry for it either (a user-message echo isn't surfaced as a live
/// runtime event).
pub fn parse_session_update(update: &SessionUpdate) -> Option<AcpRuntimeEvent> {
    match update {
        SessionUpdate::UserMessageChunk(_) => None,
        SessionUpdate::AgentMessageChunk(chunk) => {
            let text = text_from_content_block(&chunk.content)?;
            text_delta(text, AcpRuntimeTextStream::Output, "agent_message_chunk")
        }
        SessionUpdate::AgentThoughtChunk(chunk) => {
            let text = text_from_content_block(&chunk.content)?;
            text_delta(text, AcpRuntimeTextStream::Thought, "agent_thought_chunk")
        }
        SessionUpdate::ToolCall(call) => Some(tool_call_event(call)),
        SessionUpdate::ToolCallUpdate(update) => Some(tool_call_update_event(update)),
        SessionUpdate::Plan(plan) => plan_status_text(plan),
        SessionUpdate::AvailableCommandsUpdate(update) => Some(available_commands_event(update)),
        SessionUpdate::CurrentModeUpdate(update) => Some(status_event(
            "current_mode_update",
            format!("mode updated: {}", update.current_mode_id.0),
        )),
        SessionUpdate::ConfigOptionUpdate(update) => Some(status_event(
            "config_option_update",
            format!("config updated ({})", update.config_options.len()),
        )),
        SessionUpdate::SessionInfoUpdate(update) => {
            let text = update
                .title
                .clone()
                .take()
                .unwrap_or_else(|| "session updated".to_string());
            Some(status_event("session_info_update", text))
        }
        SessionUpdate::UsageUpdate(update) => {
            let text = format!("usage updated: {}/{}", update.used, update.size);
            Some(AcpRuntimeEvent::Status {
                text,
                tag: Some("usage_update".to_string()),
                used: Some(update.used),
                size: Some(update.size),
                cost: usage_cost(update.cost.as_ref()),
                breakdown: usage_breakdown_from_meta(update.meta.as_ref()),
                available_commands: None,
            })
        }
        // `SessionUpdate` is `#[non_exhaustive]`: a future ACP schema
        // revision may add variants (e.g. `PlanUpdate`/`PlanRemoved` behind
        // the `unstable_plan_operations` feature, not enabled here). Treat
        // anything this crate doesn't recognize as "no live event", the
        // same default acpx's dispatch table gives an unmatched tag.
        _ => None,
    }
}

// Split out per the workspace's <200-line file guideline; logically still
// part of this module (`super::*` sees its private items).
#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
