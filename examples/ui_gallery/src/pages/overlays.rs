use gpui::{AnyElement, ClickEvent, Context, Window, black};
use ui::prelude::*;
use ui::{
    AlertDialog, AlertModal, AnnouncementToast, Drawer, DropdownMenu, HoverCard, Modal,
    ModalFooter, ModalHeader, Popover, Section, Tooltip,
};

use crate::gallery_app::{GalleryApp, ToastItem};

use super::section;

impl GalleryApp {
    /// "Layout" page (new): Phase 5's Modal/AlertModal/Drawer/DropdownMenu/
    /// Popover/Tooltip/AnnouncementToast/ToastStack deliverables. Modal and
    /// ToastStack get real entity-backed triggers (open/close, add/dismiss)
    /// per the phase brief; the rest are self-contained (DropdownMenu) or
    /// purely visual (AlertModal/Drawer/Popover/Tooltip) so their own
    /// `preview()` is reused as-is. Menubar, Command, and Sonner reuse the
    /// `Entity`s owned by `GalleryApp` (see `command` / `sonner` /
    /// `ensure_menubar`) instead of the recreate-per-render `::preview()`
    /// helpers, so their open menu / typed query / queued toasts persist
    /// across re-renders.
    pub(crate) fn render_overlays(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .gap_8()
            .child(section("Modal", Some(self.render_modal_demo(cx))))
            .child(section("Alert Modal", AlertModal::preview(window, cx)))
            .child(section("Drawer", Drawer::preview(window, cx)))
            .child(section("Dropdown Menu", DropdownMenu::preview(window, cx)))
            .child(section("Popover", Popover::preview(window, cx)))
            .child(section("Tooltip", Tooltip::preview(window, cx)))
            .child(section("Toast Stack", Some(self.render_toast_demo(cx))))
            .child(section(
                "Sonner",
                Some(
                    div()
                        .relative()
                        .w(px(480.))
                        .h(px(320.))
                        .overflow_hidden()
                        .child(self.sonner.clone())
                        .into_any_element(),
                ),
            ))
            .child(section("Alert Dialog", AlertDialog::preview(window, cx)))
            .child(section("Hover Card", HoverCard::preview(window, cx)))
            .child(section(
                "Menubar",
                Some(self.ensure_menubar(window, cx).into_any_element()),
            ))
            .child(section(
                "Command",
                Some(self.command.clone().into_any_element()),
            ))
            .child(section(
                "Toast (deprecated)",
                Some(
                    Label::new("Skipped — shadcn deprecated Radix Toast in favor of Sonner.")
                        .color(Color::Muted)
                        .into_any_element(),
                ),
            ))
            .into_any_element()
    }

    fn render_modal_demo(&mut self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .relative()
            .w_full()
            .h(px(220.))
            .border_1()
            .border_color(semantic::border_muted(cx))
            .rounded_lg()
            .p_4()
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        Label::new("Click the button to open a real, entity-backed modal.")
                            .color(Color::Muted),
                    )
                    .child(
                        Button::new("open-modal", "Open Modal").on_click(cx.listener(
                            |this, _: &ClickEvent, _, cx| {
                                this.modal_open = true;
                                cx.notify();
                            },
                        )),
                    ),
            )
            .when(self.modal_open, |parent| {
                parent.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(black().opacity(0.5))
                        .child(
                            div().w(px(360.)).child(
                                Modal::new("gallery-modal", None)
                                    .header(
                                        ModalHeader::new()
                                            .headline("Modal Demo")
                                            .show_dismiss_button(false),
                                    )
                                    .section(Section::new().child(Label::new(
                                        "This modal's visibility is driven by real GalleryApp state, not a mock.",
                                    )))
                                    .footer(ModalFooter::new().end_slot(
                                        Button::new("close-modal", "Close").on_click(cx.listener(
                                            |this, _: &ClickEvent, _, cx| {
                                                this.modal_open = false;
                                                cx.notify();
                                            },
                                        )),
                                    )),
                            ),
                        ),
                )
            })
            .into_any_element()
    }

    fn render_toast_demo(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let toasts = self
            .toasts
            .iter()
            .map(|toast| {
                let id = toast.id;
                div()
                    .id(("gallery-toast", id))
                    .w_80()
                    .child(
                        AnnouncementToast::new()
                            .severity(toast.severity)
                            .heading(toast.heading.clone())
                            .description(toast.description.clone())
                            .dismiss_on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.toasts.retain(|t| t.id != id);
                                cx.notify();
                            })),
                    )
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        let spawn = |severity: Severity, heading: &'static str, description: &'static str| {
            cx.listener(move |this: &mut GalleryApp, _: &ClickEvent, _, cx| {
                let id = this.next_toast_id;
                this.next_toast_id += 1;
                this.toasts.push(ToastItem {
                    id,
                    severity,
                    heading: heading.into(),
                    description: description.into(),
                });
                cx.notify();
            })
        };

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("add-toast-success", "Show success").on_click(spawn(
                            Severity::Success,
                            "Saved",
                            "Your changes were saved.",
                        )),
                    )
                    .child(
                        Button::new("add-toast-warning", "Show warning").on_click(spawn(
                            Severity::Warning,
                            "Storage almost full",
                            "You're using 92% of your quota.",
                        )),
                    )
                    .child(Button::new("add-toast-error", "Show error").on_click(spawn(
                        Severity::Error,
                        "Upload failed",
                        "Check your connection and try again.",
                    ))),
            )
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(160.))
                    .border_1()
                    .border_color(semantic::border_muted(cx))
                    .rounded_lg()
                    .overflow_hidden()
                    .child(ToastStack::new().children(toasts)),
            )
            .into_any_element()
    }
}
