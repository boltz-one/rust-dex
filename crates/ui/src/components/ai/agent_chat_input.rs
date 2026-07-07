use std::sync::Arc;

use gpui::{ClickEvent, SharedString};

use crate::InputGroup;
use crate::prelude::*;

/// A mention/command trigger at the start of a chat draft (`@` or `/`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerChar {
    Mention,
    Command,
}

impl TriggerChar {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '@' => Some(TriggerChar::Mention),
            '/' => Some(TriggerChar::Command),
            _ => None,
        }
    }

    fn hint_label(self) -> &'static str {
        match self {
            TriggerChar::Mention => "Mentioning…",
            TriggerChar::Command => "Running command…",
        }
    }
    fn icon(self) -> IconName {
        match self {
            TriggerChar::Mention => IconName::AtSign,
            TriggerChar::Command => IconName::Terminal,
        }
    }
}

/// Detects a leading mention/command trigger in `draft`, returning the
/// trigger plus the query substring up to the first whitespace. Pure
/// detection only — resolving candidates is the caller's job.
fn detect_trigger(draft: &str) -> Option<(TriggerChar, &str)> {
    let mut chars = draft.chars();
    let first = chars.next()?;
    let trigger = TriggerChar::from_char(first)?;
    let rest = &draft[first.len_utf8()..];
    // A bare trigger followed by whitespace (e.g. "@ ") is no longer in-progress.
    if rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some((trigger, rest.split_whitespace().next().unwrap_or("")))
}

/// A multi-line agent chat input on [`InputGroup`] chrome, with a hint row
/// for `@`/`/` triggers and a send button. Pure builder — the caller owns
/// the draft text and `sending` state and reacts via callbacks.
#[derive(IntoElement, RegisterComponent)]
pub struct AgentChatInput {
    id: ElementId,
    draft: SharedString,
    placeholder: SharedString,
    sending: bool,
    on_send: Option<Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_trigger: Option<Arc<dyn Fn(TriggerChar, SharedString, &mut Window, &mut App) + 'static>>,
}

impl AgentChatInput {
    pub fn new(id: impl Into<ElementId>, draft: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            draft: draft.into(),
            placeholder: "Ask anything…".into(),
            sending: false,
            on_send: None,
            on_trigger: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    /// Disables the send button and shows a busy state while `true`.
    pub fn sending(mut self, sending: bool) -> Self {
        self.sending = sending;
        self
    }

    pub fn on_send(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_send = Some(Arc::new(handler));
        self
    }

    /// Called during render when `draft` starts a mention/command trigger,
    /// with the trigger char and the query substring typed so far.
    pub fn on_trigger(
        mut self,
        handler: impl Fn(TriggerChar, SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_trigger = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for AgentChatInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let trigger = detect_trigger(&self.draft).map(|(t, q)| (t, q.to_string()));

        if let (Some((trigger, query)), Some(on_trigger)) = (&trigger, &self.on_trigger) {
            on_trigger(*trigger, query.clone().into(), window, cx);
        }

        let is_empty = self.draft.is_empty();
        let content = if is_empty {
            Label::new(self.placeholder).color(Color::Placeholder)
        } else {
            Label::new(self.draft)
        };

        let disabled = self.sending || is_empty;
        let send_button = IconButton::new(("send", 0usize), IconName::PlayFilled)
            .icon_size(IconSize::Small)
            .disabled(disabled)
            .when_some(self.on_send, |this, handler| {
                this.on_click(move |event, window, cx| handler(event, window, cx))
            });

        v_flex()
            .id(self.id)
            .w_full()
            .gap_1()
            .when_some(trigger, |this, (trigger, query)| {
                let icon = Icon::new(trigger.icon())
                    .size(IconSize::XSmall)
                    .color(Color::Muted);
                let label = Label::new(trigger.hint_label())
                    .size(LabelSize::Small)
                    .color(Color::Muted);
                this.child(h_flex().gap_1().px_1().child(icon).child(label).when(
                    !query.is_empty(),
                    |this| {
                        this.child(
                            Label::new(query)
                                .size(LabelSize::Small)
                                .color(Color::Accent),
                        )
                    },
                ))
            })
            .child(
                InputGroup::new(div().w_full().min_h(px(64.)).child(content)).trailing(send_button),
            )
    }
}

impl Component for AgentChatInput {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let sending = AgentChatInput::new("chat-sending", "Explain this function").sending(true);
        Some(
            v_flex()
                .w_96()
                .gap_4()
                .child(single_example(
                    "Empty",
                    AgentChatInput::new("chat-empty", "").into_any_element(),
                ))
                .child(single_example(
                    "Mention trigger",
                    AgentChatInput::new("chat-mention", "@readm").into_any_element(),
                ))
                .child(single_example("Sending", sending.into_any_element()))
                .into_any_element(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_trigger_on_plain_text() {
        assert_eq!(detect_trigger("hello world"), None);
    }
    #[test]
    fn mention_trigger_detected() {
        assert_eq!(
            detect_trigger("@alice"),
            Some((TriggerChar::Mention, "alice"))
        );
    }
    #[test]
    fn command_trigger_detected() {
        assert_eq!(
            detect_trigger("/run tests"),
            Some((TriggerChar::Command, "run"))
        );
    }
    #[test]
    fn bare_trigger_has_empty_query() {
        assert_eq!(detect_trigger("@"), Some((TriggerChar::Mention, "")));
    }
    #[test]
    fn trigger_followed_by_space_is_not_in_progress() {
        assert_eq!(detect_trigger("@ hello"), None);
    }
    #[test]
    fn non_trigger_leading_char_ignored() {
        assert_eq!(detect_trigger("hello @alice"), None);
    }
}
