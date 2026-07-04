use std::rc::Rc;

use gpui::{
    AnyElement, App, BoxShadow, Context, Entity, Focusable, IntoElement, KeyDownEvent, Pixels,
    Render, Window, black, div, point, px, rgb,
};

use crate::{IconName, List, ListItem, TextInput, prelude::*};

use super::fuzzy::score;

/// Default panel width for a [`CommandPalette`] (matches the mockup's
/// 600px-wide overlay).
pub const COMMAND_PALETTE_WIDTH: Pixels = px(600.);

/// A single entry rendered in a [`CommandPalette`]'s results list.
pub struct CommandItem {
    label: SharedString,
    subtitle: Option<SharedString>,
    icon: Option<IconName>,
    keybinding: Option<SharedString>,
    on_select: Rc<dyn Fn(&mut Window, &mut App)>,
}

impl CommandItem {
    pub fn new(
        label: impl Into<SharedString>,
        on_select: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            subtitle: None,
            icon: None,
            keybinding: None,
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

    pub fn keybinding(mut self, keybinding: impl Into<SharedString>) -> Self {
        self.keybinding = Some(keybinding.into());
        self
    }
}

/// A searchable, keyboard-driven command overlay: a text input with a live
/// fuzzy-filtered list of [`CommandItem`]s. ↑/↓ move the highlighted row,
/// Enter runs it, Esc requests dismissal via [`CommandPalette::on_dismiss`].
///
/// `crate::Modal` intentionally renders no backdrop/centering of its own
/// (callers supply that, same as `crate::Drawer`'s convention) — so
/// `CommandPalette` supplies its own backdrop + centered panel wrapper here
/// rather than duplicating logic `Modal` doesn't have. The panel needs exact,
/// non-theme-driven mockup colors (`#12161C`/`#2A313B`/etc.), which is why it
/// doesn't call `Modal::new()` directly (that would pull in `Modal`'s
/// theme-driven `bg`/`border`/`radius`, which can't be overridden from
/// outside); it still reuses the same header/list conventions.
///
/// Caller-owned open/closed state: like `Modal`/`Drawer`, there is no
/// internal open flag. The caller mounts a `CommandPalette` into the tree
/// only while it should be visible, and reacts to `on_dismiss` (Esc) to
/// unmount it again.
pub struct CommandPalette {
    query_input: Entity<TextInput>,
    items: Vec<CommandItem>,
    selected_ix: usize,
    on_dismiss: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl CommandPalette {
    pub fn new(cx: &mut Context<Self>, items: Vec<CommandItem>) -> Self {
        let query_input =
            cx.new(|cx| TextInput::new(cx).placeholder("Type a command or search…"));

        cx.observe(&query_input, |this, _, cx| {
            this.selected_ix = 0;
            cx.notify();
        })
        .detach();

        Self {
            query_input,
            items,
            selected_ix: 0,
            on_dismiss: None,
        }
    }

    /// Registers a callback invoked when the user presses Esc, requesting
    /// that the caller unmount/close this palette.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// Moves keyboard focus into the query input. Callers should invoke this
    /// once after mounting the palette so typing works immediately.
    pub fn focus_input(&self, window: &mut Window, cx: &mut App) {
        let focus_handle = self.query_input.read(cx).focus_handle(cx);
        window.focus(&focus_handle, cx);
    }

    /// Indices into `self.items`, fuzzy-scored against the current query and
    /// sorted best-match-first. An empty query yields every item in its
    /// original order (unfiltered default state).
    fn matched_indices(&self, cx: &App) -> Vec<usize> {
        let query = self.query_input.read(cx).text();
        let mut matches: Vec<(usize, i32)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(ix, item)| score(query, item.label.as_ref()).map(|s| (ix, s)))
            .collect();
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.into_iter().map(|(ix, _)| ix).collect()
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let matched = self.matched_indices(cx);

        match event.keystroke.key.as_str() {
            "up" if !matched.is_empty() => {
                self.selected_ix = if self.selected_ix == 0 {
                    matched.len() - 1
                } else {
                    self.selected_ix - 1
                };
                cx.notify();
            }
            "down" if !matched.is_empty() => {
                self.selected_ix = (self.selected_ix + 1) % matched.len();
                cx.notify();
            }
            "enter" => {
                if let Some(item_ix) = matched.get(self.selected_ix).copied() {
                    let on_select = self.items[item_ix].on_select.clone();
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

        ListItem::new(("command-palette-item", item_ix))
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
            .when_some(item.keybinding.clone(), |this, keybinding| {
                this.end_slot(kbd_badge(keybinding))
            })
            .on_click(move |_, window, cx| on_select(window, cx))
            .into_any_element()
    }

    fn render_query_row(&self) -> impl IntoElement {
        h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .gap_3()
            .px(px(20.))
            .py(px(16.))
            .border_b_1()
            .border_color(rgb(0x2A313B))
            .child(
                div()
                    .flex_1()
                    .text_size(px(15.))
                    .child(self.query_input.clone()),
            )
            .child(kbd_badge("ESC"))
    }
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let matched = self.matched_indices(cx);
        if self.selected_ix >= matched.len() {
            self.selected_ix = 0;
        }
        let selected_ix = self.selected_ix;

        let rows: Vec<AnyElement> = matched
            .iter()
            .enumerate()
            .map(|(row_ix, item_ix)| self.render_row(*item_ix, row_ix == selected_ix))
            .collect();

        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(black().opacity(0.5))
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(
                v_flex()
                    .w(COMMAND_PALETTE_WIDTH)
                    .max_w(vw(0.9, window))
                    .max_h(px(420.))
                    .bg(rgb(0x12161C))
                    .border_1()
                    .border_color(rgb(0x2A313B))
                    .rounded(px(14.))
                    .shadow(vec![BoxShadow {
                        color: black().opacity(0.6),
                        offset: point(px(0.), px(24.)),
                        blur_radius: px(70.),
                        spread_radius: px(0.),
                    }])
                    .overflow_hidden()
                    .child(self.render_query_row())
                    .child(
                        div()
                            .id("command-palette-results")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(
                                List::new()
                                    .empty_message("No matching commands")
                                    .children(rows),
                            ),
                    ),
            )
    }
}

fn kbd_badge(label: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .px(px(6.))
        .h(px(20.))
        .rounded(px(4.))
        .bg(rgb(0x1B212A))
        .border_1()
        .border_color(rgb(0x2A313B))
        .child(
            Label::new(label.into())
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
}
