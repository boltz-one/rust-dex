# Development Workflow

## Commands

```sh
make dev        # run the app
make check      # cargo check
make fmt-check  # verify formatting
```

Use the root Makefile for common development commands. `make dev` runs with the `gpui_platform/runtime_shaders` feature, which avoids requiring the full Xcode Metal toolchain on macOS.

## Adding an App Icon

There is no bundled icon by default. To add one later, drop a PNG/ICO/ICNS into a new `assets/` folder and wire it:

- Window icon: load the PNG into `image::RgbaImage` and pass it via `WindowOptions::icon`.
- macOS Dock icon: `gpui_platform::set_application_icon_png(bytes)`.
- Windows exe resource: a `build.rs` using `embed-resource` against an `.rc`/`.ico`.
- Linux desktop entry: an `assets/linux/com.example.app.desktop` file plus hicolor icon theme PNGs.

## Adding Custom Fonts / Themes

The template ships no font/theme files. To bundle them, create an `assets/` folder, re-add a small `AssetSource` (e.g. via `rust-embed`), and switch to `LoadThemes::All(Box::new(your_assets))`.

## Adding Screens

Grow the `HelloWorldApp` entity in `crates/app/src/main.rs` first. Move a screen into a crate only after it has stable ownership and reusable API boundaries.

## Adding Persistence

No database layer is bundled. When the app needs state, re-introduce a SQLite layer (e.g. `sqlez`-style wrappers) as a dedicated crate. Keep persistence wrappers small and feature-owned.

## Editor Work

No editor surface is wired into the runnable app yet. Add the smallest GPUI-native editor needed by the new app, then extract a dedicated editor crate only after shared editor APIs become clear.
