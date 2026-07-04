use gpui::AnyElement;

use crate::prelude::*;

/// Layout + validation-state wrapper composing [`FormField`].
///
/// GPUI has no React-Hook-Form equivalent — this struct carries
/// `error`/`touched`/`dirty` flags and feeds them into `FormField`'s existing
/// error slot. Schema validation libraries are explicitly out of scope.
#[derive(IntoElement, RegisterComponent)]
pub struct Form {
    label: Option<SharedString>,
    content: AnyElement,
    help: Option<SharedString>,
    error: Option<SharedString>,
    success: Option<SharedString>,
    warning: Option<SharedString>,
    touched: bool,
    dirty: bool,
}

impl Form {
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            label: None,
            content: content.into_any_element(),
            help: None,
            error: None,
            success: None,
            warning: None,
            touched: false,
            dirty: false,
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

    pub fn success(mut self, success: impl Into<SharedString>) -> Self {
        self.success = Some(success.into());
        self
    }

    pub fn warning(mut self, warning: impl Into<SharedString>) -> Self {
        self.warning = Some(warning.into());
        self
    }

    /// Marks the field as touched — errors are only shown once touched.
    pub fn touched(mut self, touched: bool) -> Self {
        self.touched = touched;
        self
    }

    /// Marks the field as dirty (modified from its initial value).
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn is_touched(&self) -> bool {
        self.touched
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl RenderOnce for Form {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut field = FormField::new(self.content);

        if let Some(label) = self.label {
            field = field.label(label);
        }
        if let Some(help) = self.help {
            field = field.help(help);
        }
        if let Some(success) = self.success {
            field = field.success(success);
        }
        if let Some(warning) = self.warning {
            field = field.warning(warning);
        }
        if self.touched {
            if let Some(error) = self.error {
                field = field.error(error);
            }
        }

        v_flex().gap_1().child(field).when(self.dirty, |this| {
            this.child(
                Label::new("Modified")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
        })
    }
}

impl Component for Form {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some(
            "A form field wrapper with touched/dirty validation state composing FormField. Not a schema-validation library.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(
                    Form::new(Label::new("you@example.com").color(Color::Placeholder))
                        .label("Email")
                        .help("We'll never share your email.")
                        .touched(false),
                )
                .child(
                    Form::new(Label::new("").color(Color::Placeholder))
                        .label("Password")
                        .error("Password must be at least 8 characters.")
                        .touched(true)
                        .dirty(true),
                )
                .into_any_element(),
        )
    }
}
