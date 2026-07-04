//! Date picker composed from a popover trigger and [`Calendar`].

use std::cell::Cell;
use std::rc::Rc;

use chrono::{Datelike, Local, NaiveDate};
use gpui::{Bounds, Context, Entity, Pixels, Render, anchored, canvas, deferred, point};

use crate::components::calendar::Calendar;
use crate::prelude::*;

fn format_date(date: NaiveDate) -> SharedString {
    format!("{}/{}/{}", date.month(), date.day(), date.year()).into()
}

/// Popover date picker: trigger button + floating [`Calendar`].
///
/// Create with `cx.new(|cx| DatePicker::new(cx))`.
pub struct DatePicker {
    selected: Option<NaiveDate>,
    open: bool,
    placeholder: SharedString,
    calendar: Entity<Calendar>,
    trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl DatePicker {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let calendar = cx.new(|_| Calendar::new());
        cx.observe(&calendar, |this, cal, cx| {
            if let Some(date) = cal.read(cx).selection() {
                this.selected = Some(date);
                this.open = false;
                cx.notify();
            }
        })
        .detach();

        Self {
            selected: None,
            open: false,
            placeholder: "Pick a date".into(),
            calendar,
            trigger_bounds: Rc::new(Cell::new(None)),
        }
    }

    pub fn selected(mut self, date: NaiveDate, cx: &mut Context<Self>) -> Self {
        self.selected = Some(date);
        self.calendar.update(cx, |cal, _| {
            *cal = Calendar::new().selected(date);
        });
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn value(&self) -> Option<NaiveDate> {
        self.selected
    }
}

impl Render for DatePicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label = self
            .selected
            .map(format_date)
            .unwrap_or_else(|| self.placeholder.clone());
        let has_value = self.selected.is_some();
        let open = self.open;

        let trigger = h_flex()
            .id("date-picker-trigger")
            // Test-only, no-op in release builds (mirrors the
            // `Select`/`Combobox`/`MultiSelect` trigger `debug_selector`
            // precedent) — lets integration tests open the popover via a
            // genuine `simulate_click` at real rendered bounds.
            .debug_selector(|| "DATE-PICKER-TRIGGER".into())
            .w(px(240.))
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(semantic::surface(cx))
            .border_1()
            .border_color(if open {
                palette::primary(500)
            } else {
                semantic::border(cx)
            })
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.open = !this.open;
                cx.notify();
            }))
            .child(Label::new(label).color(if has_value {
                Color::Default
            } else {
                Color::Placeholder
            }))
            .child(Icon::new(IconName::Clock).size(IconSize::Small))
            .child({
                let trigger_bounds = self.trigger_bounds.clone();
                canvas(
                    move |bounds, _, _| trigger_bounds.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full()
            });

        v_flex().gap_1().child(trigger).when(open, |this| {
            let mut anchor = anchored().snap_to_window_with_margin(px(8.));
            if let Some(bounds) = self.trigger_bounds.get() {
                anchor = anchor.position(point(
                    bounds.origin.x,
                    bounds.origin.y + bounds.size.height + px(4.),
                ));
            }

            let floating = deferred(
                anchor.child(
                    div()
                        .occlude()
                        .id("date-picker-popover")
                        // Test-only, no-op in release builds — lets
                        // integration tests assert the popover
                        // opened/closed (its `debug_bounds` resolving or
                        // not) without reaching into private state.
                        .debug_selector(|| "DATE-PICKER-POPOVER".into())
                        .child(self.calendar.clone())
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.open = false;
                            cx.notify();
                        })),
                ),
            )
            .with_priority(1);

            this.child(floating)
        })
    }
}

/// Gallery catalog entry for [`DatePicker`].
#[derive(IntoElement, RegisterComponent)]
pub struct DatePickerPreview;

impl RenderOnce for DatePickerPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        cx.new(|cx| {
            let mut picker = DatePicker::new(cx);
            picker = picker.selected(Local::now().date_naive(), cx);
            picker
        })
    }
}

impl Component for DatePickerPreview {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("Popover trigger + calendar for choosing a date.")
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        DatePickerPreview
            .render(window, cx)
            .into_any_element()
            .into()
    }
}
