use std::sync::Arc;

use gpui::{ClickEvent, SharedString};

use crate::prelude::*;
use crate::{AgentMarkdown, BadgeColor, BadgeVariant, Card, CardVariant, Disclosure};

/// Who/what produced a message: `User`/`Status` render as plain text,
/// `Assistant`/`Thinking` through [`AgentMarkdown`], `ToolCall` as a card.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AgentMessageRole {
    User,
    #[default]
    Assistant,
    ToolCall,
    Status,
    Thinking,
}

/// Lifecycle state of a `ToolCall` message, driving its status badge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolCallState {
    #[default]
    Running,
    Success,
    Failed,
}

/// A single chat message. Pure builder — all state is caller-owned.
#[derive(IntoElement, RegisterComponent)]
pub struct AgentMessageBubble {
    id: ElementId,
    role: AgentMessageRole,
    body: SharedString,
    tool_name: Option<SharedString>,
    tool_state: ToolCallState,
    tool_input: Option<SharedString>,
    tool_output: Option<SharedString>,
    expanded: bool,
    on_toggle_expanded: Option<Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl AgentMessageBubble {
    pub fn new(
        id: impl Into<ElementId>,
        role: AgentMessageRole,
        body: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            role,
            body: body.into(),
            tool_name: None,
            tool_state: ToolCallState::default(),
            tool_input: None,
            tool_output: None,
            expanded: false,
            on_toggle_expanded: None,
        }
    }

    /// `ToolCall` header title; falls back to `body` when unset.
    pub fn tool_name(mut self, name: impl Into<SharedString>) -> Self {
        self.tool_name = Some(name.into());
        self
    }

    pub fn tool_state(mut self, state: ToolCallState) -> Self {
        self.tool_state = state;
        self
    }
    pub fn tool_input(mut self, input: impl Into<SharedString>) -> Self {
        self.tool_input = Some(input.into());
        self
    }

    pub fn tool_output(mut self, output: impl Into<SharedString>) -> Self {
        self.tool_output = Some(output.into());
        self
    }

    /// Whether the raw input/output section is expanded (caller-owned).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn on_toggle_expanded(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_expanded = Some(Arc::new(handler));
        self
    }
}

fn raw_block(label: &'static str, content: SharedString, cx: &App) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
        .child(
            div()
                .p_2()
                .rounded_md()
                .bg(cx.theme().colors().element_background)
                .child(content),
        )
        .into_any_element()
}

fn tool_call_card(bubble: AgentMessageBubble, cx: &App) -> AnyElement {
    let (badge_label, badge_color) = match bubble.tool_state {
        ToolCallState::Running => ("Running", BadgeColor::Secondary),
        ToolCallState::Success => ("Success", BadgeColor::Success),
        ToolCallState::Failed => ("Failed", BadgeColor::Danger),
    };
    let disclosure_id = format!("{}-disclosure", bubble.id);
    let title = bubble.tool_name.unwrap_or_else(|| bubble.body.clone());
    let has_raw = bubble.tool_input.is_some() || bubble.tool_output.is_some();
    let expanded = bubble.expanded;

    let tool_icon = Icon::new(IconName::ToolHammer)
        .size(IconSize::Small)
        .color(Color::Muted);
    let badge = Badge::new(badge_label)
        .variant(BadgeVariant::Soft)
        .color(badge_color);
    let disclosure =
        Disclosure::new(disclosure_id, expanded).on_toggle_expanded(bubble.on_toggle_expanded);
    let header = h_flex()
        .id(bubble.id)
        .w_full()
        .justify_between()
        .child(h_flex().gap_2().child(tool_icon).child(Label::new(title)))
        .child(
            h_flex()
                .gap_2()
                .child(badge)
                .when(has_raw, |this| this.child(disclosure)),
        );

    Card::new()
        .variant(CardVariant::Bordered)
        .header(header)
        .when(has_raw && expanded, |this| {
            this.child(
                v_flex()
                    .gap_2()
                    .when_some(bubble.tool_input, |this, input| {
                        this.child(raw_block("Input", input, cx))
                    })
                    .when_some(bubble.tool_output, |this, output| {
                        this.child(raw_block("Output", output, cx))
                    }),
            )
        })
        .into_any_element()
}

impl RenderOnce for AgentMessageBubble {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        match self.role {
            AgentMessageRole::User => {
                let bubble = div()
                    .max_w(relative(0.8))
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .bg(cx.theme().colors().element_active)
                    .child(Label::new(self.body));
                h_flex()
                    .id(self.id)
                    .w_full()
                    .justify_end()
                    .child(bubble)
                    .into_any_element()
            }
            AgentMessageRole::Assistant => {
                let md = AgentMarkdown::new(format!("{}-md", self.id), self.body);
                div()
                    .id(self.id.clone())
                    .w_full()
                    .child(md)
                    .into_any_element()
            }
            AgentMessageRole::Thinking => {
                let md = AgentMarkdown::new(format!("{}-md", self.id), self.body);
                let header = h_flex()
                    .gap_1()
                    .child(
                        Icon::new(IconName::ToolThink)
                            .size(IconSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new("Thinking")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    );
                v_flex()
                    .id(self.id.clone())
                    .w_full()
                    .gap_1()
                    .child(header)
                    .child(div().opacity(0.7).child(md))
                    .into_any_element()
            }
            AgentMessageRole::Status => h_flex()
                .id(self.id)
                .w_full()
                .gap_1()
                .child(
                    Label::new(self.body)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .into_any_element(),
            AgentMessageRole::ToolCall => tool_call_card(self, cx),
        }
    }
}

impl Component for AgentMessageBubble {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let tool_call =
            AgentMessageBubble::new("m-tool-1", AgentMessageRole::ToolCall, "run_tests")
                .tool_name("run_tests")
                .tool_state(ToolCallState::Success)
                .tool_input("cargo test -p boltz-ui")
                .tool_output("test result: ok. 42 passed; 0 failed")
                .expanded(true);

        let assistant = AgentMessageBubble::new(
            "m-a",
            AgentMessageRole::Assistant,
            "Run `cargo test -p boltz-ui`.",
        );
        let messages = v_flex()
            .gap_2()
            .child(AgentMessageBubble::new(
                "m-user",
                AgentMessageRole::User,
                "How do I run tests?",
            ))
            .child(assistant)
            .child(AgentMessageBubble::new(
                "m-thinking",
                AgentMessageRole::Thinking,
                "Checking...",
            ))
            .child(AgentMessageBubble::new(
                "m-status",
                AgentMessageRole::Status,
                "Connecting…",
            ));

        Some(
            v_flex()
                .w_96()
                .gap_4()
                .child(single_example("Roles", messages.into_any_element()))
                .child(single_example(
                    "Tool Call (expanded)",
                    tool_call.into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
