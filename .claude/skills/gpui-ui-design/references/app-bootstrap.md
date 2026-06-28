# App bootstrap — how a GPUI app starts and opens a window

Verbatim pattern from `crates/app/src/main.rs` in THIS repo. This is the launch point; everything else (`Render`, components, state) plugs into the window closure.

## The complete `main.rs`

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{
    App, AppContext, Bounds, Context, Font, IntoElement, Pixels, Render, Window, WindowBounds,
    WindowOptions, div, hsla, px, size, white,
};
use gpui_platform::application;
use theme::{LoadThemes, ThemeSettingsProvider, UiDensity};
use ui::prelude::*;

const APP_ID: &str = "com.example.app";

// --- Theme settings provider: tells `theme` which fonts/sizes to use ---
struct BaseThemeSettingsProvider {
    ui_font: Font,
    buffer_font: Font,
}

impl Default for BaseThemeSettingsProvider {
    fn default() -> Self {
        Self { ui_font: Font::default(), buffer_font: Font::default() }
    }
}

impl ThemeSettingsProvider for BaseThemeSettingsProvider {
    fn ui_font<'a>(&'a self, _cx: &'a App) -> &'a Font { &self.ui_font }
    fn buffer_font<'a>(&'a self, _cx: &'a App) -> &'a Font { &self.buffer_font }
    fn ui_font_size(&self, _cx: &App) -> Pixels { px(14.0) }
    fn buffer_font_size(&self, _cx: &App) -> Pixels { px(14.0) }
    fn ui_density(&self, _cx: &App) -> UiDensity { UiDensity::Default }
}

// --- Your root view ---
struct HelloWorldApp;

impl Render for HelloWorldApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().size_full().items_center().justify_center()
            .bg(hsla(0.0, 0.0, 0.08, 1.0))
            .child(Label::new("hello world").size(LabelSize::Large).color(Color::Custom(white())))
    }
}

fn run_app() {
    application().run(|cx: &mut App| {
        // 1. Init theme (colors, typography). JustBase = built-in themes only.
        theme::init(LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(BaseThemeSettingsProvider::default()), cx);

        // 2. Compute a centered window bounds.
        let bounds = Bounds::centered(None, size(px(640.0), px(420.0)), cx);

        // 3. Open the window; the closure returns the root Entity.
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
                    window.set_window_title("App");
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
```

## The pieces, explained

1. **`gpui_platform::application()`** returns the platform app (macOS/Windows/Linux backend, selected by feature flags). `.run(...)` takes over the thread and calls your closure with `&mut App` once initialization is done. This blocks until the app quits.
2. **`theme::init(LoadThemes::JustBase, cx)`** registers the base color themes. `LoadThemes::JustBase` is the no-extra-assets option — use it unless you ship extra theme JSON. Without this, `cx.theme()` has nothing.
3. **`theme::set_theme_settings_provider(...)`** is required: the theme crate needs to know the UI font, buffer font, sizes, and UI density. Implement `ThemeSettingsProvider` (as shown) — `ui_font_size` defaults to `px(14.0)`.
4. **`cx.open_window(opts, |window, cx| root_entity)`** creates the OS window. The closure constructs the **root view** — what the window renders. `cx.new(|_| HelloWorldApp)` makes the `Entity<HelloWorldApp>`. If your view needs `window`/`cx` at construction, use `cx.new(|cx| MyView::new(window, cx))`.
5. **`window.update(cx, |_, window, cx| ...)`** runs after creation to set the title and activate. Use `window.set_window_title("...")`.

## Switching the root view to a real one

When you grow past "hello world", replace `HelloWorldApp` with your app shell view:

```rust
|window, cx| cx.new(|cx| AppShell::new(window, cx))
```

`AppShell` then owns navigation state, child views, etc. — all built with the components in `references/components.md` and the patterns in `references/views-and-state.md`.

## Window options worth knowing

- `app_id` — desktop integration ID (Wayland app_id, macOS bundle hint).
- `window_bounds` — `Windowed(Bounds)`, `Maximized`, `Fullscreen`, or `Borderless`. `Bounds::centered(None, size, cx)` centers on the primary display.
- `window_min_size` — prevents shrinking below a usable size.
- `titlebar` / `decorations` — control chrome (for custom titlebars).
- `window_background` — set `WindowBackgroundAppearance::Blurred`/`Transparent` for the macOS-style vibrancy Zed uses.

## Build & run

```bash
cargo run -p app            # debug build, opens the window
cargo check -p app          # fast type-check
```
For clippy, the repo mirrors Zed's convention: prefer `./script/clippy` if present, else `cargo clippy -p app`. Use `windows_subsystem = "windows"` (the `#![cfg_attr]` at the top) on release so Windows doesn't pop a console window.
