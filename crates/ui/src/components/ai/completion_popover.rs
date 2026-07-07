use std::rc::Rc;

use gpui::{Anchor, AnyElement, Bounds, Pixels, anchored, deferred, point};

use crate::prelude::*;
use crate::score;

/// A single selectable entry offered by a [`CompletionPopover`] (e.g. a
/// `/`-command sourced from the agent runtime). Deliberately plain data —
/// carries no dependency on any particular agent/runtime crate.
#[derive(Clone, Debug, PartialEq)]
pub struct CompletionItem {
    pub label: SharedString,
    pub description: Option<SharedString>,
    pub insert_text: SharedString,
}

impl CompletionItem {
    pub fn new(label: impl Into<SharedString>, insert_text: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            description: None,
            insert_text: insert_text.into(),
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Fuzzy-filters `items` by `query` against each item's label
/// (`command_palette::fuzzy::score`), best match first. An empty `query`
/// returns every item in its original order.
pub fn filter_completions<'a>(items: &'a [CompletionItem], query: &str) -> Vec<&'a CompletionItem> {
    let mut matches: Vec<(&CompletionItem, i32)> = items
        .iter()
        .filter_map(|item| score(query, item.label.as_ref()).map(|s| (item, s)))
        .collect();
    matches.sort_by(|a, b| b.1.cmp(&a.1));
    matches.into_iter().map(|(item, _)| item).collect()
}

/// A floating, fuzzy-filtered completion list. Positioned via the same
/// `deferred`/`anchored` idiom `Combobox`'s option list uses
/// (`crates/ui/src/components/combobox.rs`) rather than `PopoverMenu`,
/// since the trigger here is a typed `/` inside a `TextInput`, not a
/// clickable button `PopoverMenu` expects.
///
/// Stateless builder: the caller (a stateful component owning the input)
/// tracks `selected_ix`/open state and rebuilds this each render, then calls
/// [`CompletionPopover::render`] to obtain the floating element. Up/Down/Esc
/// navigation is the caller's responsibility (via [`filter_completions`]);
/// this type only renders the list and dispatches clicks.
pub struct CompletionPopover {
    items: Vec<CompletionItem>,
    query: SharedString,
    selected_ix: usize,
    anchor: Bounds<Pixels>,
    width: Pixels,
    on_select: Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>,
}

impl CompletionPopover {
    pub fn new(
        items: Vec<CompletionItem>,
        query: impl Into<SharedString>,
        selected_ix: usize,
        anchor: Bounds<Pixels>,
        on_select: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            items,
            query: query.into(),
            selected_ix,
            anchor,
            width: px(280.),
            on_select: Rc::new(on_select),
        }
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// Builds the floating element, anchored above-left of `anchor` (the
    /// input row's screen bounds).
    pub fn render(self, cx: &mut App) -> AnyElement {
        let matched = filter_completions(&self.items, &self.query);
        let selected_ix = self.selected_ix.min(matched.len().saturating_sub(1));
        let hover = semantic::hover_bg(cx);

        let mut list = v_flex()
            .id("completion-popover-list")
            .w(self.width)
            .max_h(px(240.))
            .overflow_y_scroll()
            .p_1()
            .gap_0p5()
            .rounded_md()
            .bg(semantic::elevated_surface(cx))
            .border_1()
            .border_color(semantic::border(cx))
            .shadow_level(Shadow::Lg);

        if matched.is_empty() {
            list = list.child(
                div()
                    .px_3()
                    .py_2()
                    .child(Label::new("No matching commands").color(Color::Muted)),
            );
        }

        for (ix, item) in matched.into_iter().enumerate() {
            let insert_text = item.insert_text.clone();
            let on_select = self.on_select.clone();
            list = list.child(
                v_flex()
                    .id(("completion-popover-item", ix))
                    .w_full()
                    .px_3()
                    .py_1p5()
                    .gap_0p5()
                    .rounded_md()
                    .cursor_pointer()
                    .when(ix == selected_ix, |this| this.bg(hover))
                    .hover(move |style| style.bg(hover))
                    .on_click(move |_, window, cx| on_select(insert_text.clone(), window, cx))
                    .child(Label::new(item.label.clone()))
                    .when_some(item.description.clone(), |this, description| {
                        this.child(
                            Label::new(description)
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                    }),
            );
        }

        deferred(
            anchored()
                .snap_to_window_with_margin(px(8.))
                .anchor(Anchor::BottomLeft)
                .position(point(self.anchor.origin.x, self.anchor.origin.y - px(4.)))
                .child(div().occlude().child(list)),
        )
        .with_priority(1)
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<CompletionItem> {
        vec![
            CompletionItem::new("help", "/help"),
            CompletionItem::new("explain", "/explain"),
            CompletionItem::new("tests", "/tests"),
        ]
    }

    #[test]
    fn empty_query_returns_all_items_in_order() {
        let items = items();
        let matched = filter_completions(&items, "");
        assert_eq!(matched.len(), 3);
        assert_eq!(matched[0].label.as_ref(), "help");
    }

    #[test]
    fn query_filters_by_label_subsequence() {
        let items = items();
        let matched = filter_completions(&items, "exp");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].label.as_ref(), "explain");
    }

    #[test]
    fn no_match_returns_empty() {
        let items = items();
        assert!(filter_completions(&items, "zzz").is_empty());
    }
}
