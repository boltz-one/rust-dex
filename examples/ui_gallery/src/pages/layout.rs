use gpui::{AnyElement, Context, Entity, Window};
use ui::prelude::*;
use ui::{AspectRatio, Pane, PaneGroup, PaneGroupPreview, SplitDirection, TabContent, TitleBar};

use crate::gallery_app::GalleryApp;

use super::section;

/// Placeholder tab content for the Layout page's `PaneGroup` demo.
struct DemoPaneTab(&'static str);

impl TabContent for DemoPaneTab {
    fn render(&self, _focused: bool, _window: &mut Window, _cx: &mut App) -> AnyElement {
        div()
            .p_4()
            .child(Label::new(self.0).color(Color::Muted))
            .into_any_element()
    }

    fn title(&self) -> SharedString {
        self.0.into()
    }
}

/// Builds a 2x2 `PaneGroup` grid (Top/Bottom-Left/Right), one demo tab per
/// pane, for `GalleryApp`'s persistent Layout-page demo. Called once from
/// `GalleryApp::new()` — never from a render closure — so drag/split/focus
/// state survives unrelated re-renders (see this module's doc comment
/// convention shared with `Resizable`/`Calendar`/`Carousel` above).
///
/// Only `PaneGroup`'s public API (`split`/`set_active_pane`) is used to
/// build the grid: split the initial pane right (`top-left | top-right`),
/// split the new right pane down (`bottom-right`), jump back to the
/// original left pane via `set_active_pane`, then split it down
/// (`bottom-left`) — yielding a true 2x2 without needing directional
/// `focus` to cross the resulting nested-axis boundary.
pub(crate) fn build_pane_group_demo(cx: &mut Context<GalleryApp>) -> Entity<PaneGroup> {
    let top_left = cx.new(|cx| {
        let mut pane = Pane::new().with_new_tab_factory(|| Box::new(DemoPaneTab("New tab")));
        pane.add_tab(Box::new(DemoPaneTab("Top Left")), cx);
        pane
    });

    let group = cx.new(|cx| {
        PaneGroup::new(cx, top_left.clone()).with_pane_factory(|cx| {
            let mut pane = Pane::new().with_new_tab_factory(|| Box::new(DemoPaneTab("New tab")));
            pane.add_tab(Box::new(DemoPaneTab("New pane")), cx);
            pane
        })
    });

    group.update(cx, |group, cx| {
        group.split(SplitDirection::Right, cx);
    });
    group.update(cx, |group, cx| {
        group.split(SplitDirection::Down, cx);
    });
    group.update(cx, |group, cx| {
        group.set_active_pane(top_left, cx);
    });
    group.update(cx, |group, cx| {
        group.split(SplitDirection::Down, cx);
    });

    group
}

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
            .child(section(
                "Pane Group (catalog example)",
                PaneGroupPreview::preview(window, cx),
            ))
            .child(section(
                "Pane Group (interactive 2x2 demo — split/close/drag/tabs)",
                Some(
                    div()
                        .h(px(320.))
                        .rounded_lg()
                        .overflow_hidden()
                        .child(self.pane_group.clone())
                        .into_any_element(),
                ),
            ))
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
