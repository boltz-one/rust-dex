use gpui::{AnyElement, App, Window};
use ui::prelude::*;
use ui::{List, ListItem, Pagination, SortDirection, Table};

use super::section;

/// "Data" page (new): Phase 4's DataTable(`Table`)/List/DescriptionList/
/// StatsCard/MediaObject/EmptyState/Feed deliverables. Phase 5 adds a
/// standalone sortable-header + pagination-footer composition demo (see
/// `Table::preview` for the component-level variant catalog).
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_8()
        .child(section("Data Table", Table::preview(window, cx)))
        .child(section(
            "Data Table (sortable header + pagination footer)",
            Some(
                v_flex()
                    .gap_2()
                    .child(
                        Table::new(3)
                            .width(px(420.))
                            .sortable_header(
                                vec!["Name", "Role", "Status"],
                                Some(0),
                                SortDirection::Ascending,
                                |_column, _window, _cx| {},
                            )
                            .row(vec!["Alice", "Engineer", "Active"])
                            .row(vec!["Bob", "Designer", "Invited"])
                            .row(vec!["Charlie", "Manager", "Active"])
                            .into_any_element(),
                    )
                    .child(Pagination::new("data-page-table-pagination", 1, 4))
                    .into_any_element(),
            ),
        ))
        .child(section("List", List::preview(window, cx)))
        .child(section("List Item", ListItem::preview(window, cx)))
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
