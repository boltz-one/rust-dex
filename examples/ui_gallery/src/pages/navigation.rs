use gpui::{AnyElement, Context, Window};
use ui::prelude::*;
use ui::{Disclosure, Tab, TabBar, TabBarStyle};

use crate::gallery_app::GalleryApp;

use super::section;

/// Tab labels shared by both the underline and pills `TabBar` demos below,
/// so their indices line up with `GalleryApp::nav_tab`.
const NAV_TAB_LABELS: [&str; 3] = ["Overview", "Activity", "Settings"];

impl GalleryApp {
    /// "Navigation" showcase: Navbar/Sidebar plus Phase 6's
    /// Breadcrumb/Pagination/VerticalNav/Stepper additions, plus a real
    /// `TabBar`/`Tab` demo (underline + pills) wired to `self.nav_tab` so
    /// clicking a tab persists the active index across re-renders instead of
    /// a static `::preview()` mock. Navigation Menu and Menubar reuse the
    /// `Entity`s owned by `GalleryApp` (see `nav_menu` / `ensure_menubar`)
    /// instead of the recreate-per-render `::preview()` helpers, so their
    /// open submenu/dropdown state persists across re-renders.
    pub(crate) fn render_navigation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active_tab = self.nav_tab;

        let mut underline_bar = TabBar::new("nav-tabs-underline");
        for (index, label) in NAV_TAB_LABELS.into_iter().enumerate() {
            underline_bar = underline_bar.child(
                Tab::new(("nav-tab-underline", index))
                    .toggle_state(active_tab == index)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.nav_tab = index;
                        cx.notify();
                    }))
                    .child(label),
            );
        }

        let mut pills_bar = TabBar::new("nav-tabs-pills").style(TabBarStyle::Pills);
        for (index, label) in NAV_TAB_LABELS.into_iter().enumerate() {
            pills_bar = pills_bar.child(
                Tab::new(("nav-tab-pills", index))
                    .style(TabBarStyle::Pills)
                    .toggle_state(active_tab == index)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.nav_tab = index;
                        cx.notify();
                    }))
                    .child(label),
            );
        }

        v_flex()
            .gap_8()
            .child(section("Navbar", Navbar::preview(window, cx)))
            .child(section("Sidebar", Sidebar::preview(window, cx)))
            .child(section("Breadcrumb", Breadcrumb::preview(window, cx)))
            .child(section("Pagination", Pagination::preview(window, cx)))
            .child(section("Vertical Nav", VerticalNav::preview(window, cx)))
            .child(section("Stepper", Stepper::preview(window, cx)))
            .child(section(
                "Navigation Menu",
                Some(self.nav_menu.clone().into_any_element()),
            ))
            .child(section(
                "Menubar",
                Some(self.ensure_menubar(window, cx).into_any_element()),
            ))
            .child(section("Accordion", Disclosure::preview(window, cx)))
            .child(section("Collapsible", Disclosure::preview(window, cx)))
            .child(section(
                "Tabs",
                Some(
                    v_flex()
                        .gap_4()
                        .child(underline_bar)
                        .child(pills_bar)
                        .into_any_element(),
                ),
            ))
            .into_any_element()
    }
}
