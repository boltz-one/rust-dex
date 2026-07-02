use gpui::Hsla;

use crate::prelude::*;

fn role(severity: Severity, step: u16) -> Hsla {
    match severity {
        Severity::Info => palette::info(step),
        Severity::Success => palette::success(step),
        Severity::Warning => palette::warning(step),
        Severity::Error => palette::danger(step),
    }
}

fn icon_for(severity: Severity) -> IconName {
    match severity {
        Severity::Info => IconName::Info,
        Severity::Success => IconName::CheckCircle,
        Severity::Warning => IconName::ExclamationTriangle,
        Severity::Error => IconName::XCircle,
    }
}

/// A tinted, icon-led message box for one of four severities.
#[derive(IntoElement, RegisterComponent)]
pub struct Alert {
    severity: Severity,
    title: SharedString,
    message: Option<SharedString>,
}

impl Alert {
    pub fn new(severity: Severity, title: impl Into<SharedString>) -> Self {
        Self {
            severity,
            title: title.into(),
            message: None,
        }
    }

    pub fn info(title: impl Into<SharedString>) -> Self {
        Self::new(Severity::Info, title)
    }

    pub fn success(title: impl Into<SharedString>) -> Self {
        Self::new(Severity::Success, title)
    }

    pub fn warning(title: impl Into<SharedString>) -> Self {
        Self::new(Severity::Warning, title)
    }

    pub fn error(title: impl Into<SharedString>) -> Self {
        Self::new(Severity::Error, title)
    }

    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }
}

impl RenderOnce for Alert {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        h_flex()
            .items_start()
            .gap_3()
            .w_full()
            .p_4()
            .rounded_md()
            .border_l_4()
            .border_color(role(self.severity, 500))
            .bg(role(self.severity, 50))
            .child(
                Icon::new(icon_for(self.severity))
                    .size(IconSize::Small)
                    .color(Color::Custom(role(self.severity, 500))),
            )
            .child(
                v_flex()
                    .gap_0p5()
                    .child(
                        Label::new(self.title)
                            .size(LabelSize::Small)
                            .color(Color::Custom(role(self.severity, 800))),
                    )
                    .children(self.message.map(|m| {
                        Label::new(m)
                            .size(LabelSize::XSmall)
                            .color(Color::Custom(role(self.severity, 700)))
                    })),
            )
    }
}

impl Component for Alert {
    fn scope() -> ComponentScope {
        ComponentScope::Notification
    }

    fn description() -> Option<&'static str> {
        Some("A tinted, icon-led message box for info/success/warning/error.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_3()
                .w(px(420.))
                .child(Alert::info("Heads up").message("This is an informational alert."))
                .child(Alert::success("Saved").message("Your changes were saved."))
                .child(Alert::warning("Careful").message("This action needs review."))
                .child(Alert::error("Failed").message("Something went wrong."))
                .into_any_element(),
        )
    }
}
