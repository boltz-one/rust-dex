# Architecture Notes

## Entry Point

`crates/boltz/src/main.rs` is the only runnable app entrypoint. It initializes assets, themes, fonts, and a single GPUI window.

## UI Model

The app is intentionally small:

- `HelloWorldApp` renders a full-window flex container.
- The only visible child is a shared `ui::Label` with `hello world`, centered horizontally and vertically.

This follows GPUI's entity/update/render pattern while leaving the application surface blank for the next product.

## Persistence

The runnable app does not write local state yet. `db`, `sqlez`, and related crates are retained for future app state.

## Themes

`theme::init(LoadThemes::All(Box::new(Assets)), cx)` installs bundled themes and makes shared UI components render with the theme system.

## App Identity

The app id is `dev.boltz.app`. The runtime window receives `assets/images/app-icon.png` through `WindowOptions::icon`; macOS additionally sets the Dock icon at startup. Platform package assets live under `assets/macos`, `assets/windows`, and `assets/linux`.

## Removed Surface

The cleanup removed product-specific cloud, collab, agent, extension, release, Docker, Nix, CI, and docs-site code. The root workspace keeps GPUI runtime plus UI, icon, theme, asset, and persistence primitives for the base app.
