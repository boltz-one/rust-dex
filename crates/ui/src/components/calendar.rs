//! Single-month calendar grid with day selection and today indicator.
//!
//! Multi-month views and full locale/i18n month-name support are deferred
//! (time budget within Phase 6, not infeasibility).

use chrono::{Datelike, Local, NaiveDate, Weekday};
use gpui::{Context, Render, white};

use crate::prelude::*;

const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn month_grid(year: i32, month: u32) -> Vec<Option<u32>> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month");
    let days_in_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("valid month")
    .signed_duration_since(first)
    .num_days() as u32;

    let leading = match first.weekday() {
        Weekday::Sun => 0,
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
    };

    let mut cells = vec![None; leading as usize];
    for day in 1..=days_in_month {
        cells.push(Some(day));
    }
    while cells.len() % 7 != 0 {
        cells.push(None);
    }
    cells
}

/// Stateful single-month calendar.
///
/// Create with `cx.new(|_| Calendar::new())`.
pub struct Calendar {
    view_year: i32,
    view_month: u32,
    selected: Option<NaiveDate>,
    disabled: Vec<NaiveDate>,
}

impl Calendar {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            view_year: today.year(),
            view_month: today.month(),
            selected: None,
            disabled: Vec::new(),
        }
    }

    pub fn selected(mut self, date: NaiveDate) -> Self {
        self.selected = Some(date);
        self.view_year = date.year();
        self.view_month = date.month();
        self
    }

    pub fn disabled_dates(mut self, dates: impl IntoIterator<Item = NaiveDate>) -> Self {
        self.disabled = dates.into_iter().collect();
        self
    }

    pub fn selection(&self) -> Option<NaiveDate> {
        self.selected
    }

    fn is_disabled(&self, date: NaiveDate) -> bool {
        self.disabled.iter().any(|d| *d == date)
    }

    fn prev_month(&mut self, cx: &mut Context<Self>) {
        if self.view_month == 1 {
            self.view_month = 12;
            self.view_year -= 1;
        } else {
            self.view_month -= 1;
        }
        cx.notify();
    }

    fn next_month(&mut self, cx: &mut Context<Self>) {
        if self.view_month == 12 {
            self.view_month = 1;
            self.view_year += 1;
        } else {
            self.view_month += 1;
        }
        cx.notify();
    }
}

impl Render for Calendar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let today = Local::now().date_naive();
        let year = self.view_year;
        let month = self.view_month;
        let selected = self.selected;
        let grid = month_grid(year, month);
        let month_label = format!("{} {}", MONTHS[month as usize - 1], year);

        let weekday_header = h_flex().children(WEEKDAYS.iter().map(|day| {
            div().flex_1().text_center().child(
                Label::new(*day)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .weight(gpui::FontWeight::MEDIUM),
            )
        }));

        let mut weeks = v_flex().gap_1();
        for week in grid.chunks(7) {
            let mut row = h_flex().gap_1();
            for (col, day) in week.iter().enumerate() {
                let Some(day) = *day else {
                    row = row.child(div().flex_1().h(px(32.)));
                    continue;
                };

                let date = NaiveDate::from_ymd_opt(year, month, day).expect("valid day");
                let is_today = date == today;
                let is_selected = selected == Some(date);
                let is_disabled = self.is_disabled(date);
                let hover = semantic::hover_bg(cx);

                let mut cell = div()
                    .id(ElementId::Name(
                        format!("calendar-day-{year}-{month}-{day}").into(),
                    ))
                    .flex_1()
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .cursor_pointer();

                if is_disabled {
                    cell = cell.cursor_default().child(
                        Label::new(day.to_string())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    );
                } else if is_selected {
                    // `palette::primary(600)` fill + `white()` text is the
                    // established "filled/checked" pairing used across this
                    // crate for solid primary backgrounds (see
                    // `segmented_control.rs`, `stepper.rs`, `toggle.rs`) —
                    // kept consistent here rather than introducing a
                    // one-off token.
                    cell = cell.bg(palette::primary(600)).child(
                        Label::new(day.to_string())
                            .size(LabelSize::Small)
                            .color(Color::Custom(white())),
                    );
                } else {
                    cell = cell
                        // Test-only, no-op in release builds (mirrors the
                        // `Tab`/`ContextMenu`/`ActionPanel` `debug_selector`
                        // precedent): lets integration tests locate a
                        // specific day cell's real rendered bounds to drive a
                        // genuine `simulate_click`, since `Calendar` exposes
                        // no public mutator to select a day directly (only
                        // the render-time `on_click` wiring below actually
                        // sets `selected`).
                        .debug_selector(move || format!("CALENDAR-DAY-{year}-{month}-{day}"))
                        .hover(move |s| s.bg(hover))
                        .when(is_today, |this| {
                            this.border_1().border_color(palette::primary(500))
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected = Some(date);
                            cx.notify();
                        }))
                        .child(Label::new(day.to_string()).size(LabelSize::Small));
                }

                row = row.child(cell);
                let _ = col;
            }
            weeks = weeks.child(row);
        }

        v_flex()
            .id("calendar")
            .w(px(280.))
            .p_3()
            .gap_3()
            .rounded_md()
            .border_1()
            .border_color(semantic::border(cx))
            .bg(semantic::elevated_surface(cx))
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        IconButton::new("calendar-prev", IconName::ChevronLeft)
                            .on_click(cx.listener(|this, _, _, cx| this.prev_month(cx))),
                    )
                    .child(Label::new(month_label).weight(gpui::FontWeight::SEMIBOLD))
                    .child(
                        IconButton::new("calendar-next", IconName::ChevronRight)
                            .on_click(cx.listener(|this, _, _, cx| this.next_month(cx))),
                    ),
            )
            .child(weekday_header)
            .child(weeks)
    }
}

/// Gallery catalog entry for [`Calendar`].
#[derive(IntoElement, RegisterComponent)]
pub struct CalendarPreview;

impl RenderOnce for CalendarPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        cx.new(|_| Calendar::new())
    }
}

impl Component for CalendarPreview {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("Single-month date grid with day selection and today indicator.")
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        CalendarPreview.render(window, cx).into_any_element().into()
    }
}
