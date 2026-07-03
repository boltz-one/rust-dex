use gpui::{AnyElement, App, Context, Window};
use ui::Table;
use ui::prelude::*;

use crate::gallery_app::GalleryApp;

use super::section;

/// Recent-orders sample rows for the Dashboard example.
const RECENT_ORDERS: [(&str, &str, &str); 4] = [
    ("#3421", "Ada Lovelace", "$482.00"),
    ("#3420", "Grace Hopper", "$129.50"),
    ("#3419", "Alan Turing", "$998.10"),
    ("#3418", "Grete Hermann", "$56.20"),
];

/// Sample directory rows for the Table + toolbar example.
const DIRECTORY_USERS: [(&str, &str, &str, &str); 6] = [
    ("Alice Johnson", "alice@acme.co", "Admin", "Active"),
    ("Bob Martinez", "bob@acme.co", "Editor", "Active"),
    ("Carol Nguyen", "carol@acme.co", "Viewer", "Archived"),
    ("David Kim", "david@acme.co", "Editor", "Active"),
    ("Erin O'Brien", "erin@acme.co", "Admin", "Archived"),
    ("Frank Silva", "frank@acme.co", "Viewer", "Active"),
];

impl GalleryApp {
    /// "Examples" page: composed, realistic layouts (dashboard, settings
    /// form, table+toolbar, app shell) built from existing components,
    /// exercising component interplay instead of the one-per-section
    /// isolated previews the other pages show.
    pub(crate) fn render_examples(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .gap_10()
            .child(section("Dashboard", Some(render_dashboard())))
            .child(section(
                "Settings form",
                Some(self.render_settings_form(cx)),
            ))
            .child(section(
                "Table + toolbar",
                Some(self.render_table_toolbar(cx)),
            ))
            .child(section(
                "App shell (demo)",
                Some(render_app_shell_demo(window, cx)),
            ))
            .into_any_element()
    }

    /// Settings form: reuses the same `text_input`/`select` entities as the
    /// Forms page (real, shared state) plus static Switch/RadioButton
    /// controls, wrapped in an `ActionPanel`. Save/Cancel visibly flip a
    /// local `examples_settings_saved` flag rendered as a Badge.
    fn render_settings_form(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let this = cx.entity();
        let this_cancel = this.clone();
        let saved = self.examples_settings_saved;

        v_flex()
            .gap_4()
            .w(px(420.))
            .child(
                SectionHeading::new("Account settings").content(
                    Label::new("Update your profile and notification preferences.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
            )
            .when(saved, |parent| {
                parent.child(
                    Badge::new("Saved")
                        .variant(BadgeVariant::Soft)
                        .color(BadgeColor::Success),
                )
            })
            .child(
                ActionPanel::new()
                    .field(
                        FormField::new(self.text_input.clone())
                            .label("Full name")
                            .help("Shown on your public profile."),
                    )
                    .field(
                        FormField::new(self.select.clone())
                            .label("Role")
                            .help("Controls what this account can access."),
                    )
                    .field(
                        FormField::new(Switch::new("settings-email-notif", ToggleState::Selected))
                            .label("Email notifications"),
                    )
                    .field(
                        FormField::new(
                            v_flex()
                                .gap_2()
                                .child(
                                    RadioButton::new("settings-plan-free")
                                        .label("Free plan")
                                        .checked(true),
                                )
                                .child(RadioButton::new("settings-plan-pro").label("Pro plan")),
                        )
                        .label("Plan"),
                    )
                    .on_save(move |_window, cx| {
                        this.update(cx, |this, cx| {
                            this.examples_settings_saved = true;
                            cx.notify();
                        });
                    })
                    .on_cancel(move |_window, cx| {
                        this_cancel.update(cx, |this, cx| {
                            this.examples_settings_saved = false;
                            cx.notify();
                        });
                    }),
            )
            .into_any_element()
    }

    /// Table + toolbar: the toolbar's `SearchInput` (reused entity) and a
    /// `SegmentedControl` status filter both really narrow the visible rows
    /// of `DIRECTORY_USERS` — not decorative.
    fn render_table_toolbar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let this = cx.entity();
        let query = self.search_input.read(cx).query(cx).to_lowercase();
        let filter_idx = self.examples_status_filter;

        let filtered: Vec<(&str, &str, &str, &str)> = DIRECTORY_USERS
            .iter()
            .copied()
            .filter(|(name, email, _, status)| {
                let matches_query = query.is_empty()
                    || name.to_lowercase().contains(&query)
                    || email.to_lowercase().contains(&query);
                let matches_status = match filter_idx {
                    1 => *status == "Active",
                    2 => *status == "Archived",
                    _ => true,
                };
                matches_query && matches_status
            })
            .collect();

        let count_label = format!("{} of {} users", filtered.len(), DIRECTORY_USERS.len());

        let mut table = Table::new(4)
            .striped()
            .header(vec!["Name", "Email", "Role", "Status"]);
        for (name, email, role, status) in filtered {
            let status_badge = Badge::new(status)
                .variant(BadgeVariant::Soft)
                .color(if status == "Active" {
                    BadgeColor::Success
                } else {
                    BadgeColor::Neutral
                })
                .into_any_element();
            table = table.row(vec![
                name.into_any_element(),
                email.into_any_element(),
                role.into_any_element(),
                status_badge,
            ]);
        }

        v_flex()
            .gap_4()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(div().w(px(240.)).child(self.search_input.clone()))
                            .child(
                                SegmentedControl::new(
                                    "examples-status-filter",
                                    ["All", "Active", "Archived"],
                                )
                                .active(filter_idx)
                                .on_change(
                                    move |i, _window, cx| {
                                        this.update(cx, |this, cx| {
                                            this.examples_status_filter = i;
                                            cx.notify();
                                        });
                                    },
                                ),
                            ),
                    )
                    .child(Button::new("examples-add-user", "Add").primary()),
            )
            .child(
                Label::new(count_label)
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(table)
            .into_any_element()
    }
}

/// Dashboard example: static metric tiles + a `Card`-wrapped orders table.
fn render_dashboard() -> AnyElement {
    let mut table = Table::new(3)
        .width(px(520.))
        .striped()
        .header(vec!["Order", "Customer", "Amount"]);
    for (id, customer, amount) in RECENT_ORDERS {
        table = table.row(vec![id, customer, amount]);
    }

    v_flex()
        .gap_6()
        .child(PageHeading::new("Overview").subtitle("Last 30 days across all workspaces."))
        .child(
            h_flex()
                .flex_wrap()
                .gap_4()
                .child(StatsCard::new("Revenue", "$128,430").trend(StatsTrend::Up, "12.4%"))
                .child(StatsCard::new("Active users", "3,842").trend(StatsTrend::Up, "3.1%"))
                .child(StatsCard::new("Orders", "1,204").trend(StatsTrend::Down, "2.3%"))
                .child(StatsCard::new("Churn rate", "1.8%").trend(StatsTrend::Down, "0.4%")),
        )
        .child(
            Card::new()
                .header(Label::new("Recent orders").size(LabelSize::Default))
                .child(table),
        )
        .into_any_element()
}

/// App shell example: `Navbar` + `Sidebar` + content bounded to a fixed
/// height so it previews inline instead of looking like a nested window.
fn render_app_shell_demo(_window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .h(px(400.))
        .w_full()
        .rounded_lg()
        .overflow_hidden()
        .border_1()
        .border_color(semantic::border(cx))
        .child(
            AppShell::new()
                .navbar(
                    Navbar::new()
                        .child(Label::new("Acme Inbox"))
                        .trailing(Button::new("examples-shell-compose", "Compose").primary()),
                )
                .sidebar(
                    Sidebar::new()
                        .child(SidebarItem::new("shell-inbox", "Inbox").active(true))
                        .child(SidebarItem::new("shell-sent", "Sent"))
                        .child(SidebarItem::new("shell-drafts", "Drafts"))
                        .child(SidebarItem::new("shell-archive", "Archive")),
                )
                .content(
                    v_flex()
                        .p_6()
                        .gap_2()
                        .child(Label::new("Demo application shell").size(LabelSize::Large))
                        .child(
                            Label::new(
                                "Bounded to a fixed height so it previews inline instead of \
                                 taking over the window.",
                            )
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                        ),
                ),
        )
        .into_any_element()
}
