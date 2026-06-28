# Template Guide

A minimal GPUI desktop application template. Cross-platform (macOS/Linux/Windows), native only — no web/wasm target.

## What Stays

- `crates/app`: the runnable desktop app (single `hello world` window).
- `crates/gpui`: GPU-accelerated UI framework (core, platform-agnostic).
- `crates/gpui_platform`: platform facade that selects the right backend per OS.
- `crates/gpui_macos` / `crates/gpui_linux` / `crates/gpui_windows` / `crates/gpui_wgpu`: platform backends (Metal / wgpu / DirectX).
- `crates/ui`, `crates/component`, `crates/icons`: reusable GPUI UI components.
- `crates/theme`, `crates/syntax_theme`: theme system with built-in fallback themes (no on-disk theme files needed).
- Supporting crates: `collections`, `gpui_macros`, `gpui_shared_string`, `gpui_util`, `http_client`, `media`, `menu`, `refineable`, `scheduler`, `sum_tree`, `util`, `util_macros`, `logging`, `tracing_facade`, `tracing_facade_macros`, `ui_macros`, `derive_refineable`, `perf`.

## What Was Removed

- `assets/` folder and `crates/assets`: no bundled fonts/icons/themes/sounds. The app uses system fonts and built-in fallback themes.
- `db` / `sqlez` / `sqlez_macros` / `paths` / `release_channel` / `boltz_env_vars` / `env_var`: persistence layer cut (re-add when the app needs state).
- `gpui_web`: web/wasm backend removed.
- App icon: none (window opens with the OS-default icon).

## Run

```sh
make dev
```

The app opens a window with `hello world` centered.

## Develop

Start feature work in `crates/app/src/main.rs` until a concept deserves its own crate. Keep new UI inside GPUI entities and prefer existing `ui` components before adding new primitives.
