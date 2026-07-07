use std::sync::Arc;

use gpui::{ClickEvent, Entity, SharedString};
use markdown::Markdown;

use crate::prelude::*;
use crate::{
    AgentMarkdown, BadgeColor, BadgeVariant, Card, CardVariant, DiffBlock, Disclosure,
    TerminalOutputBlock, ThinkingBlock,
};

/// Who/what produced a message: `User`/`Status` render as plain text,
/// `Assistant` through [`AgentMarkdown`], `Thinking` through
/// [`ThinkingBlock`], `ToolCall` as a collapsible card.
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

/// One entry of a tool call's collapsible body. Boltz-ui-local mirror of the
/// terminal crate's tool-call content shapes — intentionally has no
/// dependency on any acpx/schema type.
#[derive(Clone)]
pub enum ToolCallContentDisplay {
    /// Markdown text content, e.g. an explanation or a file excerpt.
    Text(Entity<Markdown>),
    /// A unified old/new diff, rendered via [`DiffBlock`] (Decision #2a: no
    /// buffer-aware inline-editable diff view).
    Diff {
        old_text: SharedString,
        new_text: SharedString,
        /// File extension (no leading dot, e.g. `"rs"`) for syntax
        /// highlighting; `None` renders as plain text.
        language: Option<SharedString>,
    },
    /// Captured terminal output, rendered via [`TerminalOutputBlock`]
    /// (Decision #2b: static text, no PTY grid).
    Terminal {
        command: Option<SharedString>,
        raw_output: SharedString,
    },
}

/// A single chat message. Pure builder — all state is caller-owned.
#[derive(IntoElement, RegisterComponent)]
pub struct AgentMessageBubble {
    id: ElementId,
    role: AgentMessageRole,
    body: SharedString,
    /// Rendered body for `Assistant`/`Thinking` roles. Falls back to a plain
    /// `body` label when unset (e.g. a caller not yet passing a persistent
    /// `Entity<Markdown>` — see [`AgentMarkdown`]'s docs for why one is
    /// needed for markdown to actually render).
    markdown_body: Option<Entity<Markdown>>,
    tool_name: Option<SharedString>,
    tool_state: ToolCallState,
    content: Vec<ToolCallContentDisplay>,
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
            markdown_body: None,
            tool_name: None,
            tool_state: ToolCallState::default(),
            content: Vec::new(),
            expanded: false,
            on_toggle_expanded: None,
        }
    }

    /// Sets the rendered markdown body for `Assistant`/`Thinking` roles. See
    /// [`AgentMarkdown`]'s docs on why this takes a persistent `Entity`
    /// rather than raw text.
    pub fn markdown_body(mut self, markdown: Entity<Markdown>) -> Self {
        self.markdown_body = Some(markdown);
        self
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

    /// The tool call's collapsible body content (text/diff/terminal entries,
    /// rendered in order).
    pub fn content(mut self, content: Vec<ToolCallContentDisplay>) -> Self {
        self.content = content;
        self
    }

    /// Whether the tool-call body is expanded (caller-owned).
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

fn render_tool_call_content(
    bubble_id: &ElementId,
    content: Vec<ToolCallContentDisplay>,
) -> impl IntoElement {
    v_flex()
        .gap_2()
        .children(content.into_iter().enumerate().map(|(index, entry)| {
            let item_id = format!("{bubble_id}-content-{index}");
            match entry {
                ToolCallContentDisplay::Text(markdown) => {
                    AgentMarkdown::new(item_id, markdown).into_any_element()
                }
                ToolCallContentDisplay::Diff {
                    old_text,
                    new_text,
                    language,
                } => {
                    let diff = DiffBlock::new(item_id, old_text, new_text);
                    match language {
                        Some(language) => diff.language(language).into_any_element(),
                        None => diff.into_any_element(),
                    }
                }
                ToolCallContentDisplay::Terminal {
                    command,
                    raw_output,
                } => {
                    let terminal = TerminalOutputBlock::new(item_id, raw_output);
                    match command {
                        Some(command) => terminal.command(command).into_any_element(),
                        None => terminal.into_any_element(),
                    }
                }
            }
        }))
}

fn tool_call_card(bubble: AgentMessageBubble, _cx: &App) -> AnyElement {
    let (badge_label, badge_color) = match bubble.tool_state {
        ToolCallState::Running => ("Running", BadgeColor::Secondary),
        ToolCallState::Success => ("Success", BadgeColor::Success),
        ToolCallState::Failed => ("Failed", BadgeColor::Danger),
    };
    let disclosure_id = format!("{}-disclosure", bubble.id);
    let title = bubble.tool_name.unwrap_or_else(|| bubble.body.clone());
    let has_content = !bubble.content.is_empty();
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
        .id(bubble.id.clone())
        .w_full()
        .justify_between()
        .child(h_flex().gap_2().child(tool_icon).child(Label::new(title)))
        .child(
            h_flex()
                .gap_2()
                .child(badge)
                .when(has_content, |this| this.child(disclosure)),
        );

    Card::new()
        .variant(CardVariant::Bordered)
        .header(header)
        .when(has_content && expanded, |this| {
            this.child(render_tool_call_content(&bubble.id, bubble.content))
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
                let body: AnyElement = match self.markdown_body {
                    Some(markdown) => {
                        AgentMarkdown::new(format!("{}-md", self.id), markdown).into_any_element()
                    }
                    None => Label::new(self.body).into_any_element(),
                };
                div()
                    .id(self.id.clone())
                    .w_full()
                    .child(body)
                    .into_any_element()
            }
            AgentMessageRole::Thinking => {
                let expanded = self.expanded;
                let toggle = self.on_toggle_expanded;
                let content: AnyElement = match self.markdown_body {
                    Some(markdown) => ThinkingBlock::new(self.id.clone(), markdown)
                        .expanded(expanded)
                        .when_some(toggle, |block, handler| {
                            block.on_toggle_expanded(move |event, window, cx| {
                                handler(event, window, cx)
                            })
                        })
                        .into_any_element(),
                    None => h_flex()
                        .id(self.id.clone())
                        .gap_1()
                        .child(
                            Icon::new(IconName::ToolThink)
                                .size(IconSize::XSmall)
                                .color(Color::Muted),
                        )
                        .child(
                            Label::new(self.body)
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .into_any_element(),
                };
                div().w_full().child(content).into_any_element()
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

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let assistant_markdown = crate::agent_markdown_entity(
            "Run `cargo test -p boltz-ui`.\n\n| Step | Result |\n|---|---|\n| build | ok |",
            cx,
        );
        let thinking_markdown =
            crate::agent_markdown_entity("Checking the test suite before making changes.", cx);

        let tool_call =
            AgentMessageBubble::new("m-tool-1", AgentMessageRole::ToolCall, "run_tests")
                .tool_name("run_tests")
                .tool_state(ToolCallState::Success)
                .content(vec![
                    ToolCallContentDisplay::Terminal {
                        command: Some("cargo test -p boltz-ui".into()),
                        raw_output: "test result: ok. 42 passed; 0 failed".into(),
                    },
                    ToolCallContentDisplay::Diff {
                        old_text: "fn old() {}".into(),
                        new_text: "fn new() {}".into(),
                        language: Some("rs".into()),
                    },
                ])
                .expanded(true);

        let assistant = AgentMessageBubble::new("m-a", AgentMessageRole::Assistant, "")
            .markdown_body(assistant_markdown);

        let thinking =
            AgentMessageBubble::new("m-thinking", AgentMessageRole::Thinking, "Checking...")
                .markdown_body(thinking_markdown)
                .expanded(true);

        let messages = v_flex()
            .gap_2()
            .child(AgentMessageBubble::new(
                "m-user",
                AgentMessageRole::User,
                "How do I run tests?",
            ))
            .child(assistant)
            .child(thinking)
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
