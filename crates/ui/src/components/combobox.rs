use gpui::{AnyElement, Context, Entity, Render};

use crate::TextInput;
use crate::prelude::*;

/// A typeahead-filtered select: a `TextInput` (typed filter) plus a
/// `DropdownMenu`-style popover list of options filtered by
/// **case-insensitive substring match** (no fuzzy-match / async / remote
/// data — out of scope per plan). Selecting an option sets the input's
/// display text via `TextInput::set_text`.
///
/// Stateful view — create with `cx.new(|cx| Combobox::new(cx, options))`.
pub struct Combobox {
    options: Vec<SharedString>,
    selected: Option<usize>,
    open: bool,
    input: Entity<TextInput>,
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
            input,
        }
    }

    /// The currently selected option, if any.
    pub fn value(&self) -> Option<&SharedString> {
        self.selected.and_then(|i| self.options.get(i))
    }

    /// Options matching the current filter text (case-insensitive substring).
    fn filtered(&self, cx: &App) -> Vec<(usize, SharedString)> {
        let query = self.input.read(cx).text().to_lowercase();
        self.options
            .iter()
            .enumerate()
            .filter(|(_, option)| query.is_empty() || option.to_lowercase().contains(&query))
            .map(|(i, option)| (i, option.clone()))
            .collect()
    }
}

impl Render for Combobox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open;
        let filtered = self.filtered(cx);

        let trigger = h_flex()
            .id("combobox-trigger")
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
                    .cursor_pointer()
                    .child(Icon::new(IconName::ChevronDown).size(IconSize::Small))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open = !this.open;
                        cx.notify();
                    })),
            );

        v_flex()
            .w(px(240.))
            .gap_1()
            .child(trigger)
            .when(open, |this| {
                let hover = semantic::hover_bg(cx);
                let mut list = v_flex()
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
                this.child(list)
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
