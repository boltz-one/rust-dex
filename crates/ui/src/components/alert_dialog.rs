use smallvec::SmallVec;

use crate::{Modal, ModalFooter, ModalHeader, prelude::*};

/// A thin preset on [`Modal`] with Action/Cancel footer slots.
///
/// Does not duplicate `modal.rs` internals — composes `ModalHeader` and
/// `ModalFooter` with fixed action/cancel buttons.
#[derive(IntoElement, RegisterComponent)]
pub struct AlertDialog {
    id: SharedString,
    title: SharedString,
    description: Option<SharedString>,
    children: SmallVec<[AnyElement; 2]>,
    action_label: SharedString,
    cancel_label: SharedString,
    destructive: bool,
}

impl AlertDialog {
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            children: SmallVec::new(),
            action_label: "Continue".into(),
            cancel_label: "Cancel".into(),
            destructive: false,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn action_label(mut self, label: impl Into<SharedString>) -> Self {
        self.action_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }
}

impl ParentElement for AlertDialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for AlertDialog {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut header = ModalHeader::new()
            .headline(self.title)
            .show_dismiss_button(false);
        if let Some(description) = self.description {
            header = header.description(description);
        }

        let action = self.action_label.clone();
        let cancel = self.cancel_label.clone();
        let destructive = self.destructive;

        div().w(px(440.)).child(
            Modal::new(self.id, None)
                .header(header)
                .children(self.children)
                .footer(
                    ModalFooter::new().end_slot(
                        h_flex()
                            .gap_2()
                            .child(Button::new("alert-cancel", cancel.clone()).color(Color::Muted))
                            .child({
                                let btn = Button::new("alert-action", action.clone());
                                if destructive {
                                    btn.danger()
                                } else {
                                    btn.primary()
                                }
                            }),
                    ),
                ),
        )
    }
}

impl Component for AlertDialog {
    fn scope() -> ComponentScope {
        ComponentScope::Overlays
    }

    fn description() -> Option<&'static str> {
        Some("A modal alert preset with action and cancel buttons, built on Modal.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(
                    AlertDialog::new("alert-basic", "Are you absolutely sure?")
                        .description("This action cannot be undone.")
                        .action_label("Continue")
                        .cancel_label("Cancel"),
                )
                .child(
                    AlertDialog::new("alert-destructive", "Delete account?")
                        .description("This will permanently delete your account and all data.")
                        .action_label("Delete")
                        .destructive(true),
                )
                .into_any_element(),
        )
    }
}
