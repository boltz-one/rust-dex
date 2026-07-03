use gpui::{AnyElement, App, Window};
use ui::prelude::*;
use ui::{List, Table};

use super::section;

/// "Data" page (new): Phase 4's DataTable(`Table`)/List/DescriptionList/
/// StatsCard/MediaObject/EmptyState/Feed deliverables.
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_8()
        .child(section("Data Table", Table::preview(window, cx)))
        .child(section("List", List::preview(window, cx)))
        .child(section(
            "Description List",
            DescriptionList::preview(window, cx),
        ))
        .child(section("Stats Card", StatsCard::preview(window, cx)))
        .child(section("Media Object", MediaObject::preview(window, cx)))
        .child(section("Empty State", EmptyState::preview(window, cx)))
        .child(section("Feed", Feed::preview(window, cx)))
        .into_any_element()
}
