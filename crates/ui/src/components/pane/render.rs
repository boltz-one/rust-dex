//! [`Render`] impl for [`super::Pane`] — the tab strip (with close/add
//! affordances and drag-to-reorder) plus the active tab's content.

use gpui::{Empty, canvas};

use super::{Pane, PaneEvent};
use crate::{
    IconButton, IconButtonShape, IconName, IconSize, Tab, TabBar, TabCloseSide, TabPosition,
    prelude::*,
};

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

        // One-frame-lagged resize hook: the `canvas()` below wrote the
        // content-area bounds during the previous paint. Deliver `on_resize`
        // to the active tab when EITHER the bounds changed OR the active tab
        // changed (keyed by `TabId`) — so a freshly activated/added tab gets an
        // initial size even if the pane itself never physically resized.
        let active_id = self.tabs.get(active_idx).map(|(id, _)| *id);
        if let (Some(bounds), Some(id)) = (self.content_bounds.get(), active_id) {
            if self.notified_resize.get() != Some((id, bounds)) {
                self.notified_resize.set(Some((id, bounds)));
                self.tabs[active_idx].1.on_resize(bounds, cx);
            }
        }

        let mut tab_bar = TabBar::new("pane-tab-bar");

        let tab_count = self.tabs.len();
        for ix in 0..tab_count {
            let tab_id = self.tabs[ix].0;
            let title = self.tabs[ix].1.title();
            // Only the focused pane's active tab is drawn selected, so the
            // whole window shows a single active tab.
            let selected = ix == active_idx && self.focused;
            let position = if ix == 0 {
                TabPosition::First
            } else if ix == tab_count - 1 {
                TabPosition::Last
            } else {
                TabPosition::Middle(ix.cmp(&active_idx))
            };

            // Per-tab hover group: the close button is hidden until the mouse
            // hovers this specific tab (each tab is its own `group` scope, so
            // hovering one tab reveals only its own close button).
            let hover_group = SharedString::from(format!("pane-tab-{}", tab_id.0));
            // Close button is hover-only for every tab (active and inactive
            // alike), per Zed's `pane.rs` tab close-button spec.
            let close_ib = IconButton::new(("pane-tab-close", tab_id.0), IconName::Close)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .shape(IconButtonShape::Square)
                .size(ButtonSize::None)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.close_tab(ix, cx);
                }))
                .visible_on_hover(hover_group.clone());
            let close_button = debug_wrap(
                ("pane-tab-close-wrap", tab_id.0),
                format!("PANE-TAB-CLOSE-{}", tab_id.0),
                close_ib,
            );

            let tab = Tab::new(("pane-tab", tab_id.0))
                .group(hover_group)
                .toggle_state(selected)
                .position(position)
                .close_side(TabCloseSide::End)
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

        // Measure the tab-content area so `on_resize` can fire next frame. The
        // canvas paints nothing; it only records its own bounds (mirrors
        // `TerminalView`'s container measurement).
        let measure = self.content_bounds.clone();

        v_flex().id("pane").size_full().child(tab_bar).child(
            div()
                .relative()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .child(
                    canvas(
                        move |bounds, _, _| measure.set(Some(bounds)),
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .children(content),
        )
    }
}
