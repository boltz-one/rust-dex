use gpui::{AnyElement, FontWeight};

use crate::prelude::*;

/// A top-of-page heading: large bold title, optional muted subtitle, and a
/// right-aligned actions slot (e.g. `Button`s). `mb_6`, `justify_between`,
/// `items_center` per Tailwind's Page Heading pattern.
#[derive(IntoElement, RegisterComponent)]
pub struct PageHeading {
    title: SharedString,
    subtitle: Option<SharedString>,
    actions: Option<AnyElement>,
}

impl PageHeading {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            actions: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }
}

impl RenderOnce for PageHeading {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .mb_6()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_size(rems_from_px(30.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(semantic::text(cx))
                            .child(self.title),
                    )
                    .children(self.subtitle.map(|subtitle| {
                        div()
                            .text_size(rems_from_px(14.))
                            .text_color(semantic::text_muted(cx))
                            .child(subtitle)
                    })),
            )
            .children(self.actions)
    }
}

impl Component for PageHeading {
    fn scope() -> ComponentScope {
        ComponentScope::Typography
    }

    fn description() -> Option<&'static str> {
        Some("A page-level heading with title, optional subtitle, and right-aligned actions.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            PageHeading::new("Team Members")
                .subtitle("Manage who has access to this workspace.")
                .actions(Button::new("page-heading-invite", "Invite member"))
                .into_any_element(),
        )
    }
}
