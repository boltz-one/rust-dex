//! N-panel chrome with adjacent-pair drag-resize handles.
//!
//! Deliberately NOT a port of Zed's `workspace` crate (Dock/Pane/PaneGroup,
//! ~48kLOC, coupled to `client::Client`/`project::Project`/LSP capability) —
//! only the *idea* (a pane = header + content, panes tile horizontally) is
//! reused. Implemented as one flat entity (fractions array + N-1 handles)
//! rather than nesting [`crate::ResizablePanelGroup`] entities recursively:
//! nesting would recreate a fresh entity every render (since a nested
//! entity would have to be built inside a per-frame render closure), which
//! resets its drag state on every frame. A flat array has no such bug and
//! reuses the same [`crate::ResizablePanel`]/[`crate::ResizableHandle`]
//! pieces `ResizablePanelGroup` already established.
//!
//! No tab-drag-to-split, no cross-pane drag (dragging a handle only
//! transfers width between its two immediate neighbors), no item
//! persistence — chrome only, matching Phase A's scope.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{AnyElement, Bounds, Context, DragMoveEvent, Empty, Render, canvas};

use crate::{ResizableHandle, ResizablePanel, Tab, TabBar, prelude::*};

/// Minimum width fraction any single pane is allowed to shrink to.
const MIN_PANE_FRACTION: f32 = 0.1;

/// Drag payload for the handle between `handle_ix` and `handle_ix + 1`.
#[derive(Debug)]
struct PaneDrag {
    handle_ix: usize,
    start_x: Pixels,
    start_left_fraction: f32,
    start_right_fraction: f32,
}

/// A single pane: a header row (typically a [`crate::TabBar`]) rendered
/// above a content body. Both are render callbacks, re-invoked each frame
/// like `ResizablePanelGroup`'s `left`/`right`.
pub struct Pane {
    header: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    content: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
}

impl Pane {
    pub fn new(
        header: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            header: Rc::new(header),
            content: Rc::new(content),
        }
    }
}

/// A horizontal group of N [`Pane`]s with a draggable divider between every
/// adjacent pair. Create with `cx.new(|_| PaneGroup::new(panes))`.
pub struct PaneGroup {
    panes: Vec<Pane>,
    /// Width fraction per pane; always sums to `1.0` and has the same
    /// length as `panes`.
    fractions: Vec<f32>,
    container_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl PaneGroup {
    /// Panes start at equal width. Panics if `panes` is empty — a pane
    /// group with zero panes has nothing to render.
    pub fn new(panes: Vec<Pane>) -> Self {
        assert!(!panes.is_empty(), "PaneGroup requires at least one pane");
        let fraction = 1.0 / panes.len() as f32;
        let fractions = vec![fraction; panes.len()];
        Self {
            panes,
            fractions,
            container_bounds: Rc::new(Cell::new(None)),
        }
    }

    fn on_drag_move(&mut self, event: &DragMoveEvent<PaneDrag>, cx: &mut Context<Self>) {
        let drag = event.drag(cx);
        let container_width = self
            .container_bounds
            .get()
            .map(|b| b.size.width)
            .unwrap_or(px(1.));
        if container_width <= px(0.) {
            return;
        }
        let delta_fraction = (event.event.position.x - drag.start_x) / container_width;
        // Clamp so neither neighbor crosses `MIN_PANE_FRACTION`, while
        // keeping their combined width constant (only these two panes'
        // fractions change — no redistribution across the rest of the row).
        let min_delta = -(drag.start_right_fraction - MIN_PANE_FRACTION);
        let max_delta = drag.start_left_fraction - MIN_PANE_FRACTION;
        let clamped_delta = delta_fraction.clamp(min_delta.min(0.), max_delta.max(0.));

        self.fractions[drag.handle_ix] = drag.start_left_fraction + clamped_delta;
        self.fractions[drag.handle_ix + 1] = drag.start_right_fraction - clamped_delta;
        cx.notify();
    }
}

impl Render for PaneGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let container_bounds = self.container_bounds.clone();
        let mouse_x = window.mouse_position().x;
        let pane_count = self.panes.len();

        let mut row = h_flex()
            .id("pane-group")
            .size_full()
            .overflow_hidden()
            .child({
                let measure = container_bounds.clone();
                canvas(
                    move |bounds, _, _| measure.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            });

        for ix in 0..pane_count {
            let pane = &self.panes[ix];
            let header = (pane.header)(window, cx);
            let content = (pane.content)(window, cx);

            row = row.child(
                ResizablePanel::new().fraction(self.fractions[ix]).child(
                    v_flex()
                        .h_full()
                        .child(header)
                        .child(div().flex_1().min_h_0().overflow_hidden().child(content)),
                ),
            );

            if ix + 1 < pane_count {
                let start_left_fraction = self.fractions[ix];
                let start_right_fraction = self.fractions[ix + 1];
                row = row.child(
                    div()
                        .id(("pane-group-handle", ix))
                        .w(px(8.))
                        .h_full()
                        .flex_shrink_0()
                        .cursor_col_resize()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(ResizableHandle)
                        .on_drag(
                            PaneDrag {
                                handle_ix: ix,
                                start_x: mouse_x,
                                start_left_fraction,
                                start_right_fraction,
                            },
                            |_, _, _, cx| cx.new(|_| Empty),
                        )
                        .on_drag_move::<PaneDrag>(cx.listener(|this, event, _, cx| {
                            this.on_drag_move(event, cx);
                        })),
                );
            }
        }

        row
    }
}

/// Gallery catalog entry for [`PaneGroup`].
#[derive(IntoElement, RegisterComponent)]
pub struct PaneGroupPreview;

impl RenderOnce for PaneGroupPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().h(px(280.)).child(cx.new(|_| {
            PaneGroup::new(vec![
                Pane::new(
                    |_, _| {
                        TabBar::new("pane-1-tabs")
                            .child(Tab::new("pane-1-tab-1").toggle_state(true).child("main.rs"))
                            .child(Tab::new("pane-1-tab-2").child("lib.rs"))
                            .into_any_element()
                    },
                    |_, _| {
                        div()
                            .p_4()
                            .child(Label::new("Pane 1 content").color(Color::Muted))
                            .into_any_element()
                    },
                ),
                Pane::new(
                    |_, _| {
                        TabBar::new("pane-2-tabs")
                            .child(
                                Tab::new("pane-2-tab-1")
                                    .toggle_state(true)
                                    .child("README.md"),
                            )
                            .into_any_element()
                    },
                    |_, _| {
                        div()
                            .p_4()
                            .child(Label::new("Pane 2 content").color(Color::Muted))
                            .into_any_element()
                    },
                ),
            ])
        }))
    }
}

impl Component for PaneGroupPreview {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some(
            "N-panel horizontal chrome with a header (e.g. TabBar) per pane and \
             adjacent-pair drag-resize handles.",
        )
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        PaneGroupPreview
            .render(window, cx)
            .into_any_element()
            .into()
    }
}
