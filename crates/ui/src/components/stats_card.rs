use gpui::{AnyElement, FontWeight};

use crate::prelude::*;

// Note: Tailwind's "Grid List" category (card-based grid) is covered by
// composing existing `Card`s in a caller-owned `h_flex()`/wrap layout — no
// dedicated component is needed for it.

/// Direction of a [`StatsCard`] trend indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsTrend {
    Up,
    Down,
}

/// A `Card`-based metric tile: big number, label, and optional trend indicator.
/// Callers compose multiple `StatsCard`s into a grid.
#[derive(IntoElement, RegisterComponent)]
pub struct StatsCard {
    label: SharedString,
    value: SharedString,
    trend: Option<(StatsTrend, SharedString)>,
}

impl StatsCard {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            trend: None,
        }
    }

    pub fn trend(mut self, trend: StatsTrend, delta: impl Into<SharedString>) -> Self {
        self.trend = Some((trend, delta.into()));
        self
    }
}

impl RenderOnce for StatsCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        Card::new().child(
            v_flex()
                .gap_2()
                .child(
                    Label::new(self.label)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .child(
                    h_flex()
                        .items_end()
                        .justify_between()
                        .child(
                            div()
                                .text_size(rems(1.75))
                                .font_weight(FontWeight::BOLD)
                                .text_color(semantic::text(cx))
                                .child(self.value),
                        )
                        .when_some(self.trend, |this, (trend, delta)| {
                            let (icon, color) = match trend {
                                StatsTrend::Up => (IconName::ArrowUp, palette::success(600)),
                                StatsTrend::Down => (IconName::ArrowDown, palette::danger(600)),
                            };
                            this.child(
                                h_flex()
                                    .gap_1()
                                    .items_center()
                                    .child(
                                        Icon::new(icon)
                                            .size(IconSize::XSmall)
                                            .color(Color::Custom(color)),
                                    )
                                    .child(
                                        Label::new(delta)
                                            .size(LabelSize::Small)
                                            .color(Color::Custom(color)),
                                    ),
                            )
                        }),
                ),
        )
    }
}

impl Component for StatsCard {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> Option<&'static str> {
        Some("A metric tile built on `Card`, with a large number, label, and optional trend.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            h_flex()
                .gap_4()
                .child(StatsCard::new("Total Subscribers", "71,897"))
                .child(StatsCard::new("Avg. Open Rate", "58.16%").trend(StatsTrend::Up, "4.05%"))
                .child(StatsCard::new("Avg. Click Rate", "24.57%").trend(StatsTrend::Down, "1.39%"))
                .into_any_element(),
        )
    }
}
