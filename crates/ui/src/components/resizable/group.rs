//! Standalone two-panel [`ResizablePanelGroup`] built on [`super::ResizablePanel`]/
//! [`super::ResizableHandle`] — unrelated to [`crate::PaneGroup`]'s recursive
//! tree, kept for existing simple left/right split use cases.

use std::rc::Rc;

use gpui::{AnyElement, Bounds, Context, DragMoveEvent, Empty, Render, canvas};

use super::{ResizableHandle, ResizablePanel};
use crate::prelude::*;

/// Drag payload for [`ResizableHandle`].
#[derive(Debug)]
struct ResizableDrag {
    start_fraction: f32,
    start_x: Pixels,
}

fn clamp_fraction(fraction: f32, min: f32, max: f32) -> f32 {
    fraction.clamp(min, max)
}

/// Horizontal panel group with a draggable split handle.
///
/// Create with `cx.new(|_| ResizablePanelGroup::new(left, right))` where
/// `left`/`right` are render callbacks returning panel content each frame.
pub struct ResizablePanelGroup {
    left: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    right: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    left_fraction: f32,
    min_left_fraction: f32,
    max_left_fraction: f32,
    container_bounds: Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
}

impl ResizablePanelGroup {
    pub fn new(
        left: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        right: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            left: Rc::new(left),
            right: Rc::new(right),
            left_fraction: 0.5,
            min_left_fraction: 0.2,
            max_left_fraction: 0.8,
            container_bounds: Rc::new(std::cell::Cell::new(None)),
        }
    }

    pub fn left_fraction(mut self, fraction: f32) -> Self {
        self.left_fraction =
            clamp_fraction(fraction, self.min_left_fraction, self.max_left_fraction);
        self
    }

    pub fn min_left_fraction(mut self, min: f32) -> Self {
        self.min_left_fraction = min.clamp(0.05, 0.95);
        self
    }

    pub fn max_left_fraction(mut self, max: f32) -> Self {
        self.max_left_fraction = max.clamp(self.min_left_fraction, 0.95);
        self
    }

    pub fn left_fraction_value(&self) -> f32 {
        self.left_fraction
    }

    fn on_drag_move(&mut self, event: &DragMoveEvent<ResizableDrag>, cx: &mut Context<Self>) {
        let drag = event.drag(cx);
        let container_width = self
            .container_bounds
            .get()
            .map(|b| b.size.width)
            .unwrap_or(px(1.));
        if container_width <= px(0.) {
            return;
        }
        let delta = event.event.position.x - drag.start_x;
        let delta_fraction = delta / container_width;
        self.left_fraction = clamp_fraction(
            drag.start_fraction + delta_fraction,
            self.min_left_fraction,
            self.max_left_fraction,
        );
        cx.notify();
    }
}

impl Render for ResizablePanelGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let left_fraction = self.left_fraction;
        let right_fraction = 1.0 - left_fraction;
        let left = (self.left)(window, cx);
        let right = (self.right)(window, cx);
        let container_bounds = self.container_bounds.clone();
        let mouse_x = window.mouse_position().x;

        let handle = ResizableHandle::new("resizable-handle-hit")
            .on_drag(
                ResizableDrag {
                    start_fraction: left_fraction,
                    start_x: mouse_x,
                },
                |_, _, _, cx| cx.new(|_| Empty),
            )
            .on_drag_move::<ResizableDrag>(cx.listener(|this, event, _, cx| {
                this.on_drag_move(event, cx);
            }));

        h_flex()
            .id("resizable-panel-group")
            .w_full()
            .h(px(240.))
            .rounded_md()
            .border_1()
            .border_color(semantic::border(cx))
            .overflow_hidden()
            .child({
                let measure = container_bounds.clone();
                canvas(
                    move |bounds, _, _| measure.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            })
            .child(
                ResizablePanel::new()
                    .fraction(left_fraction)
                    .child(div().h_full().p_4().child(left)),
            )
            .child(handle)
            .child(
                ResizablePanel::new()
                    .fraction(right_fraction)
                    .child(div().h_full().p_4().child(right)),
            )
    }
}

/// Gallery catalog entry for [`ResizablePanelGroup`].
#[derive(IntoElement, RegisterComponent)]
pub struct ResizablePreview;

impl RenderOnce for ResizablePreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        cx.new(|_| {
            ResizablePanelGroup::new(
                |_, _| {
                    v_flex()
                        .gap_2()
                        .child(Label::new("Left panel").weight(gpui::FontWeight::SEMIBOLD))
                        .child(Label::new("Drag the handle to resize.").color(Color::Muted))
                        .into_any_element()
                },
                |_, _| {
                    v_flex()
                        .gap_2()
                        .child(Label::new("Right panel").weight(gpui::FontWeight::SEMIBOLD))
                        .child(Label::new("Clamped between 20% and 80%.").color(Color::Muted))
                        .into_any_element()
                },
            )
            .min_left_fraction(0.2)
            .max_left_fraction(0.8)
        })
    }
}

impl Component for ResizablePreview {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some("Horizontal split panels with a draggable divider and min/max clamping.")
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        ResizablePreview
            .render(window, cx)
            .into_any_element()
            .into()
    }
}
