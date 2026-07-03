use gpui::AnyElement;
use smallvec::SmallVec;

use crate::prelude::*;

/// A fixed max-width centering wrapper, matching Tailwind's `.container`
/// pattern.
///
/// Centering approach: GPUI's `Styled` trait generates `max_w()` and
/// `mx_auto()` from `margin_style_methods!` (see `gpui_macros::styles`,
/// `margin_box_style_prefixes` has `auto_allowed: true` for the `mx` prefix),
/// giving a true CSS-like auto-margin centering primitive — no
/// `justify_center()` fallback is needed here.
#[derive(IntoElement, RegisterComponent)]
pub struct Container {
    max_width: Pixels,
    children: SmallVec<[AnyElement; 2]>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            max_width: px(1024.),
            children: SmallVec::new(),
        }
    }

    /// Sets the max width of the container (defaults to `1024px`).
    pub fn max_width(mut self, max_width: Pixels) -> Self {
        self.max_width = max_width;
        self
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Container {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Container {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .max_w(self.max_width)
            .mx_auto()
            .px_6()
            .children(self.children)
    }
}

impl Component for Container {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some("A fixed max-width wrapper that centers its content horizontally.")
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let boxed = |label: &'static str| {
            div()
                .bg(semantic::surface(cx))
                .border_1()
                .border_color(semantic::border(cx))
                .rounded_md()
                .py_2()
                .child(Label::new(label))
        };
        let frame = |content: AnyElement| {
            div()
                .border_1()
                .border_dashed()
                .border_color(semantic::border_muted(cx))
                .child(content)
        };

        Some(
            v_flex()
                .gap_4()
                .w(px(700.))
                .child(frame(
                    Container::new()
                        .max_width(px(400.))
                        .child(boxed("max_width(400px) — centered"))
                        .into_any_element(),
                ))
                .child(frame(
                    Container::new()
                        .max_width(px(240.))
                        .child(boxed("max_width(240px) — narrower"))
                        .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
