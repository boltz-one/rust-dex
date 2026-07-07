use gpui::{FollowMode, ListAlignment, ListState, SharedString, list};

use crate::prelude::*;

/// Renders a scrollable chat transcript on top of `gpui`'s virtualized
/// [`list`] element.
///
/// This component owns none of the thread state: the caller constructs and
/// keeps the [`ListState`] (item count, scroll position) alive across
/// renders, and supplies a `render_item` callback that maps an index to a
/// message element (typically an [`AgentMessageBubble`](super::AgentMessageBubble)).
/// `sticky_to_bottom` is likewise caller-computed (e.g. "was the user already
/// scrolled to the bottom before this render?") and only toggles the list's
/// [`FollowMode`] here.
#[derive(IntoElement, RegisterComponent)]
pub struct AgentThreadView {
    id: ElementId,
    list_state: ListState,
    render_item: Box<dyn FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static>,
    sticky_to_bottom: bool,
}

impl AgentThreadView {
    pub fn new(
        id: impl Into<ElementId>,
        list_state: ListState,
        render_item: impl FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            list_state,
            render_item: Box::new(render_item),
            sticky_to_bottom: false,
        }
    }

    /// When `true`, the list auto-scrolls to follow newly appended items
    /// (e.g. the caller determined the user hasn't scrolled up to read
    /// history). When `false`, the current scroll position is preserved.
    pub fn sticky_to_bottom(mut self, sticky: bool) -> Self {
        self.sticky_to_bottom = sticky;
        self
    }
}

impl RenderOnce for AgentThreadView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.list_state.set_follow_mode(if self.sticky_to_bottom {
            FollowMode::Tail
        } else {
            FollowMode::Normal
        });

        div()
            .id(self.id)
            .size_full()
            .child(list(self.list_state, self.render_item).size_full())
    }
}

impl Component for AgentThreadView {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn name() -> &'static str {
        "AgentThreadView"
    }

    fn description() -> Option<&'static str> {
        Some(
            "A scrollable agent chat transcript backed by gpui's virtualized \
             list element. The caller owns the ListState and item data; this \
             component only wires up rendering and sticky-to-bottom follow mode.",
        )
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let messages: Vec<SharedString> = vec![
            "User: How do I run tests?".into(),
            "Assistant: Run `cargo test -p boltz-ui`.".into(),
            "User: Thanks!".into(),
        ];

        let list_state = ListState::new(messages.len(), ListAlignment::Bottom, px(256.));
        let border_color = cx.theme().colors().border_variant;

        Some(
            v_flex()
                .gap_4()
                .child(single_example(
                    "Default",
                    div()
                        .w_96()
                        .h(px(160.))
                        .border_1()
                        .border_color(border_color)
                        .child(
                            AgentThreadView::new("thread-preview", list_state, move |ix, _, _| {
                                div()
                                    .px_2()
                                    .py_1()
                                    .child(Label::new(messages[ix].clone()))
                                    .into_any_element()
                            })
                            .sticky_to_bottom(true),
                        )
                        .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
