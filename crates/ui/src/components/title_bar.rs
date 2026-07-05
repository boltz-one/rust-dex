use component::{Component, ComponentScope, example_group_with_title, single_example};
use gpui::{AnyElement, ClickEvent};

use crate::prelude::*;

/// Height of the [`TitleBar`] chrome.
const TITLE_BAR_HEIGHT: Pixels = px(36.);
/// Diameter of each traffic-light button.
const TRAFFIC_LIGHT_SIZE: Pixels = px(12.);

/// A single traffic-light button (`close`/`minimize`/`maximize`).
struct TrafficLight {
    color: u32,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl TrafficLight {
    fn render(self, id: &'static str) -> impl IntoElement {
        div()
            .id(id)
            .size(TRAFFIC_LIGHT_SIZE)
            .rounded_full()
            .bg(gpui::rgb(self.color))
            .when(self.on_click.is_some(), |this| this.cursor_pointer())
            .when_some(self.on_click, |this, handler| this.on_click(handler))
    }
}

/// Chrome-only window title bar: a centered title plus three macOS-style
/// traffic-light buttons (close/minimize/maximize). Renders no real window
/// controls — callers supply `on_close`/`on_minimize`/`on_maximize`, which
/// is the caller's (`crates/app`, via the `gpui_platform` facade) job, not
/// this crate's. Platform-agnostic: this crate never calls Cocoa/Win32 APIs
/// directly (see `docs/code-standards.md` § Platform-Specific Code).
#[derive(IntoElement, RegisterComponent)]
pub struct TitleBar {
    title: SharedString,
    on_close: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_minimize: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_maximize: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl TitleBar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            on_close: None,
            on_minimize: None,
            on_maximize: None,
        }
    }

    pub fn on_close(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }

    pub fn on_minimize(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_minimize = Some(Box::new(handler));
        self
    }

    pub fn on_maximize(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_maximize = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .id("title-bar")
            .w_full()
            .h(TITLE_BAR_HEIGHT)
            .flex_none()
            .items_center()
            .px_3()
            .gap_2()
            .bg(semantic::surface(cx))
            .border_b_1()
            .border_color(semantic::border(cx))
            .child(
                h_flex()
                    .flex_none()
                    .gap_2()
                    .child(
                        TrafficLight {
                            color: 0xFF5F57,
                            on_click: self.on_close,
                        }
                        .render("title-bar-close"),
                    )
                    .child(
                        TrafficLight {
                            color: 0xFEBC2E,
                            on_click: self.on_minimize,
                        }
                        .render("title-bar-minimize"),
                    )
                    .child(
                        TrafficLight {
                            color: 0x28C840,
                            on_click: self.on_maximize,
                        }
                        .render("title-bar-maximize"),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .justify_center()
                    .child(Label::new(self.title).size(LabelSize::Small)),
            )
            // Balances the traffic-light cluster's width so the title stays
            // visually centered in the bar, not just in the remaining space.
            .child(div().flex_none().w(px(3. * 12. + 2. * 8.)))
    }
}

impl Component for TitleBar {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some(
            "Chrome-only window title bar with a centered title and macOS-style \
             close/minimize/maximize buttons. No real window control calls.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(example_group_with_title(
                    "Basic Usage",
                    vec![
                        single_example(
                            "With Handlers",
                            TitleBar::new("Untitled Project")
                                .on_close(|_, _, _| {})
                                .on_minimize(|_, _, _| {})
                                .on_maximize(|_, _, _| {})
                                .into_any_element(),
                        ),
                        single_example(
                            "Without Handlers",
                            TitleBar::new("Read-only Preview").into_any_element(),
                        ),
                    ],
                ))
                .into_any_element(),
        )
    }
}
