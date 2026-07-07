use std::sync::Arc;

use gpui::{ClickEvent, Entity};
use markdown::Markdown;

use crate::prelude::*;
use crate::{AgentMarkdown, Disclosure};

/// Collapsible "Thinking…" block: a chevron + label header, and (when
/// expanded) the thought body rendered as markdown.
///
/// Expansion is entirely caller-owned — mirrors `AgentThreadView`'s
/// `sticky_to_bottom` pattern — so the caller can auto-expand while a turn
/// is still streaming and collapse (or leave user-collapsed) once it
/// finishes, without this component tracking any state itself.
#[derive(IntoElement, RegisterComponent)]
pub struct ThinkingBlock {
    id: ElementId,
    markdown: Entity<Markdown>,
    expanded: bool,
    on_toggle_expanded: Option<Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl ThinkingBlock {
    pub fn new(id: impl Into<ElementId>, markdown: Entity<Markdown>) -> Self {
        Self {
            id: id.into(),
            markdown,
            expanded: false,
            on_toggle_expanded: None,
        }
    }

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

impl RenderOnce for ThinkingBlock {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let expanded = self.expanded;
        let disclosure_id = format!("{}-disclosure", self.id);
        let body_id = format!("{}-md", self.id);

        let header = h_flex()
            .id(self.id.clone())
            .gap_1()
            .child(
                Icon::new(IconName::ToolThink)
                    .size(IconSize::XSmall)
                    .color(Color::Muted),
            )
            .child(
                Label::new("Thinking…")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Disclosure::new(disclosure_id, expanded)
                    .on_toggle_expanded(self.on_toggle_expanded),
            );

        v_flex()
            .w_full()
            .gap_1()
            .child(header)
            .when(expanded, |this| {
                this.child(
                    div()
                        .opacity(0.7)
                        .pl_5()
                        .child(AgentMarkdown::new(body_id, self.markdown)),
                )
            })
    }
}

impl Component for ThinkingBlock {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn description() -> Option<&'static str> {
        Some(
            "A collapsible 'Thinking…' block; expansion is caller-owned so it \
             can auto-expand while streaming and stay put once a turn finishes.",
        )
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let entity = crate::agent_markdown_entity(
            "Checking the test suite before making changes to `agent_message.rs`.",
            cx,
        );

        Some(
            v_flex()
                .w_96()
                .gap_4()
                .child(single_example(
                    "Expanded",
                    ThinkingBlock::new("thinking-preview-1", entity)
                        .expanded(true)
                        .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
