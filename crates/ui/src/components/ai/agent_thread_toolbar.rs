use gpui::SharedString;

use crate::Tooltip;
use crate::prelude::*;

/// Plain, protocol-agnostic aggregate usage figures for an agent thread:
/// cumulative token count and cost, plus an optional itemized `breakdown`
/// (e.g. per-request figures) shown in a tooltip on hover.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UsageDisplay {
    pub tokens: Option<u64>,
    pub cost: Option<SharedString>,
    pub breakdown: Vec<(SharedString, SharedString)>,
}

impl UsageDisplay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tokens(mut self, tokens: u64) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn cost(mut self, cost: impl Into<SharedString>) -> Self {
        self.cost = Some(cost.into());
        self
    }

    /// Adds an itemized `(label, value)` row shown in the hover tooltip
    /// (e.g. `("Input", "8.2k tokens")`, `("Output", "4.2k tokens")`).
    pub fn breakdown_item(
        mut self,
        label: impl Into<SharedString>,
        value: impl Into<SharedString>,
    ) -> Self {
        self.breakdown.push((label.into(), value.into()));
        self
    }

    fn summary_label(&self) -> Option<SharedString> {
        let mut parts = Vec::new();
        if let Some(tokens) = self.tokens {
            parts.push(format_token_count(tokens));
        }
        if let Some(cost) = &self.cost {
            parts.push(cost.to_string());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" · ").into())
        }
    }
}

fn format_token_count(tokens: u64) -> String {
    if tokens >= 1_000 {
        format!("{:.1}k tokens", tokens as f64 / 1000.0)
    } else {
        format!("{tokens} tokens")
    }
}

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
    usage_display: Option<UsageDisplay>,
}

impl AgentThreadToolbar {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            right_slot: None,
            usage: None,
            usage_display: None,
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

    /// A compact token+cost summary (e.g. "12.4k tokens · $0.08") with a
    /// hover tooltip showing the full [`UsageDisplay::breakdown`].
    pub fn usage_summary(mut self, usage: UsageDisplay) -> Self {
        self.usage_display = Some(usage);
        self
    }
}

impl RenderOnce for AgentThreadToolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let toolbar_id = self.id.clone();
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
                    })
                    .when_some(
                        self.usage_display.and_then(|usage| {
                            usage
                                .summary_label()
                                .map(|summary| (summary, usage.breakdown))
                        }),
                        |this, (summary, breakdown)| {
                            this.child(
                                h_flex()
                                    .id((toolbar_id, "usage"))
                                    .gap_1()
                                    .flex_shrink_0()
                                    .child(
                                        Label::new(summary)
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .when(!breakdown.is_empty(), |el| {
                                        el.tooltip(Tooltip::element(move |_, _| {
                                            v_flex()
                                                .gap_1()
                                                .children(breakdown.iter().cloned().map(
                                                    |(label, value)| {
                                                        h_flex()
                                                            .gap_2()
                                                            .justify_between()
                                                            .child(
                                                                Label::new(label)
                                                                    .size(LabelSize::Small)
                                                                    .color(Color::Muted),
                                                            )
                                                            .child(
                                                                Label::new(value)
                                                                    .size(LabelSize::Small),
                                                            )
                                                    },
                                                ))
                                                .into_any_element()
                                        }))
                                    }),
                            )
                        },
                    ),
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
                    "With usage summary + hover breakdown",
                    AgentThreadToolbar::new("toolbar-3", "Fix flaky test")
                        .usage_summary(
                            UsageDisplay::new()
                                .tokens(12_400)
                                .cost("$0.08")
                                .breakdown_item("Input", "8.2k tokens")
                                .breakdown_item("Output", "4.2k tokens")
                                .breakdown_item("Cache read", "1.1k tokens"),
                        )
                        .into_any_element(),
                ))
                .child(single_example(
                    "With right slot",
                    AgentThreadToolbar::new("toolbar-4", "Fix flaky test")
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
