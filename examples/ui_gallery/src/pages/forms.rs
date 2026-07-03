use gpui::{AnyElement, Context, Window};
use ui::prelude::*;

use crate::gallery_app::GalleryApp;

use super::section;

impl GalleryApp {
    /// "Forms" page: hand-built entries (text input/textarea/select/radio/
    /// checkbox/switch, wired to real entity state) plus Phase 3's
    /// InputGroup/SearchInput/Combobox/MultiSelect/SegmentedControl/
    /// FormField/ActionPanel/FileInput additions. SearchInput/Combobox/
    /// MultiSelect/SegmentedControl are backed by state persisted on
    /// `GalleryApp` (entities created once in `new()`, or a plain index
    /// field) instead of the recreate-per-render `*_preview()` helpers, so
    /// typed/selected/active state survives re-renders. Every control below
    /// is wrapped in `FormField` to show the label/help/error pattern real
    /// forms use.
    pub(crate) fn render_forms(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .gap_6()
            .w(px(360.))
            .child(
                FormField::new(self.text_input.clone())
                    .label("Email")
                    .help("We'll send order confirmations here."),
            )
            .child(
                FormField::new(self.textarea.clone())
                    .label("Message")
                    .help("Optional — visible to your whole team."),
            )
            .child(
                FormField::new(Label::new("SUMMER20").color(Color::Custom(palette::danger(600))))
                    .label("Coupon code")
                    .error("This code expired on June 30."),
            )
            .child(
                FormField::new(Label::new("acme-corp").color(Color::Default))
                    .label("Workspace slug")
                    .success("This slug is available."),
            )
            .child(
                FormField::new(Label::new("admin@example.com").color(Color::Default))
                    .label("Recovery email")
                    .warning("This email is not yet verified."),
            )
            .child(
                FormField::new(self.select.clone())
                    .label("Priority")
                    .help("Determines the SLA reminder cadence."),
            )
            .child(
                FormField::new(
                    v_flex()
                        .gap_2()
                        .child(RadioButton::new("plan-free").label("Free").checked(true))
                        .child(RadioButton::new("plan-pro").label("Pro"))
                        .child(
                            RadioButton::new("plan-enterprise")
                                .label("Enterprise (contact sales)")
                                .disabled(true),
                        ),
                )
                .label("Plan")
                .help("You can upgrade or downgrade at any time."),
            )
            .child(
                FormField::new(
                    v_flex()
                        .gap_2()
                        .child(
                            Checkbox::new("chk-updates", ToggleState::Selected)
                                .label("Product updates"),
                        )
                        .child(
                            Checkbox::new("chk-marketing", ToggleState::Unselected)
                                .label("Marketing emails"),
                        )
                        .child(
                            Checkbox::new("chk-security", ToggleState::Selected)
                                .label("Security alerts")
                                .disabled(true),
                        ),
                )
                .label("Preferences"),
            )
            .child(
                FormField::new(
                    h_flex()
                        .gap_4()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    Label::new("Email")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                                .child(Switch::new("sw-on", ToggleState::Selected)),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    Label::new("Push")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                                .child(Switch::new("sw-off", ToggleState::Unselected)),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    Label::new("SMS")
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                                .child(
                                    Switch::new("sw-disabled", ToggleState::Selected)
                                        .disabled(true),
                                ),
                        ),
                )
                .label("Notifications")
                .help("SMS requires a verified phone number."),
            )
            .child(section("Input Group", InputGroup::preview(window, cx)))
            .child(section(
                "Search Input",
                Some(self.search_input.clone().into_any_element()),
            ))
            .child(section(
                "Combobox",
                Some(self.combobox.clone().into_any_element()),
            ))
            .child(section(
                "Multi Select",
                Some(self.multi_select.clone().into_any_element()),
            ))
            .child(section(
                "Segmented Control",
                Some({
                    let this = cx.entity();
                    SegmentedControl::new("segmented-demo", ["Day", "Week", "Month", "Year"])
                        .active(self.forms_segment)
                        .on_change(move |i, _window, cx| {
                            this.update(cx, |this, cx| {
                                this.forms_segment = i;
                                cx.notify();
                            });
                        })
                        .into_any_element()
                }),
            ))
            .child(section("Form Field", FormField::preview(window, cx)))
            .child(section("Action Panel", ActionPanel::preview(window, cx)))
            .child(section("File Input", FileInput::preview(window, cx)))
            .into_any_element()
    }
}
