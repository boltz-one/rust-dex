# Development Workflow

## Commands

```sh
make dev
make check
make fmt-check
```

The old repo scripts were removed. Use the root Makefile for common development commands.

## App Icon

The source icon is `assets/images/app-icon.png`. Keep platform assets in sync when changing it:

- macOS bundle icon: `assets/macos/app-icon.icns`.
- Windows executable resource: `assets/windows/app-icon.ico`, wired by `crates/boltz/build.rs`.
- Linux desktop icon theme files: `assets/linux/hicolor/*/apps/dev.boltz.app.png`.
- Linux desktop entry: `assets/linux/dev.boltz.app.desktop`.

The runtime app id is `dev.boltz.app`, matching the Linux desktop entry and Wayland app id.

## Adding Screens

Grow the `HelloWorldApp` entity in `crates/boltz/src/main.rs` first. Move a screen into a crate only after it has stable ownership and reusable API boundaries.

## Adding Database State

Use `db` and `sqlez` when the new app has concrete state to store. Keep persistence wrappers small and feature-owned.

## Adding Config

Start with in-memory configuration until a feature needs persisted preferences. When persistence is added, propagate errors to the UI layer.

## Editor Work

No editor surface is wired into the runnable app yet. Add the smallest GPUI-native editor needed by the new app, then extract a dedicated editor crate only after shared editor APIs become clear.
