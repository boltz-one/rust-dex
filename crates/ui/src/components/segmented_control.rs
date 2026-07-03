use gpui::{AnyElement, ElementId, IntoElement, white};
use std::rc::Rc;

use crate::prelude::*;

/// A horizontal row of connected segments where exactly one is active.
/// Mirrors `RadioButton`'s checked/on_click pattern, rendered as a single
/// connected pill row (visual family of Phase 2's `ButtonGroup`).
///
/// Exclusivity/state is caller-managed: the parent holds the active index
/// and passes it in via `.active(index)`.
#[derive(IntoElement, RegisterComponent)]
pub struct SegmentedControl {
    id: ElementId,
    segments: Vec<SharedString>,
    active: usize,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl SegmentedControl {
    pub fn new(
        id: impl Into<ElementId>,
        segments: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            id: id.into(),
            segments: segments.into_iter().map(Into::into).collect(),
            active: 0,
            disabled: false,
            on_change: None,
        }
    }

    /// Sets the active segment index.
    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Binds a handler called with the clicked segment's index.
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let active = self.active;
        let disabled = self.disabled;
        let group_id = self.id.clone();
        let on_change = self.on_change;

        let mut row = h_flex()
            .id(self.id)
            .rounded_md()
            .border_1()
            .border_color(semantic::border(cx))
            .bg(semantic::surface(cx))
            .p(px(2.))
            .gap_0p5();

        for (i, segment) in self.segments.into_iter().enumerate() {
            let is_active = i == active;
            let on_change = on_change.clone();

            let mut cell = h_flex()
                .id((group_id.clone(), i.to_string()))
                .flex_1()
                .justify_center()
                .px_3()
                .py_1()
                .rounded_sm()
                .when(is_active, |this| this.bg(palette::primary(600)))
                .when(disabled, |this| this.opacity(0.5))
                .child(
                    Label::new(segment)
                        .size(LabelSize::Small)
                        .color(if is_active {
                            Color::Custom(white())
                        } else {
                            Color::Default
                        }),
                );

            if !disabled {
                cell = cell.cursor_pointer().on_click(move |_, window, cx| {
                    if let Some(handler) = &on_change {
                        handler(i, window, cx);
                    }
                });
            }

            row = row.child(cell);
        }

        row
    }
}

impl Component for SegmentedControl {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("A connected row of segments where exactly one is active.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .child(
                    SegmentedControl::new("segmented-default", ["Day", "Week", "Month"])
                        .active(1)
                        .into_any_element(),
                )
                .child(
                    SegmentedControl::new("segmented-disabled", ["List", "Grid"])
                        .disabled(true)
                        .into_any_element(),
                )
                .into_any_element(),
        )
    }
}
