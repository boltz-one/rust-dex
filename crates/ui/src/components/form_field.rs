use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, Styled};

use crate::prelude::*;

/// A label + input + help/error wrapper for form layouts.
///
/// Note: `FormField` renders label/help/error text around an opaque
/// `AnyElement` child; it cannot retroactively restyle that child's internal
/// focus ring. When `.error(...)` is set, pass a child that already reflects
/// the error state itself (e.g. `TextInput::invalid(true)`, which already
/// routes to `focus_ring_error` colors) — this is the caller's
/// responsibility, matching how `TextInput`'s own `invalid` flag works.
#[derive(IntoElement, RegisterComponent)]
pub struct FormField {
    label: Option<SharedString>,
    content: AnyElement,
    help: Option<SharedString>,
    error: Option<SharedString>,
}

impl FormField {
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            label: None,
            content: content.into_any_element(),
            help: None,
            error: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn help(mut self, help: impl Into<SharedString>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }
}

impl RenderOnce for FormField {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let has_error = self.error.is_some();

        v_flex()
            .gap_1()
            .when_some(self.label, |this, label| {
                this.child(
                    Label::new(label)
                        .size(LabelSize::Small)
                        .weight(FontWeight::MEDIUM),
                )
            })
            .child(self.content)
            .when_some(self.error, |this, error| {
                this.child(
                    Label::new(error)
                        .size(LabelSize::XSmall)
                        .color(Color::Custom(palette::danger(600))),
                )
            })
            .when(!has_error, |this| {
                this.when_some(self.help, |this, help| {
                    this.child(Label::new(help).size(LabelSize::XSmall).color(Color::Muted))
                })
            })
    }
}

impl Component for FormField {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("Label + input + optional help/error text for form layouts.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(
                    FormField::new(Label::new("you@example.com").color(Color::Placeholder))
                        .label("Email")
                        .help("We'll never share your email.")
                        .into_any_element(),
                )
                .child(
                    FormField::new(Label::new("").color(Color::Placeholder))
                        .label("Password")
                        .error("Password must be at least 8 characters.")
                        .into_any_element(),
                )
                .into_any_element(),
        )
    }
}
