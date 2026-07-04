use gpui::AnyElement;

use crate::prelude::*;

/// A layout wrapper that preserves a fixed width-to-height ratio.
#[derive(IntoElement, RegisterComponent)]
pub struct AspectRatio {
    ratio: f32,
    child: AnyElement,
}

impl AspectRatio {
    /// Creates a wrapper with the given aspect ratio (width / height).
    ///
    /// For example, `16.0 / 9.0` for widescreen media.
    pub fn new(ratio: f32, child: impl IntoElement) -> Self {
        Self {
            ratio,
            child: child.into_any_element(),
        }
    }

    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio;
        self
    }
}

impl RenderOnce for AspectRatio {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id("aspect-ratio")
            .w_full()
            .aspect_ratio(self.ratio)
            .overflow_hidden()
            .child(self.child)
    }
}

impl Component for AspectRatio {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some("A layout wrapper that preserves a fixed width-to-height ratio.")
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .w(px(320.))
                .child(
                    AspectRatio::new(
                        16.0 / 9.0,
                        div()
                            .w_full()
                            .h_full()
                            .bg(semantic::muted_bg(cx))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Label::new("16:9").color(Color::Muted)),
                    )
                    .into_any_element(),
                )
                .child(
                    AspectRatio::new(
                        1.0,
                        div()
                            .w_full()
                            .h_full()
                            .bg(semantic::secondary_bg(cx))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Label::new("1:1").color(Color::Muted)),
                    )
                    .into_any_element(),
                )
                .into_any_element(),
        )
    }
}
