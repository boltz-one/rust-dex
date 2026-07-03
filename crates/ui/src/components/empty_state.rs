use gpui::{AnyElement, FontWeight};

use crate::prelude::*;

/// A centered placeholder shown when a list/table/collection has no content.
#[derive(IntoElement, RegisterComponent)]
pub struct EmptyState {
    icon: IconName,
    heading: SharedString,
    description: Option<SharedString>,
    action: Option<AnyElement>,
}

impl EmptyState {
    pub fn new(heading: impl Into<SharedString>) -> Self {
        Self {
            icon: IconName::User,
            heading: heading.into(),
            description: None,
            action: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .items_center()
            .justify_center()
            .gap_2()
            .py_12()
            .child(
                Icon::new(self.icon)
                    .size(IconSize::XLarge)
                    .color(Color::Custom(semantic::text_muted(cx))),
            )
            .child(Label::new(self.heading).weight(FontWeight::MEDIUM))
            .children(
                self.description
                    .map(|d| Label::new(d).size(LabelSize::Small).color(Color::Muted)),
            )
            .children(self.action)
    }
}

impl Component for EmptyState {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> Option<&'static str> {
        Some("A centered placeholder shown when a list/table/collection has no content.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .children(vec![
                    example_group_with_title(
                        "Basic",
                        vec![single_example(
                            "No description",
                            EmptyState::new("No projects").into_any_element(),
                        )],
                    ),
                    example_group_with_title(
                        "With description and action",
                        vec![single_example(
                            "Full",
                            EmptyState::new("No projects")
                                .icon(IconName::File)
                                .description("Get started by creating a new project.")
                                .action(Button::new("new_project", "New Project"))
                                .into_any_element(),
                        )],
                    ),
                ])
                .into_any_element(),
        )
    }
}
