use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, Styled};

use crate::prelude::*;

/// A label + input + help/error/success/warning wrapper for form layouts.
///
/// Note: `FormField` renders label/help/validation text around an opaque
/// `AnyElement` child; it cannot retroactively restyle that child's internal
/// focus ring. When `.error(...)`/`.success(...)`/`.warning(...)` is set,
/// pass a child that already reflects that state itself (e.g.
/// `TextInput::invalid(true)`, which already routes to `focus_ring_error`
/// colors) — this is the caller's responsibility, matching how `TextInput`'s
/// own validation flags work.
#[derive(IntoElement, RegisterComponent)]
pub struct FormField {
    label: Option<SharedString>,
    content: AnyElement,
    help: Option<SharedString>,
    error: Option<SharedString>,
    success: Option<SharedString>,
    warning: Option<SharedString>,
}

impl FormField {
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            label: None,
            content: content.into_any_element(),
            help: None,
            error: None,
            success: None,
            warning: None,
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

    /// Shows a success message with a check-circle icon below the field.
    /// Takes precedence over `.help(...)` but not `.error(...)`/`.warning(...)`.
    pub fn success(mut self, success: impl Into<SharedString>) -> Self {
        self.success = Some(success.into());
        self
    }

    /// Shows a warning message with a warning icon below the field.
    /// Takes precedence over `.help(...)`/`.success(...)` but not `.error(...)`.
    pub fn warning(mut self, warning: impl Into<SharedString>) -> Self {
        self.warning = Some(warning.into());
        self
    }
}

impl RenderOnce for FormField {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Precedence: error > warning > success > help.
        let validation_message = self
            .error
            .as_ref()
            .map(|msg| (msg.clone(), IconName::XCircle, palette::danger(600)))
            .or_else(|| {
                self.warning
                    .as_ref()
                    .map(|msg| (msg.clone(), IconName::Warning, palette::warning(600)))
            })
            .or_else(|| {
                self.success
                    .as_ref()
                    .map(|msg| (msg.clone(), IconName::CheckCircle, palette::success(600)))
            });
        let has_validation_message = validation_message.is_some();

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
            .when_some(validation_message, |this, (message, icon, color)| {
                this.child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(
                            Icon::new(icon)
                                .size(IconSize::XSmall)
                                .color(Color::Custom(color)),
                        )
                        .child(
                            Label::new(message)
                                .size(LabelSize::XSmall)
                                .color(Color::Custom(color)),
                        ),
                )
            })
            .when(!has_validation_message, |this| {
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
        Some("Label + input + optional help/error/success/warning text for form layouts.")
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
                .child(
                    FormField::new(Label::new("acme-corp").color(Color::Default))
                        .label("Workspace slug")
                        .success("This slug is available.")
                        .into_any_element(),
                )
                .child(
                    FormField::new(Label::new("admin@example.com").color(Color::Default))
                        .label("Recovery email")
                        .warning("This email is not yet verified.")
                        .into_any_element(),
                )
                .into_any_element(),
        )
    }
}
