use gpui::{AnyElement, App, Window};
use ui::prelude::*;

use super::section;

/// "Layout" page (new): Phase 7's AppShell/PageHeading/SectionHeading/
/// Container/Card deliverables. `AppShell::preview()` already bounds itself
/// to a fixed height, avoiding a "shell within a shell" nested-window look.
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
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
        .into_any_element()
}
