use gpui::{Context, Render};

use crate::prelude::*;

/// A dropdown select: a trigger styled like a text field plus an expandable
/// option list. Stateful view — create with `cx.new(|_| Select::new(options))`.
pub struct Select {
    options: Vec<SharedString>,
    selected: Option<usize>,
    open: bool,
    placeholder: SharedString,
}

impl Select {
    pub fn new(options: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Self {
            options: options.into_iter().map(Into::into).collect(),
            selected: None,
            open: false,
            placeholder: "Select…".into(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    /// The currently selected option text, if any.
    pub fn value(&self) -> Option<&SharedString> {
        self.selected.and_then(|i| self.options.get(i))
    }
}

impl Render for Select {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label: SharedString = self
            .selected
            .and_then(|i| self.options.get(i).cloned())
            .unwrap_or_else(|| self.placeholder.clone());
        let has_value = self.selected.is_some();

        let trigger = h_flex()
            .id("select-trigger")
            .w_full()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(semantic::surface(cx))
            .border_1()
            .border_color(if self.open {
                palette::primary(500)
            } else {
                semantic::border(cx)
            })
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.open = !this.open;
                cx.notify();
            }))
            .child(Label::new(label).color(if has_value {
                Color::Default
            } else {
                Color::Placeholder
            }))
            .child(Icon::new(IconName::ChevronDown).size(IconSize::Small));

        v_flex()
            .w(px(240.))
            .gap_1()
            .child(trigger)
            .when(self.open, |this| {
                let hover = semantic::hover_bg(cx);
                let mut list = v_flex()
                    .p_1()
                    .rounded_md()
                    .bg(semantic::elevated_surface(cx))
                    .border_1()
                    .border_color(semantic::border(cx))
                    .shadow_level(Shadow::Lg);
                for (i, option) in self.options.iter().enumerate() {
                    let option = option.clone();
                    list = list.child(
                        h_flex()
                            .id(("select-option", i))
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected = Some(i);
                                this.open = false;
                                cx.notify();
                            }))
                            .child(Label::new(option)),
                    );
                }
                this.child(list)
            })
    }
}
