# Desktop App Template

A minimal, cross-platform Rust desktop application template built on [GPUI](https://gpui.rs). The main binary is a small GPUI app in `crates/app` that opens a single centered `hello world` window.

## Highlights

- GPUI windowing with direct element styling and flexbox layout
- Reusable `ui` components, `icons`, and a theme system with built-in fallback themes
- Cross-platform native backends: macOS (Metal), Linux (wgpu/Wayland/X11), Windows (DirectX)
- No bundled assets — system fonts and built-in themes keep the template lean
- No persistence layer — re-add when the app needs state

## Quick Start

```sh
make dev        # Run the app
make check      # Verify build
make fmt-check  # Check formatting
```

`make dev` uses the `gpui_platform/runtime_shaders` feature, which avoids requiring the full Xcode Metal toolchain on macOS.

## Documentation

All project documentation lives in `docs/`. Start here:

- **[Project Overview & PDR](./docs/project-overview-pdr.md)** — Vision, scope, constraints, success criteria
- **[Codebase Summary](./docs/codebase-summary.md)** — Workspace structure, crate roles, dependency graph, entry points
- **[Code Standards](./docs/code-standards.md)** — Naming conventions, organization, formatting, testing patterns, platform isolation
- **[System Architecture](./docs/system-architecture.md)** — Layered architecture, platform abstraction, theme/font/text systems, rendering pipeline

### Quick Reference

- **Start developing**: Open `crates/app/src/main.rs` (70 lines, fully commented)
- **Add components**: See patterns in `crates/ui/src/components/`
- **Extend theme**: Edit `crates/theme/src/colors.rs`
- **Platform-specific code**: Isolate in `crates/gpui_macos/`, `crates/gpui_linux/`, `crates/gpui_windows/`; use `gpui_platform` facade in app code

## Project Structure

```
crates/
├── app/                  # Runnable binary (start here)
├── gpui/                 # Core UI framework
├── gpui_platform/        # Platform selection facade
├── gpui_macos/           # macOS Metal backend
├── gpui_linux/           # Linux wgpu backend
├── gpui_windows/         # Windows DirectX backend
├── ui/, component, icons # Reusable components
├── theme/                # Theme system
├── font_kit/             # Vendored font library
└── [25 other crates]     # Utilities, macros, etc.
```

## Features & Next Steps

### Current State

- ✅ Single window with "hello world" label
- ✅ Built-in theme (One Dark) with no bundled assets
- ✅ Cross-platform platform backends (macOS/Linux/Windows)
- ✅ Reusable component library
- ❌ No persistence (database, config files)
- ❌ No icons beyond embedded system defaults
- ❌ No app branding (icon, name, etc.)

### Typical Next Steps

1. **Add custom UI** — Edit `crates/app/src/main.rs` to build out screens
2. **Create custom components** — Add to `crates/ui/src/components/`
3. **Add state persistence** — Create `crates/db/` with SQLite when needed
4. **Branding** — Add app icon to `assets/` (see `docs/development.md`)
5. **Multi-window / menus** — Extend after core app is stable

## Conventions

- **Edition**: Rust 2024
- **Toolchain**: Stable (no nightly)
- **Naming**: kebab-case crates, snake_case modules, PascalCase types
- **Formatting**: `cargo fmt --all` (2024 style via `rustfmt.toml`)
- **Testing**: Use GPUI `TestAppContext`, no mocks
- **Platform gates**: Never use `#[cfg]` in app code; use `gpui_platform` facade

## gpui-probe

`crates/gpui-probe/` is a shared element-tree core for a GPUI inspector overlay and an in-process UI test driver, built on top of `boltz-gpui` only (no theme/ui/icons dependency).

## Reference

The `app/` directory contains the upstream App editor source kept only as a reference; it is gitignored and not part of the workspace.
