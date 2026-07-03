use gpui::{AnyElement, FontWeight};

use crate::prelude::*;

/// A mid-page section heading: bold title with optional grouped content
/// (e.g. a tab bar or filter row) below it. `mb_4` per Tailwind's Section
/// Heading pattern.
#[derive(IntoElement, RegisterComponent)]
pub struct SectionHeading {
    title: SharedString,
    content: Option<AnyElement>,
}

impl SectionHeading {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            content: None,
        }
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

impl RenderOnce for SectionHeading {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_2()
            .mb_4()
            .child(
                div()
                    .text_size(rems_from_px(24.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(semantic::text(cx))
                    .child(self.title),
            )
            .children(self.content)
    }
}

impl Component for SectionHeading {
    fn scope() -> ComponentScope {
        ComponentScope::Typography
    }

    fn description() -> Option<&'static str> {
        Some("A mid-page section heading with an optional grouped content row below.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            SectionHeading::new("Billing history")
                .content(
                    Label::new("Showing invoices from the last 12 months")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .into_any_element(),
        )
    }
}
