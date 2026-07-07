use gpui::SharedString;

use crate::prelude::*;

/// Toolbar shown atop an agent chat thread: a title, an optional right-hand
/// slot (typically an [`AgentModelSelector`](super::AgentModelSelector)), and
/// optional usage/cost figures.
///
/// `usage` is a list of `(label, value)` pairs (e.g. `("Tokens", "12.4k")`,
/// `("Cost", "$0.08")`) supplied by the caller. This is intentionally decoupled
/// from any `boltz-acpx` usage/cost type — `ui` must not depend on `acpx`.
#[derive(IntoElement, RegisterComponent)]
pub struct AgentThreadToolbar {
    id: ElementId,
    title: SharedString,
    right_slot: Option<AnyElement>,
    usage: Option<Vec<(String, String)>>,
}

impl AgentThreadToolbar {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            right_slot: None,
            usage: None,
        }
    }

    /// Right-hand slot, typically a model selector.
    pub fn right_slot(mut self, element: impl IntoElement) -> Self {
        self.right_slot = Some(element.into_any_element());
        self
    }

    /// Usage/cost figures as `(label, value)` pairs, e.g.
    /// `[("Tokens", "12.4k"), ("Cost", "$0.08")]`.
    pub fn usage(mut self, usage: Vec<(String, String)>) -> Self {
        self.usage = Some(usage);
        self
    }
}

impl RenderOnce for AgentThreadToolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .id(self.id)
            .w_full()
            .justify_between()
            .items_center()
            .gap_2()
            .px_2()
            .py_1p5()
            .border_b_1()
            .border_color(cx.theme().colors().border_variant)
            .child(
                h_flex()
                    .gap_3()
                    .min_w_0()
                    .child(Label::new(self.title).truncate())
                    .when_some(self.usage, |this, usage| {
                        this.children(usage.into_iter().map(|(label, value)| {
                            h_flex()
                                .gap_1()
                                .flex_shrink_0()
                                .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
                                .child(Label::new(value).size(LabelSize::Small))
                        }))
                    }),
            )
            .when_some(self.right_slot, |this, slot| this.child(slot))
    }
}

impl Component for AgentThreadToolbar {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn description() -> Option<&'static str> {
        Some(
            "The toolbar shown atop an agent chat thread: title, a right-hand \
             slot for a model selector, and optional usage/cost figures.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .w_96()
                .gap_4()
                .child(single_example(
                    "Title only",
                    AgentThreadToolbar::new("toolbar-1", "New Thread").into_any_element(),
                ))
                .child(single_example(
                    "With usage",
                    AgentThreadToolbar::new("toolbar-2", "Refactor auth module")
                        .usage(vec![
                            ("Tokens".to_string(), "12.4k".to_string()),
                            ("Cost".to_string(), "$0.08".to_string()),
                        ])
                        .into_any_element(),
                ))
                .child(single_example(
                    "With right slot",
                    AgentThreadToolbar::new("toolbar-3", "Fix flaky test")
                        .right_slot(
                            Button::new("toolbar-model", "claude-sonnet-4.6")
                                .style(ButtonStyle::Subtle),
                        )
                        .usage(vec![("Cost".to_string(), "$0.02".to_string())])
                        .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
