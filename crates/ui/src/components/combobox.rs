use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, Bounds, Context, Entity, Pixels, Render, anchored, canvas, deferred, point,
};

use crate::TextInput;
use crate::prelude::*;
use crate::utils::fuzzy_subsequence_score;

/// A typeahead-filtered select: a `TextInput` (typed filter) plus a
/// `DropdownMenu`-style popover list of options filtered by
/// **case-insensitive substring match** by default, or optional subsequence
/// fuzzy filter via [`Combobox::fuzzy_filter`]. Selecting an option sets the input's
/// display text via `TextInput::set_text`.
///
/// Stateful view — create with `cx.new(|cx| Combobox::new(cx, options))`.
pub struct Combobox {
    options: Vec<SharedString>,
    selected: Option<usize>,
    open: bool,
    fuzzy_filter: bool,
    input: Entity<TextInput>,
    /// Real screen bounds of the trigger row, captured via an invisible
    /// `canvas()` measurement child every render and read back on the
    /// *next* render to position the floating option list. See
    /// `Select::trigger_bounds` for the full rationale.
    trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl Combobox {
    pub fn new(
        cx: &mut Context<Self>,
        options: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        let input = cx.new(|cx| TextInput::new(cx).placeholder("Search…"));
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self {
            options: options.into_iter().map(Into::into).collect(),
            selected: None,
            open: false,
            fuzzy_filter: false,
            input,
            trigger_bounds: Rc::new(Cell::new(None)),
        }
    }

    /// The currently selected option, if any.
    pub fn value(&self) -> Option<&SharedString> {
        self.selected.and_then(|i| self.options.get(i))
    }

    /// Enables subsequence fuzzy filtering (hand-rolled, no external crate).
    pub fn fuzzy_filter(mut self, fuzzy_filter: bool) -> Self {
        self.fuzzy_filter = fuzzy_filter;
        self
    }

    /// Options matching the current filter text.
    fn filtered(&self, cx: &App) -> Vec<(usize, SharedString)> {
        let query = self.input.read(cx).text();
        if query.is_empty() {
            return self
                .options
                .iter()
                .enumerate()
                .map(|(i, option)| (i, option.clone()))
                .collect();
        }

        if self.fuzzy_filter {
            let mut matches: Vec<(usize, usize, SharedString)> = self
                .options
                .iter()
                .enumerate()
                .filter_map(|(i, option)| {
                    fuzzy_subsequence_score(query, option).map(|score| (i, score, option.clone()))
                })
                .collect();
            matches.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            matches
                .into_iter()
                .map(|(i, _, option)| (i, option))
                .collect()
        } else {
            let query_lower = query.to_lowercase();
            self.options
                .iter()
                .enumerate()
                .filter(|(_, option)| {
                    query_lower.is_empty() || option.to_lowercase().contains(&query_lower)
                })
                .map(|(i, option)| (i, option.clone()))
                .collect()
        }
    }
}

impl Render for Combobox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open;
        let filtered = self.filtered(cx);

        let trigger = h_flex()
            .id("combobox-trigger")
            // Test-only (no-op in release builds, per `debug_selector`'s own
            // doc comment): lets integration tests locate the trigger's real
            // rendered pixel bounds via `VisualTestContext::debug_bounds`.
            .debug_selector(|| "COMBOBOX-TRIGGER".into())
            .w_full()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(semantic::surface(cx))
            .border_1()
            .border_color(if open {
                palette::primary(500)
            } else {
                semantic::border(cx)
            })
            .child(div().flex_1().min_w_0().child(self.input.clone()))
            .child(
                div()
                    .id("combobox-toggle")
                    // Test-only (no-op in release builds): the actual click
                    // target for opening/closing the list — narrower than
                    // the whole trigger row (which is mostly the embedded
                    // `TextInput`).
                    .debug_selector(|| "COMBOBOX-TOGGLE".into())
                    .cursor_pointer()
                    .child(Icon::new(IconName::ChevronDown).size(IconSize::Small))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open = !this.open;
                        cx.notify();
                    })),
            )
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

        let trigger_width = px(240.);

        v_flex()
            .w(trigger_width)
            .gap_1()
            .child(trigger)
            .when(open, |this| {
                let hover = semantic::hover_bg(cx);
                let mut list = v_flex()
                    .w(trigger_width)
                    .p_1()
                    .rounded_md()
                    .bg(semantic::elevated_surface(cx))
                    .border_1()
                    .border_color(semantic::border(cx))
                    .shadow_level(Shadow::Lg);

                if filtered.is_empty() {
                    list = list.child(
                        div()
                            .px_3()
                            .py_2()
                            .child(Label::new("No matches").color(Color::Muted)),
                    );
                }

                for (i, option) in filtered {
                    let label = option.clone();
                    list = list.child(
                        h_flex()
                            .id(("combobox-option", i))
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected = Some(i);
                                this.open = false;
                                let label = label.clone();
                                this.input
                                    .update(cx, |input, cx| input.set_text(label.to_string(), cx));
                                cx.notify();
                            }))
                            .child(Label::new(option)),
                    );
                }

                // Float the list in a `deferred` overlay pass, anchored just
                // below the trigger's real (previous-frame) bounds, instead
                // of an inline flow child — so it never pushes sibling
                // content down. Same idiom as `PopoverMenu`/`ContextMenu`
                // (`crates/ui/src/components/popover_menu.rs`,
                // `crates/ui/src/components/context_menu.rs`).
                let mut anchor = anchored().snap_to_window_with_margin(px(8.));
                if let Some(bounds) = self.trigger_bounds.get() {
                    anchor = anchor.position(point(
                        bounds.origin.x,
                        bounds.origin.y + bounds.size.height + px(4.),
                    ));
                }
                let floating_list = deferred(
                    anchor.child(
                        div()
                            .occlude()
                            .debug_selector(|| "COMBOBOX-LIST".into())
                            .child(list),
                    ),
                )
                .with_priority(1);
                this.child(floating_list)
            })
    }
}

/// Standalone gallery preview for `Combobox` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching `Select`'s
/// existing convention in this crate).
pub fn combobox_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_4()
        .child(cx.new(|cx| Combobox::new(cx, ["Apple", "Banana", "Cherry", "Date", "Elderberry"])))
        .into_any_element()
}
