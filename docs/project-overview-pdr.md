# Project Overview & Product Development Requirements (PDR)

## Vision

A minimal, cross-platform Rust desktop application template built on GPUI — a GPU-accelerated UI framework. Serves as an instant bootstrap for native desktop apps on macOS, Linux, and Windows with no boilerplate, no persistence layer, and no bundled assets.

## Scope

### What's Included

- **Runnable app**: `crates/app` — single "hello world" window demonstrating GPUI entity/update/render pattern
- **Core UI framework**: `crates/gpui` (platform-agnostic) + native platform backends:
  - `crates/gpui_macos` — Metal rendering on macOS
  - `crates/gpui_linux` — wgpu rendering on Linux (Wayland/X11)
  - `crates/gpui_windows` — DirectX rendering on Windows
  - `crates/gpui_wgpu` — Shared wgpu implementation (Linux/fallback)
- **Platform facade**: `crates/gpui_platform` — single API across all OS targets; OS selection automatic
- **UI primitives**: `crates/ui`, `crates/component`, `crates/icons` — reusable GPUI components
- **Theme system**: `crates/theme`, `crates/syntax_theme` — built-in fallback themes; no on-disk JSON needed
- **Support libraries**: macros, utilities, font system, HTTP client, scheduler, collections, tracing, logging

### What's NOT Included

- **No persistence layer**: no database, no SQLite, no file-based state. Intentionally removed to keep template focused. Re-add when app needs state.
- **No bundled assets**: no fonts, no icons, no themes on disk. Uses system fonts and built-in fallback themes only.
- **No web backend**: no wasm, no web server, no REST API. Desktop native only.
- **No cloud/collaboration/extensions**: removed product-specific features. Template is blank canvas.
- **No app icon**: window opens with OS-default icon. Provide one via `assets/` if needed.

## Design Constraints

### Cross-Platform Native

- **macOS**: GPUI windowing + Metal GPU rendering + Cocoa platform APIs
- **Linux**: Wayland/X11 with wgpu rendering + freedesktop standards (desktop entries, XDG portal)
- **Windows**: DirectX 12 rendering + Win32 windowing

No shared windowing abstraction; each platform has custom code. GPUI wraps platform details.

### GPU-Accelerated UI Only

GPUI is not HTML/CSS. It is a **direct-mode, immediate-mode UI framework** using GPU for rendering:
- No DOM
- No virtual tree diffing  
- Flexbox layout calculated per-frame
- Direct styling (no CSS classes)
- Rendered frame-by-frame via GPU command buffers

This enables 60+ fps on trivial hardware but requires thinking in terms of GPUI entities, not web components.

### Minimal Runtime

- Single window per process
- No windowing beyond one app window (no floating panels, no native menus wired yet)
- Intentionally small to keep onboarding clear

Features like multi-window, menu bars, dialogs are architectural choices; add them after learning GPUI basics.

### No Feature Gates for Platform Logic

Platform selection happens via `gpui_platform::application()` — users never write `#[cfg(target_os = "...")]`. The facade is mandatory.

## Primary Goals

1. **Instant bootstrap** — fork template, start feature work in `crates/app/src/main.rs` immediately without boilerplate
2. **Clear architecture** — GPUI layering, platform facades, component reuse patterns all visible from day one
3. **Lean & teachable** — small codebase, no removed features to hunt for, direct source reading possible
4. **Cross-platform parity** — all three platforms work out-of-the-box with `make dev`

## Success Criteria

- [ ] Template builds & runs on macOS, Linux, Windows with single `make dev` command
- [ ] New developers can fork, read `crates/app/src/main.rs`, and add features within 30 minutes
- [ ] All platform backends compile & pass CI (GitHub Actions workflow configured)
- [ ] Documentation clearly explains when to split code into new crates vs. inline features
- [ ] Theme system can be extended without re-adding bundled asset infrastructure
- [ ] Persistence layer can be re-added as a single isolated crate without disrupting core UI flow

## Non-Goals

- Production-ready framework (use Tauri or Electron if mature product needed)
- Backward compatibility with upstream Zed Editor code
- Zero-copy or advanced memory optimizations
- Plugin/extension system
- Accessibility compliance beyond platform defaults

## Development Model

- **Monorepo workspace**: all crates in single Cargo workspace under `crates/`
- **Single binary**: only `crates/app` is executable; all others are libraries
- **Stable Rust**: uses edition 2024, compatible with stable toolchain (no nightly features)
- **No mocks**: tests use real rendering context from GPUI test harness
- **Fast iteration**: `make dev` = one command to see changes (uses `gpui_platform/runtime_shaders` feature to avoid full Metal toolchain on macOS)

## Configuration

- **Edition**: Rust 2024
- **Toolchain**: stable
- **Publishing**: disabled (`publish = false` in workspace)
- **Formatters**: rustfmt with 2024 edition style
- **App ID**: `com.example.app` (platform-specific: used by Wayland desktop entry, Windows registry, macOS bundle)

## Maintenance

- Workspace members must not depend on unused crates (prevent bloat)
- New features start in `crates/app` until API becomes stable and reusable; then split into new crate
- Each crate documents its role in `Cargo.toml` package.metadata or README
- Breaking changes to GPUI APIs are acceptable; this is a living template, not a long-term API contract
