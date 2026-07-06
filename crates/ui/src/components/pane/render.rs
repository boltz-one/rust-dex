//! [`Render`] impl for [`super::Pane`] — the tab strip (with close/add
//! affordances and drag-to-reorder) plus the active tab's content.

use gpui::Empty;

use super::{Pane, PaneEvent};
use crate::{IconButton, IconName, IconSize, Tab, TabBar, prelude::*};

/// Bare `IconButton::new`'s own default `debug_selector` (`"ICON-{icon:?}"`)
/// collides across every tab sharing the same icon within a `Pane` — this
/// wraps it in a `div` carrying a per-instance selector instead. The wrapper
/// does not intercept clicks; the inner `IconButton` still owns the only
/// click handler (mirrors `ActionPanel`'s Save/Cancel wrapper precedent).
fn debug_wrap(id: impl Into<ElementId>, selector: String, child: IconButton) -> impl IntoElement {
    div().id(id).debug_selector(move || selector).child(child)
}

/// Drag payload used for reordering tabs within a [`Pane`]'s tab strip.
#[derive(Clone, Copy)]
struct TabDragPayload {
    from_idx: usize,
}

impl Render for Pane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_idx = self.active_idx;
        let mut tab_bar = TabBar::new("pane-tab-bar");

        for ix in 0..self.tabs.len() {
            let tab_id = self.tabs[ix].0;
            let title = self.tabs[ix].1.title();
            let selected = ix == active_idx;

            // Per-tab hover group: the close button is hidden until the mouse
            // hovers this specific tab (each tab is its own `group` scope, so
            // hovering one tab reveals only its own close button).
            let hover_group = SharedString::from(format!("pane-tab-{}", tab_id.0));
            // VSCode behavior: the active tab always shows its close button;
            // inactive tabs reveal it only on hover.
            let mut close_ib = IconButton::new(("pane-tab-close", tab_id.0), IconName::Close)
                .icon_size(IconSize::XSmall)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.close_tab(ix, cx);
                }));
            if !selected {
                close_ib = close_ib.visible_on_hover(hover_group.clone());
            }
            let close_button = debug_wrap(
                ("pane-tab-close-wrap", tab_id.0),
                format!("PANE-TAB-CLOSE-{}", tab_id.0),
                close_ib,
            );

            let tab = Tab::new(("pane-tab", tab_id.0))
                .group(hover_group)
                .toggle_state(selected)
                .end_slot(close_button)
                // Wrap in `Label` so the tab title uses the UI font family/size
                // consistently (a bare string child inherits the window's
                // default text size, which mismatches other UI text and can
                // overflow the tab strip's height).
                .child(Label::new(title).size(LabelSize::Small))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.activate(ix, cx);
                }))
                .on_drag(TabDragPayload { from_idx: ix }, |_, _, _, cx| {
                    cx.new(|_| Empty)
                })
                .drag_over::<TabDragPayload>(|style, _, _, _| style.opacity(0.5))
                .on_drop(cx.listener(move |this, payload: &TabDragPayload, _, cx| {
                    this.reorder(payload.from_idx, ix, cx);
                }));

            tab_bar = tab_bar.child(tab);
        }

        tab_bar = tab_bar.end_child(debug_wrap(
            "pane-add-tab-wrap",
            "PANE-ADD-TAB".to_string(),
            IconButton::new("pane-add-tab", IconName::Plus)
                .icon_size(IconSize::XSmall)
                .on_click(cx.listener(|this, _, _, cx| {
                    let content = (this.new_tab_factory)();
                    this.add_tab(content, cx);
                })),
        ));

        // Close-pane "x" at the far right of the header — asks the parent
        // PaneGroup to remove this whole pane (ignored if it is the last one).
        tab_bar = tab_bar.end_child(debug_wrap(
            "pane-close-wrap",
            "PANE-CLOSE".to_string(),
            IconButton::new("pane-close", IconName::Close)
                .icon_size(IconSize::XSmall)
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(PaneEvent::CloseRequested);
                })),
        ));

        let content = self
            .tabs
            .get(active_idx)
            .map(|(_, content)| content.render(true, window, cx));

        v_flex()
            .id("pane")
            .size_full()
            .child(tab_bar)
            .child(div().flex_1().min_h_0().overflow_hidden().children(content))
    }
}
