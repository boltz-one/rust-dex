use gpui::{AnyElement, App, Window};
use ui::prelude::*;

use crate::gallery_app::GalleryApp;

use super::{field, section};

impl GalleryApp {
    /// "Forms" page: hand-built entries (text input/textarea/select/radio/
    /// checkbox/switch, wired to real entity state) plus Phase 3's
    /// InputGroup/SearchInput/Combobox/MultiSelect/SegmentedControl/
    /// FormField/ActionPanel/FileInput additions (reusing each component's
    /// own preview/`*_preview` helper).
    pub(crate) fn render_forms(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        v_flex()
            .gap_6()
            .w(px(360.))
            .child(field("Email", self.text_input.clone().into_any_element()))
            .child(field("Message", self.textarea.clone().into_any_element()))
            .child(field("Priority", self.select.clone().into_any_element()))
            .child(field(
                "Plan",
                v_flex()
                    .gap_2()
                    .child(RadioButton::new("plan-free").label("Free").checked(true))
                    .child(RadioButton::new("plan-pro").label("Pro"))
                    .into_any_element(),
            ))
            .child(field(
                "Preferences",
                v_flex()
                    .gap_2()
                    .child(Checkbox::new("chk-updates", ToggleState::Selected))
                    .child(Checkbox::new("chk-marketing", ToggleState::Unselected))
                    .into_any_element(),
            ))
            .child(field(
                "Notifications",
                h_flex()
                    .gap_3()
                    .child(Switch::new("sw-on", ToggleState::Selected))
                    .child(Switch::new("sw-off", ToggleState::Unselected))
                    .into_any_element(),
            ))
            .child(section("Input Group", InputGroup::preview(window, cx)))
            .child(section(
                "Search Input",
                Some(search_input_preview(window, cx)),
            ))
            .child(section("Combobox", Some(combobox_preview(window, cx))))
            .child(section(
                "Multi Select",
                Some(multi_select_preview(window, cx)),
            ))
            .child(section(
                "Segmented Control",
                SegmentedControl::preview(window, cx),
            ))
            .child(section("Form Field", FormField::preview(window, cx)))
            .child(section("Action Panel", ActionPanel::preview(window, cx)))
            .child(section("File Input", FileInput::preview(window, cx)))
            .into_any_element()
    }
}
