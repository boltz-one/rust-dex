//! Draggable divider between two adjacent siblings of a [`PaneAxis`]:
//! payload, drag-move math (pointer delta -> flex-fraction delta, clamped),
//! and the [`ResizableHandle`] wiring that drives it.

use gpui::{Axis, DragMoveEvent, Empty};

use super::PaneGroup;
use super::tree::axis_at_mut;
use crate::{ResizableHandle, prelude::*};

/// Minimum flex fraction any single pane may shrink to while dragging.
const MIN_PANE_FRACTION: f32 = 0.1;

/// Drag payload for the divider between `handle_ix` and `handle_ix + 1` of
/// the [`super::PaneAxis`] found by walking `path` from the tree root.
#[derive(Clone)]
pub(super) struct DividerDrag {
    pub(super) path: Vec<usize>,
    pub(super) handle_ix: usize,
    pub(super) axis: Axis,
    pub(super) start_pos: Pixels,
    pub(super) start_left_fraction: f32,
    pub(super) start_right_fraction: f32,
}

impl PaneGroup {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_divider(
        &self,
        path: Vec<usize>,
        handle_ix: usize,
        axis: Axis,
        start_left_fraction: f32,
        start_right_fraction: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let mouse_pos = window.mouse_position();
        let start_pos = match axis {
            Axis::Horizontal => mouse_pos.x,
            Axis::Vertical => mouse_pos.y,
        };
        let drag = DividerDrag {
            path: path.clone(),
            handle_ix,
            axis,
            start_pos,
            start_left_fraction,
            start_right_fraction,
        };

        ResizableHandle::new(format!("pane-divider-{path:?}-{handle_ix}"))
            .axis(axis)
            .on_drag(drag, |_, _, _, cx| cx.new(|_| Empty))
            .on_drag_move::<DividerDrag>(cx.listener(|this, event, _, cx| {
                this.on_divider_drag(event, cx);
            }))
            .into_any_element()
    }

    pub(super) fn on_divider_drag(
        &mut self,
        event: &DragMoveEvent<DividerDrag>,
        cx: &mut Context<Self>,
    ) {
        let drag = event.drag(cx);
        let (path, handle_ix, axis, start_pos, start_left, start_right) = (
            drag.path.clone(),
            drag.handle_ix,
            drag.axis,
            drag.start_pos,
            drag.start_left_fraction,
            drag.start_right_fraction,
        );
        let current_pos = match axis {
            Axis::Horizontal => event.event.position.x,
            Axis::Vertical => event.event.position.y,
        };

        let Some(pane_axis) = axis_at_mut(&mut self.root, &path) else {
            return;
        };
        let container_size = pane_axis
            .bounds
            .get()
            .map(|b| match axis {
                Axis::Horizontal => b.size.width,
                Axis::Vertical => b.size.height,
            })
            .unwrap_or(px(1.));
        if container_size <= px(0.) {
            return;
        }

        let delta_fraction = (current_pos - start_pos) / container_size;
        let min_delta = -(start_right - MIN_PANE_FRACTION);
        let max_delta = start_left - MIN_PANE_FRACTION;
        let clamped_delta = delta_fraction.clamp(min_delta.min(0.), max_delta.max(0.));

        pane_axis.flexes[handle_ix] = start_left + clamped_delta;
        pane_axis.flexes[handle_ix + 1] = start_right - clamped_delta;
        cx.notify();
    }
}
