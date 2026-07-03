use gpui::AnyElement;

use crate::Avatar;
use crate::prelude::*;

/// A vertical activity timeline: a left connecting line with avatar + text +
/// timestamp entries (Tailwind "Feed").
#[derive(IntoElement, RegisterComponent)]
pub struct Feed {
    entries: Vec<(AnyElement, SharedString, SharedString)>,
}

impl Feed {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn entry(
        mut self,
        avatar: impl IntoElement,
        content: impl Into<SharedString>,
        timestamp: impl Into<SharedString>,
    ) -> Self {
        self.entries
            .push((avatar.into_any_element(), content.into(), timestamp.into()));
        self
    }
}

impl Default for Feed {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Feed {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .border_l_1()
            .border_color(semantic::border_muted(cx))
            .pl_4()
            .children(
                self.entries
                    .into_iter()
                    .map(|(avatar, content, timestamp)| {
                        h_flex().items_start().gap_3().mb_4().child(avatar).child(
                            v_flex().gap_0p5().child(Label::new(content)).child(
                                Label::new(timestamp)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                        )
                    }),
            )
    }
}

impl Component for Feed {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> Option<&'static str> {
        Some("A vertical activity timeline with a connecting line and avatar/text entries.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let example_avatar = "https://avatars.githubusercontent.com/u/1714999?v=4";

        Some(
            v_flex()
                .gap_6()
                .child(
                    Feed::new()
                        .entry(
                            Avatar::new(example_avatar).size(px(32.)),
                            "Jane Cooper created the project",
                            "1h ago",
                        )
                        .entry(
                            Avatar::new(example_avatar).size(px(32.)),
                            "Cody Fisher commented on an issue",
                            "3h ago",
                        )
                        .entry(
                            Avatar::new(example_avatar).size(px(32.)),
                            "Jenny Wilson closed the milestone",
                            "1d ago",
                        ),
                )
                .into_any_element(),
        )
    }
}
