# Architecture Notes

## Entry Point

`crates/app/src/main.rs` is the only runnable app entrypoint. It initializes themes and a single GPUI window via the platform facade.

## UI Model

The app is intentionally small:

- `HelloWorldApp` renders a full-window flex container.
- The only visible child is a shared `ui::Label` with `hello world`, centered horizontally and vertically.

This follows GPUI's entity/update/render pattern while leaving the application surface blank for the next product.

## Platform Selection

`gpui_platform::application()` returns a `gpui::Application` wired to the current OS backend (`gpui_macos` / `gpui_linux` / `gpui_windows`). Consumers never write `#[cfg]` gates for platform selection.

## Themes

`theme::init(LoadThemes::JustBase, cx)` installs the built-in fallback themes (default: "One Dark") and default icon theme. No on-disk theme JSON is required. `ThemeSettingsProvider` supplies UI/buffer font sizes and density.

## Fonts

No fonts are bundled. The text system falls back to platform system fonts (font discovery handled by the platform backend, e.g. font-kit on macOS).

## App Identity

The app id is `com.example.app`, set via `WindowOptions::app_id` (used by Wayland/Linux desktop entry). There is no bundled app icon; the window opens with the OS-default icon.

## Removed Surface

The cleanup removed product-specific cloud, collab, agent, extension, release, Docker, Nix, CI, persistence, and docs-site code, plus the entire `assets/` bundle and the web/wasm backend. The root workspace keeps only the GPUI runtime plus UI/icon/theme primitives for the base app.
