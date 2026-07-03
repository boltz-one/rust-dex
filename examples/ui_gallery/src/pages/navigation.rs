use gpui::{AnyElement, App, Window};
use ui::prelude::*;

use super::section;

/// Static "Navigation" showcase: Navbar/Sidebar plus Phase 6's
/// Breadcrumb/Pagination/VerticalNav/Stepper additions.
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_8()
        .child(section("Navbar", Navbar::preview(window, cx)))
        .child(section("Sidebar", Sidebar::preview(window, cx)))
        .child(section("Breadcrumb", Breadcrumb::preview(window, cx)))
        .child(section("Pagination", Pagination::preview(window, cx)))
        .child(section("Vertical Nav", VerticalNav::preview(window, cx)))
        .child(section("Stepper", Stepper::preview(window, cx)))
        .into_any_element()
}
