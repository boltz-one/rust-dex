use gpui::{AnyElement, Context, Render};

use crate::prelude::*;
use crate::{Chip, ToggleState};

/// A multi-value select: a trigger showing selected values as dismissible
/// `Chip`s plus a `DropdownMenu`-style checklist popover; toggling an option
/// adds/removes its `Chip`. Stateful view — create with
/// `cx.new(|_| MultiSelect::new(options))`.
pub struct MultiSelect {
    options: Vec<SharedString>,
    selected: Vec<usize>,
    open: bool,
    placeholder: SharedString,
}

impl MultiSelect {
    pub fn new(options: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Self {
            options: options.into_iter().map(Into::into).collect(),
            selected: Vec::new(),
            open: false,
            placeholder: "Select options…".into(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn selected_indices(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.selected = indices.into_iter().collect();
        self
    }

    /// The currently selected option labels.
    pub fn values(&self) -> Vec<&SharedString> {
        self.selected
            .iter()
            .filter_map(|&i| self.options.get(i))
            .collect()
    }
}

impl Render for MultiSelect {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open;
        let has_selection = !self.selected.is_empty();

        let mut chips_row = h_flex().flex_1().flex_wrap().gap_1();
        if has_selection {
            for &i in &self.selected {
                if let Some(label) = self.options.get(i).cloned() {
                    chips_row = chips_row.child(Chip::new(label).pill(true).dismissible(
                        cx.listener(move |this, _, _, cx| {
                            this.selected.retain(|&x| x != i);
                            cx.notify();
                        }),
                    ));
                }
            }
        } else {
            chips_row =
                chips_row.child(Label::new(self.placeholder.clone()).color(Color::Placeholder));
        }

        let trigger = h_flex()
            .id("multi-select-trigger")
            .w_full()
            .min_h(px(40.))
            .items_center()
            .justify_between()
            .gap_2()
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
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.open = !this.open;
                cx.notify();
            }))
            .child(chips_row)
            .child(Icon::new(IconName::ChevronDown).size(IconSize::Small));

        v_flex()
            .w(px(280.))
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

                for (i, option) in self.options.iter().enumerate() {
                    let checked = self.selected.contains(&i);
                    let option = option.clone();
                    list = list.child(
                        h_flex()
                            .id(("multi-select-option", i))
                            .w_full()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(pos) = this.selected.iter().position(|&x| x == i) {
                                    this.selected.remove(pos);
                                } else {
                                    this.selected.push(i);
                                }
                                cx.notify();
                            }))
                            .child(
                                Checkbox::new(
                                    ("multi-select-check", i),
                                    if checked {
                                        ToggleState::Selected
                                    } else {
                                        ToggleState::Unselected
                                    },
                                )
                                .visualization_only(true),
                            )
                            .child(Label::new(option)),
                    );
                }
                this.child(list)
            })
    }
}

/// Standalone gallery preview for `MultiSelect` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching `Select`'s
/// existing convention in this crate).
pub fn multi_select_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_4()
        .child(cx.new(|_| {
            MultiSelect::new(["Design", "Engineering", "Marketing", "Sales", "Support"])
                .selected_indices([0, 2])
        }))
        .into_any_element()
}
