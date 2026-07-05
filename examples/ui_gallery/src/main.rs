#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gallery_app;
mod pages;

use gallery_app::GalleryApp;
use gpui::{
    App, AppContext, AssetSource, Bounds, Font, Pixels, WindowBounds, WindowOptions, font, px, size,
};
use gpui_platform::application;
use theme::{LoadThemes, ThemeSettingsProvider, UiDensity};

const APP_ID: &str = "com.example.ui_gallery";

struct BaseThemeSettingsProvider {
    ui_font: Font,
    buffer_font: Font,
}

impl Default for BaseThemeSettingsProvider {
    fn default() -> Self {
        Self {
            ui_font: font("Inter"),
            buffer_font: font("Inter"),
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

fn run_app() {
    application()
        .with_assets(icons::Assets)
        .run(|cx: &mut App| {
            // Load the bundled Inter font (Tailwind's typeface) into the text
            // system so UI text has a font to shape against.
            if let Ok(Some(inter)) = icons::Assets.load("fonts/Inter.ttf") {
                cx.text_system().add_fonts(vec![inter]).ok();
            }

            // `CodeEditor`/`TerminalView`/`TerminalPanel` (crates/ui) all
            // hardcode `font_family("IBM Plex Mono")` for their monospace
            // content, but never load it themselves — same pattern as Inter
            // above: the *consuming app* is responsible for loading whatever
            // bundled font its `crates/ui` components ask for by name (see
            // `docs/system-architecture.md` § Font System — "no bundled
            // fonts; all system fonts" for the template itself, consumer
            // apps opt in via `rust-embed`). Without this, "IBM Plex Mono"
            // resolves to a fallback font that doesn't have the same glyph
            // coverage, which is what broke text/box-drawing rendering
            // inside the terminal.
            let ibm_plex_mono_fonts = fonts_ibm_plex::Assets::iter()
                .filter(|path| path.starts_with("fonts/IBMPlexMono-"))
                .filter_map(|path| fonts_ibm_plex::Assets.load(&path).ok().flatten())
                .collect::<Vec<_>>();
            cx.text_system().add_fonts(ibm_plex_mono_fonts).ok();

            theme::init(LoadThemes::JustBase, cx);
            theme::set_theme_settings_provider(Box::new(BaseThemeSettingsProvider::default()), cx);

            let bounds = Bounds::centered(None, size(px(1100.0), px(760.0)), cx);
            let window = cx.open_window(
                WindowOptions {
                    app_id: Some(APP_ID.to_string()),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(720.0), px(480.0))),
                    ..Default::default()
                },
                |_, cx| cx.new(|cx| GalleryApp::new(cx)),
            );

            match window {
                Ok(window) => {
                    if let Err(error) = window.update(cx, |_, window, cx| {
                        window.set_window_title("UI Gallery");
                        cx.activate(true);
                    }) {
                        eprintln!("failed to activate gallery window: {error:#}");
                        cx.quit();
                    }
                }
                Err(error) => {
                    eprintln!("failed to open gallery window: {error:#}");
                    cx.quit();
                }
            }
        });
}

fn main() {
    run_app();
}
