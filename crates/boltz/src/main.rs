#![cfg_attr(target_family = "wasm", no_main)]
#![cfg_attr(
    all(not(debug_assertions), not(target_family = "wasm")),
    windows_subsystem = "windows"
)]

use assets::Assets;
use gpui::{
    App, AppContext, Bounds, Context, Font, IntoElement, Pixels, Render, Result, Window,
    WindowBounds, WindowOptions, div, hsla, px, size, white,
};
use gpui_platform::application;
use std::sync::Arc;
use theme::{LoadThemes, ThemeSettingsProvider, UiDensity};
use ui::prelude::*;

const APP_ICON_PNG: &[u8] = include_bytes!("../../../assets/images/app-icon.png");
const APP_ID: &str = "dev.boltz.app";

struct BaseThemeSettingsProvider {
    ui_font: Font,
    buffer_font: Font,
}

impl Default for BaseThemeSettingsProvider {
    fn default() -> Self {
        Self {
            ui_font: Font::default(),
            buffer_font: gpui::font(".BoltzMono"),
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

fn load_app_icon() -> Result<Arc<image::RgbaImage>> {
    Ok(Arc::new(
        image::load_from_memory(APP_ICON_PNG)?.into_rgba8(),
    ))
}

fn run_app() {
    application().with_assets(Assets).run(|cx: &mut App| {
        theme::init(LoadThemes::All(Box::new(Assets)), cx);
        theme::set_theme_settings_provider(Box::new(BaseThemeSettingsProvider::default()), cx);
        if let Err(error) = Assets.load_fonts(cx) {
            eprintln!("failed to load fonts: {error:#}");
        }
        if let Err(error) = gpui_platform::set_application_icon_png(APP_ICON_PNG) {
            eprintln!("failed to set application icon: {error:#}");
        }

        let app_icon = match load_app_icon() {
            Ok(app_icon) => Some(app_icon),
            Err(error) => {
                eprintln!("failed to load application icon: {error:#}");
                None
            }
        };

        let bounds = Bounds::centered(None, size(px(640.0), px(420.0)), cx);
        let window = cx.open_window(
            WindowOptions {
                app_id: Some(APP_ID.to_string()),
                icon: app_icon,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(320.0), px(240.0))),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HelloWorldApp),
        );

        match window {
            Ok(window) => {
                if let Err(error) = window.update(cx, |_, window, cx| {
                    window.set_window_title("Boltz");
                    cx.activate(true);
                }) {
                    eprintln!("failed to activate Boltz window: {error:#}");
                    cx.quit();
                }
            }
            Err(error) => {
                eprintln!("failed to open Boltz window: {error:#}");
                cx.quit();
            }
        }
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_app();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_app();
}
