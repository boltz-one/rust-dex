use gpui::{AnyElement, Context, Entity, Render};

use crate::prelude::*;
use crate::{InputGroup, TextInput};

/// A search field: `TextInput` with a leading `MagnifyingGlass` icon and a
/// trailing clear (`XMark`) button shown once the input is non-empty.
/// Stateful view — create with `cx.new(|cx| SearchInput::new(cx, "Search…"))`.
pub struct SearchInput {
    input: Entity<TextInput>,
}

impl SearchInput {
    pub fn new(cx: &mut Context<Self>, placeholder: impl Into<SharedString>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx).placeholder(placeholder));
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self { input }
    }

    /// The current search query text.
    pub fn query(&self, cx: &App) -> String {
        self.input.read(cx).text().to_string()
    }
}

impl Render for SearchInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_empty = self.input.read(cx).text().is_empty();

        let mut group = InputGroup::new(self.input.clone()).leading(
            Icon::new(IconName::MagnifyingGlass)
                .size(IconSize::Small)
                .color(Color::Muted),
        );

        if !is_empty {
            group = group.trailing(
                div()
                    .id("search-input-clear")
                    .cursor_pointer()
                    .child(
                        Icon::new(IconName::XMark)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.input.update(cx, |input, cx| input.clear(cx));
                        cx.notify();
                    })),
            );
        }

        group
    }
}

/// Standalone gallery preview for `SearchInput` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching `Select`'s
/// existing convention in this crate).
pub fn search_input_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_4()
        .child(cx.new(|cx| SearchInput::new(cx, "Search…")))
        .into_any_element()
}
