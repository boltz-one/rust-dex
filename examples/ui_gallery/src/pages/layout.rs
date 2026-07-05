use gpui::{AnyElement, Context, Window};
use ui::prelude::*;
use ui::{AspectRatio, PaneGroupPreview, TitleBar};

use crate::gallery_app::GalleryApp;

use super::section;

impl GalleryApp {
    /// Layout shells, cards, and advanced layout primitives. Resizable,
    /// Calendar, Date Picker, and Carousel are stateful `Entity`s owned by
    /// `GalleryApp` (created once in `new()`) instead of the
    /// recreate-per-render `*Preview::preview()` helpers, so their drag
    /// position / selected day / picked date / active slide persist across
    /// re-renders.
    pub(crate) fn render_layout(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .gap_8()
            .child(section("App Shell", AppShell::preview(window, cx)))
            .child(section("Page Heading", PageHeading::preview(window, cx)))
            .child(section(
                "Section Heading",
                SectionHeading::preview(window, cx),
            ))
            .child(section("Container", Container::preview(window, cx)))
            .child(section("Card", Card::preview(window, cx)))
            .child(section("Aspect Ratio", AspectRatio::preview(window, cx)))
            .child(section(
                "Resizable",
                Some(self.resizable.clone().into_any_element()),
            ))
            .child(section(
                "Calendar",
                Some(self.calendar.clone().into_any_element()),
            ))
            .child(section(
                "Date Picker",
                Some(self.date_picker.clone().into_any_element()),
            ))
            .child(section(
                "Carousel",
                Some(self.carousel.clone().into_any_element()),
            ))
            .child(section(
                "Code Editor",
                Some(code_editor_preview(window, cx)),
            ))
            .child(section("Title Bar", TitleBar::preview(window, cx)))
            .child(section("Pane Group", PaneGroupPreview::preview(window, cx)))
            .child(section(
                "Terminal Panel (chrome only, no PTY)",
                Some(terminal_panel_preview(window, cx)),
            ))
            .child(section(
                "Terminal View (real PTY, macOS-verified only)",
                Some(
                    div()
                        .h(px(280.))
                        .rounded_lg()
                        .overflow_hidden()
                        .child(self.terminal_view.clone())
                        .into_any_element(),
                ),
            ))
            .into_any_element()
    }
}
