use gpui::{AnyElement, FontWeight};
use smallvec::SmallVec;

use crate::prelude::*;

/// A styled keyboard key chip, mirroring shadcn's `<kbd>`.
#[derive(IntoElement, RegisterComponent)]
pub struct Kbd {
    label: SharedString,
}

impl Kbd {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(px(20.))
            .min_w(px(20.))
            .px_1p5()
            .rounded_md()
            .border_1()
            .border_color(semantic::border(cx))
            .bg(semantic::secondary_bg(cx))
            .text_color(semantic::text_muted(cx))
            .child(
                Label::new(self.label)
                    .size(LabelSize::XSmall)
                    .weight(FontWeight::MEDIUM),
            )
    }
}

/// Groups multiple [`Kbd`] chips for shortcut combos (e.g. ⌘ + K).
#[derive(IntoElement)]
pub struct KbdGroup {
    children: SmallVec<[AnyElement; 4]>,
}

impl KbdGroup {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for KbdGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for KbdGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for KbdGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        h_flex().gap_1().items_center().children(self.children)
    }
}

impl Component for Kbd {
    fn scope() -> ComponentScope {
        ComponentScope::Typography
    }

    fn description() -> Option<&'static str> {
        Some("A styled keyboard key chip for displaying shortcuts.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .child(Kbd::new("⌘"))
                .child(Kbd::new("Shift"))
                .child(KbdGroup::new().child(Kbd::new("⌘")).child(Kbd::new("K")))
                .into_any_element(),
        )
    }
}
