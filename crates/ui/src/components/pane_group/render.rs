//! [`Render`] impl for [`PaneGroup`]: recursive `h_flex`/`v_flex` +
//! [`ResizablePanel`] layout, click-to-focus, the active-pane highlight
//! border, and the `key_context`/`on_action` wiring that makes
//! `pane_actions`'s actions drive this group for free once mounted.
//! Divider drag math lives in [`super::divider`].

use gpui::{AnyElement, Axis, Entity, MouseButton, canvas};

use super::{Member, PaneAxis, PaneGroup, SplitDirection};
use crate::{
    ClosePane, FocusDown, FocusLeft, FocusRight, FocusUp, Pane, ResizablePanel, SplitDown,
    SplitLeft, SplitRight, SplitUp, prelude::*,
};

impl PaneGroup {
    fn render_member(
        &self,
        member: &Member,
        path: Vec<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match member {
            Member::Leaf(pane) => self.render_leaf(pane, cx),
            Member::Split(axis) => self.render_axis(axis, path, window, cx),
        }
    }

    fn render_leaf(&self, pane: &Entity<Pane>, cx: &mut Context<Self>) -> AnyElement {
        let is_active = pane.entity_id() == self.active_pane.entity_id();
        let focus_target = pane.clone();
        let leaf_id = pane.entity_id().as_u64();

        div()
            .id(("pane-leaf", leaf_id))
            // Test-only (no-op in release builds, per `debug_selector`'s own
            // cfg gate): lets tests locate a specific pane leaf's real
            // rendered bounds by its stable entity id, e.g. to assert split
            // quadrants or resize deltas.
            .debug_selector(move || format!("PANE-LEAF-{leaf_id}"))
            .size_full()
            .overflow_hidden()
            .border_2()
            .border_color(if is_active {
                semantic::border_focused(cx)
            } else {
                gpui::transparent_black()
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    this.active_pane = focus_target.clone();
                    window.focus(&this.focus_handle, cx);
                    cx.notify();
                }),
            )
            .child(pane.clone())
            .into_any_element()
    }

    fn render_axis(
        &self,
        pane_axis: &PaneAxis,
        path: Vec<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let axis = pane_axis.axis;
        let mut container = match axis {
            Axis::Horizontal => h_flex(),
            Axis::Vertical => v_flex(),
        }
        .id(format!("pane-axis-{path:?}"))
        .size_full()
        .overflow_hidden();

        let measure = pane_axis.bounds.clone();
        container = container.child(
            canvas(
                move |bounds, _, _| measure.set(Some(bounds)),
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        );

        let n = pane_axis.members.len();
        for (ix, child) in pane_axis.members.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(ix);
            let child_el = self.render_member(child, child_path, window, cx);

            container = container.child(
                ResizablePanel::new()
                    .axis(axis)
                    .fraction(pane_axis.flexes[ix])
                    .child(child_el),
            );

            if ix + 1 < n {
                container = container.child(self.render_divider(
                    path.clone(),
                    ix,
                    axis,
                    pane_axis.flexes[ix],
                    pane_axis.flexes[ix + 1],
                    window,
                    cx,
                ));
            }
        }

        container.into_any_element()
    }
}

impl Render for PaneGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.render_member(&self.root, Vec::new(), window, cx);

        div()
            .id("pane-group")
            .key_context("PaneGroup")
            .track_focus(&self.focus_handle)
            .size_full()
            .on_action(cx.listener(|this, _: &SplitRight, _, cx| {
                this.split(SplitDirection::Right, cx);
            }))
            .on_action(cx.listener(|this, _: &SplitDown, _, cx| {
                this.split(SplitDirection::Down, cx);
            }))
            .on_action(cx.listener(|this, _: &SplitLeft, _, cx| {
                this.split(SplitDirection::Left, cx);
            }))
            .on_action(cx.listener(|this, _: &SplitUp, _, cx| {
                this.split(SplitDirection::Up, cx);
            }))
            .on_action(cx.listener(|this, _: &ClosePane, _, cx| {
                let _ = this.close_active(cx);
            }))
            .on_action(cx.listener(|this, _: &FocusLeft, _, cx| {
                this.focus(SplitDirection::Left, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusRight, _, cx| {
                this.focus(SplitDirection::Right, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusUp, _, cx| {
                this.focus(SplitDirection::Up, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusDown, _, cx| {
                this.focus(SplitDirection::Down, cx);
            }))
            .child(content)
    }
}
