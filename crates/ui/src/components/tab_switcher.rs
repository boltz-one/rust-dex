use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, FocusHandle, Focusable, IntoElement, KeyDownEvent, Pixels, Render,
    Window, div, px, rgb,
};

use crate::{IconName, List, ListItem, overlay_backdrop, overlay_panel, prelude::*};

/// Default panel width for a [`TabSwitcher`] overlay.
pub const TAB_SWITCHER_WIDTH: Pixels = px(420.);

/// A single entry rendered in a [`TabSwitcher`]'s list.
pub struct TabSwitcherItem {
    label: SharedString,
    subtitle: Option<SharedString>,
    icon: Option<IconName>,
    on_select: Rc<dyn Fn(&mut Window, &mut App)>,
}

impl TabSwitcherItem {
    pub fn new(
        label: impl Into<SharedString>,
        on_select: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            subtitle: None,
            icon: None,
            on_select: Rc::new(on_select),
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A Cmd+Tab-style overlay: a plain list of open items (no search query),
/// navigated with ↑/↓ (or Tab/Shift-Tab to cycle) and confirmed with Enter.
/// Esc requests dismissal via [`TabSwitcher::on_dismiss`].
///
/// Deliberately NOT built on [`crate::CommandPalette`]/`PickerDelegate`
/// generic abstraction — with only these two overlay use-cases in the
/// codebase so far, a second bespoke ~150-line file is cheaper than
/// generalizing early (YAGNI). Revisit if a third picker-shaped overlay
/// (file-finder, go-to-line) shows up.
///
/// Caller-owned open/closed state, matching `CommandPalette`/`Modal`/`Drawer`:
/// there is no internal open flag. The caller mounts a `TabSwitcher` into the
/// tree only while it should be visible.
pub struct TabSwitcher {
    items: Vec<TabSwitcherItem>,
    selected_ix: usize,
    focus_handle: FocusHandle,
    on_dismiss: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl TabSwitcher {
    pub fn new(cx: &mut Context<Self>, items: Vec<TabSwitcherItem>) -> Self {
        Self {
            items,
            selected_ix: 0,
            focus_handle: cx.focus_handle(),
            on_dismiss: None,
        }
    }

    /// Registers a callback invoked when the user presses Esc, requesting
    /// that the caller unmount/close this switcher.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// Moves keyboard focus onto the switcher so Tab/↑/↓/Enter/Esc work
    /// immediately. Callers should invoke this once after mounting.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus_handle, cx);
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        self.selected_ix = (self.selected_ix + 1) % self.items.len();
        cx.notify();
    }

    fn select_previous(&mut self, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        self.selected_ix = if self.selected_ix == 0 {
            self.items.len() - 1
        } else {
            self.selected_ix - 1
        };
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let shift = event.keystroke.modifiers.shift;
        match event.keystroke.key.as_str() {
            "down" => self.select_next(cx),
            "tab" if !shift => self.select_next(cx),
            "up" => self.select_previous(cx),
            "tab" if shift => self.select_previous(cx),
            "enter" => {
                if let Some(item) = self.items.get(self.selected_ix) {
                    let on_select = item.on_select.clone();
                    on_select(window, cx);
                }
            }
            "escape" => {
                if let Some(on_dismiss) = self.on_dismiss.clone() {
                    on_dismiss(window, cx);
                }
            }
            _ => {}
        }
    }

    fn render_row(&self, item_ix: usize, is_selected: bool) -> AnyElement {
        let item = &self.items[item_ix];
        let on_select = item.on_select.clone();

        ListItem::new(("tab-switcher-item", item_ix))
            .focused(is_selected)
            .when_some(item.icon, |this, icon| this.start_slot(Icon::new(icon)))
            .child(
                v_flex()
                    .gap_0p5()
                    .child(Label::new(item.label.clone()))
                    .when_some(item.subtitle.clone(), |this, subtitle| {
                        this.child(
                            Label::new(subtitle)
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                    }),
            )
            .on_click(move |_, window, cx| on_select(window, cx))
            .into_any_element()
    }
}

impl Focusable for TabSwitcher {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TabSwitcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.selected_ix >= self.items.len() {
            self.selected_ix = 0;
        }
        let selected_ix = self.selected_ix;

        let rows: Vec<AnyElement> = (0..self.items.len())
            .map(|item_ix| self.render_row(item_ix, item_ix == selected_ix))
            .collect();

        overlay_backdrop()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(
                overlay_panel()
                    .w(TAB_SWITCHER_WIDTH)
                    .max_h(px(360.))
                    .child(
                        div()
                            .px(px(20.))
                            .py(px(12.))
                            .border_b_1()
                            .border_color(rgb(0x2A313B))
                            .child(
                                Label::new("Switch Tab")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(
                        div()
                            .id("tab-switcher-results")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(List::new().empty_message("No open tabs").children(rows)),
                    ),
            )
    }
}

/// Standalone gallery preview for `TabSwitcher` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching
/// `CodeEditor`/`SearchInput`'s existing convention in this crate).
pub fn tab_switcher_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    // `.relative()` + a fixed height confines the switcher's `.absolute()
    // .inset_0()` backdrop to this preview box instead of the nearest
    // positioned ancestor (which could be the whole gallery content pane).
    div()
        .relative()
        .h(px(400.))
        .child(cx.new(|cx| {
            TabSwitcher::new(
                cx,
                vec![
                    TabSwitcherItem::new("main.rs", |_, _| {}).icon(IconName::File),
                    TabSwitcherItem::new("lib.rs", |_, _| {})
                        .icon(IconName::File)
                        .subtitle("crates/ui/src"),
                    TabSwitcherItem::new("Cargo.toml", |_, _| {}).icon(IconName::File),
                ],
            )
        }))
        .into_any_element()
}
