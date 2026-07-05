# Codebase Summary

## Workspace Structure

The `rust-dex` repository is a Rust workspace with 35+ crates under `crates/` and `examples/` (see `/Cargo.toml` `[workspace] members` for the authoritative, current list — this doc is a summary, not re-verified against every addition). All crates share dependencies, edition (2024), and workspace metadata defined in `/Cargo.toml`.

### Crate Categories

#### Application Layer

| Crate | Role | Key Files |
|-------|------|-----------|
| **app** | Runnable desktop binary. Entry point: `main.rs`. Initializes platform, theme, and root UI entity (`HelloWorldApp`). | `src/main.rs` (70 lines) |

#### UI Framework (Core)

| Crate | Role | Key Files |
|-------|------|-----------|
| **gpui** | Platform-agnostic GPU-accelerated UI framework. Provides: window manager, element/render system, flexbox layout, styling, input handling, asset cache, animation, text system. | `src/window.rs` (3.4% of tokens, largest file) `src/elements/div.rs` (2.3%) `src/geometry.rs` (2.2%) |
| **gpui_macros** | Procedural macros for GPUI: `#[derive(Render)]`, `#[derive(IntoElement)]`, `#[action]`, `#[register_action]`, inspector reflection. | `src/derive_*.rs` (6 macro modules) |
| **gpui_shared_string** | Cow-based immutable string for GPUI (memory efficiency). | `gpui_shared_string.rs` (shared string wrapper) |
| **gpui_util** | Small utilities: `ArcCow` type, arc-based copy-on-write. | `src/lib.rs` |

#### Platform Backends

| Crate | Role | Key Files |
|-------|------|-----------|
| **gpui_platform** | Platform facade. Single module: `gpui_platform()` → `gpui::Application` selecting the right backend based on target OS. | `src/gpui_platform.rs` (OS detection + backend routing) |
| **gpui_macos** | macOS backend: Metal rendering, Cocoa windowing, font discovery via Core Text. | `src/metal_renderer.rs` `src/window.rs` `src/text_system.rs` |
| **gpui_linux** | Linux backend: wgpu rendering, Wayland/X11 windowing, freedesktop integration. | `src/linux/wayland.rs` `src/linux/x11.rs` `src/xdg_desktop_portal.rs` |
| **gpui_windows** | Windows backend: DirectX 12 rendering, Win32 windowing. | (wip in template) |
| **gpui_wgpu** | Shared wgpu renderer & text system (used by Linux and Windows). | `src/wgpu_renderer.rs` `src/cosmic_text_system.rs` |

#### UI Components & Styling

| Crate | Role | Key Files |
|-------|------|-----------|
| **ui** | High-level reusable components: `Label`, `Button`, `Input`, etc., built on GPUI primitives. Also: workspace-chrome components (`TabSwitcher`, `PaneGroup`, `TitleBar`) ported/redesigned from Zed's UI patterns; `CodeEditor` (real tree-sitter syntax highlighting when `read_only` + `.language(ext)` — see `language_core`); `TerminalPanel` (static chrome demo) and `TerminalView` (real PTY-backed, spawns an actual shell — see `terminal` crate). | `src/components/` (per-component modules) |
| **component** | Mid-level layout & component infrastructure. | `src/component_layout.rs` `src/component.rs` |
| **icons** | Icon library & icon rendering component. | Icon asset definitions, GPUI `Icon` element |
| **ui_macros** | Macros for UI component definition. | Procedural macro support for `ui` crate |

#### Editor & Terminal

| Crate | Role | Key Files |
|-------|------|-----------|
| **rope** | Copy-on-write rope text buffer (ported from Zed's `rope` crate). Zero coupling to Zed-internal types beyond `sum_tree`/`util`, which this workspace already vendors. Published as `boltz-rope`. | `src/rope.rs`, `src/chunk.rs` |
| **language_core** | Minimal tree-sitter language registry + highlight-query runner (no LSP). Grammars are Cargo features: `lang-rust` (default), `lang-javascript`, `lang-typescript`, `lang-markdown`, `lang-json` — each opt-in to avoid paying binary size for unused languages. Published as `boltz-language-core`. | `src/language_core.rs`, `src/highlight.rs` |
| **terminal** | Real PTY terminal session backed by the published `alacritty_terminal` crate (not GPUI-aware — no view/render code). Spawns the user's shell, exposes write/resize/read-screen-as-text/shutdown. macOS/Unix verified only; Linux/Windows untested in this environment. Published as `boltz-terminal`. | `src/terminal.rs` |

#### Theme & Styling

| Crate | Role | Key Files |
|-------|------|-----------|
| **theme** | Theme system: built-in fallback themes (One Dark), color palettes, UI density. Loaded via `theme::init()`. No on-disk theme files needed. | `LoadThemes::JustBase` for minimal theme bootstrap |
| **syntax_theme** | Syntax highlighting theme definitions — capture-name → `HighlightStyle` map (Zed-standard capture names, e.g. `"keyword"`, `"string"`, `"type"`). `style_for_name` is an EXACT match only; dotted-prefix fallback (`"type.builtin"` → `"type"`) is the caller's job (see `ui`'s `code_editor.rs::style_for_capture`), not built into this crate. | Color scheme for code/text rendering |

#### Font System

| Crate | Role | Key Files |
|-------|------|-----------|
| **font_kit** | Vendored font kit library (Servo's `font-kit`). Handles font discovery, loading, and metrics across platforms. Supports: Core Text (macOS), FreeType (Linux), DirectWrite (Windows). | `src/loaders/` (platform font loaders) `src/sources/` (font sources: fs, core_text, etc.) |

#### Utilities & Infrastructure

| Crate | Role | Key Files |
|-------|------|-----------|
| **collections** | Data structures: `VecMap`, custom collections. Published as `boltz-collections`. | `src/vecmap.rs` |
| **scheduler** | Async task scheduler. Published as `boltz-scheduler`. | Executor & task queuing |
| **sum_tree** | Data structure for efficient tree operations (used by text/buffer code). Published as `boltz-sum-tree`. | Segment tree variant |
| **util** | General utilities, path handling, environment. Published as `boltz-util`. | `src/paths.rs` (2% of tokens, path utilities) |
| **http_client** | HTTP client wrapper (for future features). Published as `boltz-http-client`. | |
| **media** | Media & image handling. Published as `boltz-media`. | |
| **menu** | Menu system (desktop app menus). | |
| **refineable** | Refinement-based API for partial updates. Published as `boltz-refineable`. | `derive_refineable` for derive macro |
| **logging** | Logging infrastructure. Published as `boltz-logging`. | |
| **tracing_facade** | Tracing/metrics facade. Published as `boltz-tracing-facade`. | `tracing_facade_macros` for derive support |

#### Derive Macros

| Crate | Role | Key Files |
|-------|------|-----------|
| **derive_refineable** | Derive macro for `refineable` pattern. | `#[derive(Refineable)]` |
| **util_macros** | General derive macros. Published as `boltz-util-macros`. | |
| **tracing_facade_macros** | Tracing derive macro. | |

#### Examples

| Example | Role | Key Files |
|---------|------|-----------|
| **examples/ui_gallery** | Demo of reusable UI components from `crates/ui`. Shows how to compose and style components. | `src/main.rs` |

## Dependency Graph (High-Level)

```
crates/app
├── gpui_platform → (selects platform backend)
├── gpui → (core UI framework)
├── ui → component, icons
├── theme, syntax_theme
└── + various utilities (logging, http_client, etc.)

crates/gpui
├── gpui_macros
├── gpui_shared_string, gpui_util
├── (no platform-specific code; platform backend is plugged in at runtime)

gpui_platform
├── gpui_macos OR gpui_linux OR gpui_windows (one selected per target)

Platform backends (gpui_macos, gpui_linux, gpui_windows)
├── gpui_wgpu (Linux/Windows use wgpu)
├── font_kit
└── platform-specific libraries (Metal, wgpu, DirectX, Cocoa, Win32, etc.)

ui
├── component, icons
├── gpui
├── theme, syntax_theme
├── language_core → rope, tree-sitter + grammar crates (feature-gated)
└── terminal → alacritty_terminal (terminal has NO gpui dependency itself;
                ui's TerminalView is the only place PTY output meets a
                render tree)

Component tree flow:
app → gpui_platform → [gpui_macos | gpui_linux | gpui_windows] + wgpu/font_kit
    ↓
   ui components → gpui primitives
    ↓
   theme/styling
```

## Entry Point & Initialization

**File**: `/crates/app/src/main.rs` (70 lines)

Flow:
1. Import `gpui_platform::application()` to get platform-specific `gpui::Application`
2. Create `BaseThemeSettingsProvider` implementing `theme::ThemeSettingsProvider`
3. Call `theme::init(LoadThemes::JustBase, cx)` to install built-in themes
4. Register `HelloWorldApp` struct (implements `Render` trait)
5. Open single window with `Window::open()`
6. `HelloWorldApp::render()` returns flex layout with centered `Label` child

No platform-specific conditional compilation; all hidden behind facades.

## Key Files to Understand

### To Learn GPUI Basics
- `crates/gpui/src/window.rs` — window lifecycle, frame loop, context
- `crates/gpui/src/elements/div.rs` — flexbox div element (most common primitive)
- `crates/gpui/src/view.rs` — reactive view pattern
- `crates/app/src/main.rs` — minimal example app

### To Add Features
- `crates/ui/src/` — existing component library; copy patterns for new components
- `crates/theme/src/` — theme system; extend colors/themes here
- `crates/gpui_platform/src/gpui_platform.rs` — if need platform-specific code, use facade

### To Debug Platform Issues
- `crates/gpui_macos/src/platform.rs` — macOS-specific initialization
- `crates/gpui_linux/src/linux/platform.rs` — Linux-specific initialization
- `crates/gpui_windows/src/platform.rs` — Windows-specific initialization

### To Modify Build
- `/Makefile` — dev, check, fmt-check targets
- `/Cargo.toml` — workspace config, dependencies, publish settings
- `rustfmt.toml` — code formatting rules (Rust 2024 edition)
- `.cargo/config.toml` — cargo settings (if any)

## Token Distribution

Top files by token count (from repomix):
1. `crates/gpui/src/window.rs` — 47.4k tokens (3.4%)
2. `crates/gpui/src/elements/div.rs` — 31.3k tokens (2.3%)
3. `crates/gpui/src/geometry.rs` — 30.7k tokens (2.2%)
4. `crates/util/src/paths.rs` — 27.8k tokens (2%)
5. `crates/gpui_macos/src/window.rs` — 24.5k tokens (1.8%)

Total: **1.38M tokens** across 515 files. Core GPUI window/element logic dominates.

## Notes for Contributors

- **No mocks in tests**: Use `TestAppContext` or `VisualTestContext` from GPUI for real rendering context
- **No unused dependencies**: Keep workspace members small; don't depend on crate X just because X exists
- **Platform-agnostic first**: Isolate platform-specific code in backend crates; use `gpui_platform` facade in app code
- **Features, not features gates**: Prefer runtime selection (e.g., `gpui_platform::application()`) over `#[cfg]`
- **Monorepo discipline**: Interdependent crates are fine; cyclic dependencies are not
- **`SyntaxTheme::style_for_name` is exact-match only**: it does NOT do dotted-prefix fallback (`"type.builtin"` does not automatically resolve to `"type"`) — that fallback is a separate method (`highlight_id`, returns an index not a style). Callers mapping tree-sitter capture names to colors need their own fallback (see `ui`'s `code_editor.rs::style_for_capture` for the pattern) or many captures will silently render uncolored.
- **Not every OS-specific dependency needs the `gpui_platform` facade**: if the branching is entirely internal to a third-party crate (e.g. `alacritty_terminal::tty::new` handles forkpty/ConPTY internally with no `#[cfg]` exposed to callers), calling it directly from a platform-agnostic crate doesn't violate the "no `#[cfg(target_os)]` outside platform crates" rule — that rule is about code *this workspace* writes, not opaque dependency internals (same principle already applied to `wgpu`/`cosmic-text`).
