//! [`InspectorOverlay`] — read-only MVP hover highlight + element list panel.
//!
//! Renders a window-filling `div` that tracks the mouse position, resolves
//! the innermost (smallest-area) tracked element under the cursor from the
//! shared [`ElementRegistry`], and draws a bordered highlight rect over its
//! bounds. The side panel (see [`crate::inspector::panel`]) lists every
//! currently visible tracked element.
//!
//! Read-only MVP: no click-through / hit-testing (see ADR 0007, Phase 06).

use gpui::{
    App, Bounds, Context, InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Point,
    Render, Styled as _, Window, div, rgb, rgba,
};

use crate::inspector::panel;
use crate::registry::{ElementRegistry, ElementSnapshot};

/// A read-only overlay: hover-highlights the innermost tracked element under
/// the cursor and lists all currently visible tracked elements in a side
/// panel. Mount this as an absolutely-positioned sibling over your app's
/// content (see `examples/inspector_demo.rs`).
#[derive(Default)]
pub struct InspectorOverlay {
    hover: Option<Point<Pixels>>,
}

impl InspectorOverlay {
    /// Construct an overlay with no hover position recorded yet.
    pub fn new() -> Self {
        Self::default()
    }

    fn area(bounds: &Bounds<Pixels>) -> f32 {
        f32::from(bounds.size.width) * f32::from(bounds.size.height)
    }

    /// The innermost (smallest-area) visible tracked element whose bounds
    /// contain `point`, if any. Heuristic only — real occlusion / z-order is
    /// Phase 06's `hit_test` job.
    fn hovered_snapshot(cx: &App, point: Point<Pixels>) -> Option<ElementSnapshot> {
        cx.try_global::<ElementRegistry>()?
            .all_visible()
            .filter(|s| s.bounds.contains(&point))
            .min_by(|a, b| Self::area(&a.bounds).total_cmp(&Self::area(&b.bounds)))
            .cloned()
    }
}

impl Render for InspectorOverlay {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let hovered = self
            .hover
            .and_then(|point| Self::hovered_snapshot(cx, point));

        let mut root = div().relative().size_full().on_mouse_move(cx.listener(
            |this, event: &gpui::MouseMoveEvent, _window, cx| {
                this.hover = Some(event.position);
                cx.notify();
            },
        ));

        if let Some(snap) = hovered {
            root = root.child(
                div()
                    .absolute()
                    .top(snap.bounds.origin.y)
                    .left(snap.bounds.origin.x)
                    .w(snap.bounds.size.width)
                    .h(snap.bounds.size.height)
                    .border_2()
                    .border_color(rgb(0xff3b30))
                    .bg(rgba(0xff3b3026)),
            );
        }

        root.child(panel::render_panel(cx))
    }
}
