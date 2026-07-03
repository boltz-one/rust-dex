use crate::prelude::*;
use gpui::{AnyElement, IntoElement, ParentElement, Styled};

/// Wraps an input-like child (typically an `Entity<TextInput>`) with an
/// optional leading and/or trailing slot (icon or button), presenting a
/// single unified border/background around the whole group.
///
/// Note: `TextInput` (out of scope for this phase) draws its own
/// border/background/focus-ring internally, so the wrapped input still shows
/// its own inner border — this wrapper adds the *outer* shared border and
/// slot dividers around it. A fully seamless single-border look would need a
/// `borderless()` variant on `TextInput` (follow-up, not this phase).
#[derive(IntoElement, RegisterComponent)]
pub struct InputGroup {
    content: AnyElement,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    invalid: bool,
}

impl InputGroup {
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            content: content.into_any_element(),
            leading: None,
            trailing: None,
            invalid: false,
        }
    }

    /// Sets the leading slot (rendered before the input, e.g. an icon).
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// Sets the trailing slot (rendered after the input, e.g. a button).
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Marks the group as invalid, switching the outer border to the danger color.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

impl RenderOnce for InputGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let outer_border = if self.invalid {
            palette::danger(500)
        } else {
            semantic::border(cx)
        };
        let divider = semantic::border_muted(cx);

        h_flex()
            .w_full()
            .items_stretch()
            .rounded_md()
            .border_1()
            .border_color(outer_border)
            .bg(semantic::surface(cx))
            .overflow_hidden()
            .when_some(self.leading, |this, element| {
                this.child(
                    h_flex()
                        .flex_none()
                        .items_center()
                        .px_3()
                        .border_r_1()
                        .border_color(divider)
                        .child(element),
                )
            })
            .child(div().flex_1().min_w_0().child(self.content))
            .when_some(self.trailing, |this, element| {
                this.child(
                    h_flex()
                        .flex_none()
                        .items_center()
                        .px_3()
                        .border_l_1()
                        .border_color(divider)
                        .child(element),
                )
            })
    }
}

impl Component for InputGroup {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("Wraps an input with a leading/trailing slot under one unified border.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .child(
                    InputGroup::new(Label::new("you@example.com").color(Color::Placeholder))
                        .leading(
                            Icon::new(IconName::AtSign)
                                .size(IconSize::Small)
                                .color(Color::Muted),
                        )
                        .into_any_element(),
                )
                .child(
                    InputGroup::new(Label::new("https://example.com").color(Color::Placeholder))
                        .trailing(Button::new("input-group-copy", "Copy").into_any_element())
                        .into_any_element(),
                )
                .child(
                    InputGroup::new(Label::new("Invalid value").color(Color::Placeholder))
                        .invalid(true)
                        .into_any_element(),
                )
                .into_any_element(),
        )
    }
}
