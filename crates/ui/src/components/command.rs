use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, Bounds, Context, Entity, KeyDownEvent, Pixels, Render, anchored, canvas, deferred,
    point,
};

use crate::utils::fuzzy_subsequence_score;
use crate::{ListHeader, TextInput, prelude::*};

/// A command-palette item: either a non-selectable group label or a selectable entry.
#[derive(Clone)]
pub enum CommandItem {
    Group(SharedString),
    Entry {
        id: SharedString,
        label: SharedString,
    },
}

/// A command palette built on the Combobox input+list pattern with subsequence
/// fuzzy filtering and keyboard navigation (up/down/enter, clamped at ends).
///
/// Stateful view — create with `cx.new(|cx| Command::new(cx, items))`.
#[derive(RegisterComponent)]
pub struct Command {
    items: Vec<CommandItem>,
    input: Entity<TextInput>,
    selected: usize,
    open: bool,
    last_query: String,
    trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl Command {
    pub fn new(cx: &mut Context<Self>, items: Vec<CommandItem>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx).placeholder("Type a command or search…"));
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self {
            items,
            input,
            selected: 0,
            open: true,
            last_query: String::new(),
            trigger_bounds: Rc::new(Cell::new(None)),
        }
    }

    fn filtered_indices(&self, cx: &App) -> Vec<usize> {
        let query = self.input.read(cx).text();
        let mut matches: Vec<(usize, usize)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| match item {
                CommandItem::Group(_) => None,
                CommandItem::Entry { label, .. } => {
                    fuzzy_subsequence_score(query, label).map(|score| (i, score))
                }
            })
            .collect();
        matches.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        matches.into_iter().map(|(i, _)| i).collect()
    }

    fn selectable_count(&self, cx: &App) -> usize {
        self.filtered_indices(cx).len()
    }

    fn clamp_selected(&mut self, cx: &App) {
        let count = self.selectable_count(cx);
        if count == 0 {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(count - 1);
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let count = self.selectable_count(cx);
        if count == 0 {
            return;
        }
        match event.keystroke.key.as_str() {
            "up" => {
                self.selected = self.selected.saturating_sub(1);
                cx.notify();
            }
            "down" => {
                self.selected = (self.selected + 1).min(count - 1);
                cx.notify();
            }
            "enter" => {
                // Selection handled by caller via `selected_entry`.
                cx.notify();
            }
            _ => {}
        }
    }

    /// The currently highlighted entry, if any.
    pub fn selected_entry(&self, cx: &App) -> Option<&CommandItem> {
        self.filtered_indices(cx)
            .get(self.selected)
            .and_then(|&i| self.items.get(i))
    }
}

impl Render for Command {
    // NOTE: the trigger-measurement (`canvas` + `trigger_bounds` +
    // `anchored().snap_to_window_with_margin`) and floating-list
    // (`deferred(..).with_priority(1)`) logic below intentionally duplicates
    // `Combobox`'s render (`crates/ui/src/components/combobox.rs`). Command
    // is a working, tested component — extracting a shared "measured
    // trigger + floating list" helper is worthwhile but risks destabilizing
    // both call sites for a purely internal cleanup, so it's deferred.
    // TODO: extract a shared helper once a third caller needs the same
    // pattern (or during a dedicated refactor pass), rather than as a
    // side-effect of this fix.
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.input.read(cx).text().to_string();
        if query != self.last_query {
            self.last_query = query;
            self.selected = 0;
        }
        self.clamp_selected(cx);
        let open = self.open;
        let filtered = self.filtered_indices(cx);
        let selected = self.selected;
        let items = self.items.clone();

        let trigger = h_flex()
            .id("command-trigger")
            .w_full()
            .items_center()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(semantic::surface(cx))
            .border_1()
            .border_color(semantic::border(cx))
            .child(div().flex_1().min_w_0().child(self.input.clone()))
            .child(
                div()
                    .id("command-toggle")
                    .cursor_pointer()
                    .child(Icon::new(IconName::MagnifyingGlass).size(IconSize::Small))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open = !this.open;
                        cx.notify();
                    })),
            )
            .on_key_down(cx.listener(Self::on_key_down))
            .child({
                let trigger_bounds = self.trigger_bounds.clone();
                canvas(
                    move |bounds, _window, _cx| trigger_bounds.set(Some(bounds)),
                    |_bounds, _state, _window, _cx| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full()
            });

        let trigger_width = px(360.);
        let hover = semantic::hover_bg(cx);

        v_flex()
            .w(trigger_width)
            .gap_1()
            .child(trigger)
            .when(open, |this| {
                let mut list = v_flex()
                    .w(trigger_width)
                    .p_1()
                    .rounded_md()
                    .bg(semantic::elevated_surface(cx))
                    .border_1()
                    .border_color(semantic::border(cx))
                    .shadow_level(Shadow::Lg)
                    .max_h(px(280.));

                if filtered.is_empty() {
                    list = list.child(
                        div()
                            .px_3()
                            .py_2()
                            .child(Label::new("No results found.").color(Color::Muted)),
                    );
                } else {
                    let mut selectable = 0usize;
                    let mut last_group: Option<SharedString> = None;

                    for (idx, &item_index) in filtered.iter().enumerate() {
                        let Some(item) = items.get(item_index) else {
                            continue;
                        };
                        let CommandItem::Entry { label, .. } = item else {
                            continue;
                        };

                        for (gi, group_item) in items[..item_index].iter().enumerate().rev() {
                            if let CommandItem::Group(group) = group_item {
                                if last_group.as_ref() != Some(group) {
                                    last_group = Some(group.clone());
                                    list = list.child(
                                        ListHeader::new(group.clone())
                                            .inset(true)
                                            .into_any_element(),
                                    );
                                }
                                break;
                            }
                            if gi == 0 {
                                break;
                            }
                        }

                        let is_selected = idx == selected;
                        let label = label.clone();
                        list = list.child(
                            h_flex()
                                .id(("command-item", item_index))
                                .w_full()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .cursor_pointer()
                                .when(is_selected, |this| this.bg(palette::primary(100)))
                                .when(!is_selected, |this| this.hover(move |s| s.bg(hover)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.selected = selectable;
                                    cx.notify();
                                }))
                                .child(Label::new(label)),
                        );
                        selectable += 1;
                    }
                }

                let mut anchor = anchored().snap_to_window_with_margin(px(8.));
                if let Some(bounds) = self.trigger_bounds.get() {
                    anchor = anchor.position(point(
                        bounds.origin.x,
                        bounds.origin.y + bounds.size.height + px(4.),
                    ));
                }

                this.child(deferred(anchor.child(div().occlude().child(list))).with_priority(1))
            })
    }
}

impl Component for Command {
    fn scope() -> ComponentScope {
        ComponentScope::Overlays
    }

    fn description() -> Option<&'static str> {
        Some("A command palette with fuzzy filter and keyboard navigation.")
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let items = vec![
            CommandItem::Group("Suggestions".into()),
            CommandItem::Entry {
                id: "cal".into(),
                label: "Calendar".into(),
            },
            CommandItem::Entry {
                id: "sr".into(),
                label: "Search Emoji".into(),
            },
            CommandItem::Entry {
                id: "calc".into(),
                label: "Calculator".into(),
            },
            CommandItem::Group("Settings".into()),
            CommandItem::Entry {
                id: "prof".into(),
                label: "Profile".into(),
            },
            CommandItem::Entry {
                id: "bill".into(),
                label: "Billing".into(),
            },
        ];
        Some(cx.new(|cx| Command::new(cx, items)).into_any_element())
    }
}

/// Standalone gallery preview for `Command` (stateful `Entity`).
pub fn command_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    Command::preview(_window, cx).unwrap_or_else(|| div().into_any_element())
}
