#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{
    App, AppContext, Bounds, Context, Font, IntoElement, Pixels, Render, Window, WindowBounds,
    WindowOptions, div, hsla, px, size, white,
};
use gpui_platform::application;
use theme::{LoadThemes, ThemeSettingsProvider, UiDensity};
use ui::prelude::*;

const APP_ID: &str = "com.example.{{project-name}}";

struct BaseThemeSettingsProvider {
    ui_font: Font,
    buffer_font: Font,
}

impl Default for BaseThemeSettingsProvider {
    fn default() -> Self {
        Self {
            ui_font: Font::default(),
            buffer_font: Font::default(),
        }
    }
}

impl ThemeSettingsProvider for BaseThemeSettingsProvider {
    fn ui_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.ui_font
    }

    fn buffer_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.buffer_font
    }

    fn ui_font_size(&self, _cx: &App) -> Pixels {
        px(14.0)
    }

    fn buffer_font_size(&self, _cx: &App) -> Pixels {
        px(14.0)
    }

    fn ui_density(&self, _cx: &App) -> UiDensity {
        UiDensity::Default
    }
}

struct HelloWorldApp;

impl Render for HelloWorldApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.08, 1.0))
            .child(
                Label::new("hello world")
                    .size(LabelSize::Large)
                    .color(Color::Custom(white())),
            )
    }
}

fn run_app() {
    application().run(|cx: &mut App| {
        theme::init(LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(BaseThemeSettingsProvider::default()), cx);

        let bounds = Bounds::centered(None, size(px(640.0), px(420.0)), cx);
        let window = cx.open_window(
            WindowOptions {
                app_id: Some(APP_ID.to_string()),
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(320.0), px(240.0))),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HelloWorldApp),
        );

        match window {
            Ok(window) => {
                if let Err(error) = window.update(cx, |_, window, cx| {
                    window.set_window_title("{{project-name}}");
                    cx.activate(true);
                }) {
                    eprintln!("failed to activate app window: {error:#}");
                    cx.quit();
                }
            }
            Err(error) => {
                eprintln!("failed to open app window: {error:#}");
                cx.quit();
            }
        }
    });
}

fn main() {
    run_app();
}
