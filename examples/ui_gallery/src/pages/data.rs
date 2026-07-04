use gpui::{AnyElement, Context, Window};
use ui::prelude::*;
use ui::{
    Chart, LayoutTable, LayoutTableBody, LayoutTableCell, LayoutTableHead, LayoutTableHeader,
    LayoutTableRow, List, ListItem, Pagination, SortDirection, Table,
};

use crate::gallery_app::GalleryApp;

use super::section;

impl GalleryApp {
    /// "Data" page: Phase 4's DataTable(`Table`)/List/DescriptionList/
    /// StatsCard/MediaObject/EmptyState/Feed deliverables. Phase 5 adds a
    /// standalone sortable-header + pagination-footer composition demo (see
    /// `Table::preview` for the component-level variant catalog). Navigation
    /// Menu and Resizable reuse the `Entity`s owned by `GalleryApp` (created
    /// once in `new()`) instead of the recreate-per-render
    /// `*Preview::preview()` helpers, so their open submenu / drag position
    /// persist across re-renders.
    pub(crate) fn render_data(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
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
            .child(section(
                "Table (static)",
                Some(
                    LayoutTable::new()
                        .child(
                            LayoutTableHeader::new().child(
                                LayoutTableRow::new()
                                    .child(LayoutTableHead::new().child(Label::new("Name")))
                                    .child(LayoutTableHead::new().child(Label::new("Status"))),
                            ),
                        )
                        .child(
                            LayoutTableBody::new()
                                .child(
                                    LayoutTableRow::new()
                                        .child(LayoutTableCell::new().child(Label::new("Alice")))
                                        .child(LayoutTableCell::new().child(Label::new("Active"))),
                                )
                                .child(
                                    LayoutTableRow::new()
                                        .child(LayoutTableCell::new().child(Label::new("Bob")))
                                        .child(LayoutTableCell::new().child(Label::new("Invited"))),
                                ),
                        )
                        .into_any_element(),
                ),
            ))
            .child(section(
                "Navigation Menu",
                Some(self.nav_menu.clone().into_any_element()),
            ))
            .child(section(
                "Resizable",
                Some(self.resizable.clone().into_any_element()),
            ))
            .child(section("Chart", Chart::preview(window, cx)))
            .into_any_element()
    }
}
