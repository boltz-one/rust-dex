//! [`ResizablePanel`]/[`ResizableHandle`]: the reusable divider-drag
//! primitives shared by the standalone [`group::ResizablePanelGroup`] and by
//! [`crate::PaneGroup`]'s recursive split rendering.
//!
//! Divider drag reuses the pointer-delta + clamp pattern from column resize
//! (`redistributable_columns.rs` / `data_table.rs`).

mod group;

use gpui::{AnyElement, Axis, Div, Stateful};
use smallvec::SmallVec;

pub use group::{ResizablePanelGroup, ResizablePreview};

use crate::prelude::*;

/// A panel inside a [`ResizablePanelGroup`] (or, via [`Self::axis`], a
/// [`crate::PaneGroup`] split).
#[derive(IntoElement)]
pub struct ResizablePanel {
    children: SmallVec<[AnyElement; 2]>,
    fraction: f32,
    axis: Axis,
}

impl ResizablePanel {
    /// Defaults to [`Axis::Horizontal`] — every existing caller
    /// (`ResizablePanelGroup`) keeps its current row-of-panels behavior
    /// unless it opts into [`Self::axis`].
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            fraction: 1.0,
            axis: Axis::Horizontal,
        }
    }

    pub(crate) fn fraction(mut self, fraction: f32) -> Self {
        self.fraction = fraction;
        self
    }

    /// Lays this panel out along `axis`: sized by width (`Horizontal`) or
    /// height (`Vertical`).
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
}

impl Default for ResizablePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ResizablePanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ResizablePanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_horizontal = self.axis == Axis::Horizontal;
        div()
            .when(is_horizontal, |this| this.h_full())
            .when(!is_horizontal, |this| this.w_full())
            .overflow_hidden()
            .when(self.fraction >= 1.0, |this| this.flex_grow())
            .when(self.fraction < 1.0 && is_horizontal, |this| {
                this.flex_shrink_0().w(relative(self.fraction))
            })
            .when(self.fraction < 1.0 && !is_horizontal, |this| {
                this.flex_shrink_0().h(relative(self.fraction))
            })
            .bg(semantic::surface(cx))
            .children(self.children)
    }
}

/// Draggable divider between two [`ResizablePanel`]s: a wide invisible hit
/// area (with the axis-appropriate resize cursor) centered on a thin
/// visible line.
#[derive(IntoElement)]
pub struct ResizableHandle {
    div: Stateful<Div>,
    axis: Axis,
}

impl ResizableHandle {
    /// Defaults to [`Axis::Horizontal`] (vertical line, `col-resize`
    /// cursor) — call [`Self::axis`] for a horizontal divider.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            div: div().id(id),
            axis: Axis::Horizontal,
        }
    }

    /// Orients this divider along `axis`: `Horizontal` renders a vertical
    /// line with a `col-resize` cursor (splits panels side by side);
    /// `Vertical` renders a horizontal line with a `row-resize` cursor
    /// (splits panels top/bottom).
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
}

impl InteractiveElement for ResizableHandle {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.div.interactivity()
    }
}

impl StatefulInteractiveElement for ResizableHandle {}

impl RenderOnce for ResizableHandle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_horizontal = self.axis == Axis::Horizontal;
        let line = div()
            .when(is_horizontal, |this| this.w(px(1.)).h_full())
            .when(!is_horizontal, |this| this.h(px(1.)).w_full())
            .bg(semantic::border(cx));

        self.div
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .when(is_horizontal, |this| {
                this.w(px(8.)).h_full().cursor_col_resize()
            })
            .when(!is_horizontal, |this| {
                this.h(px(8.)).w_full().cursor_row_resize()
            })
            .child(line)
    }
}
