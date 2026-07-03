# Research: VisualTestContext API + offscreen "computer-use" test harness for GPUI

## 1. API Reference

### VisualTestAppContext — `crates/gpui/src/app/visual_test_context.rs`
Real macOS (MacPlatform) rendering + `TestDispatcher` for deterministic scheduling. Distinct from `TestAppContext` (mocked `TestPlatform`, no real GPU render).

- **Construct**: `VisualTestAppContext::new(platform: Rc<dyn Platform>)` or `with_asset_source(platform, asset_source)` (needed for real SVG icon rendering — default `()` asset source = no icons). `platform` comes from `gpui_platform::current_platform(headless: bool)` (`crates/gpui_platform/src/gpui_platform.rs:33-38`, `#[cfg(target_os="macos")] Rc::new(gpui_macos::MacPlatform::new(headless))`). **macOS-only path confirmed**; no Windows/Linux branch reaches VisualTestAppContext usefully.
- **Open window offscreen**: `open_offscreen_window::<V: Render>(size: Size<Pixels>, build_root: FnOnce(&mut Window, &mut App) -> Entity<V>) -> Result<WindowHandle<V>>` (visual_test_context.rs:97). Places window at `(-10000, -10000)`, `show: true`, `focus: false` — invisible to user, still compositor-rendered so screenshot works. `open_offscreen_window_default` = 1280x800 convenience (line 122).
- **Input simulation** (all on `WindowHandle`/`AnyWindowHandle`, all call `run_until_parked()` internally except raw `dispatch_*`):
  - `simulate_keystrokes(window, "cmd-p escape")` — space-separated `Keystroke::parse` tokens (line 225).
  - `dispatch_keystroke(window, Keystroke)` — single raw keystroke, no auto-park (line 235).
  - `simulate_input(window, "hello")` — types text char-by-char as keystrokes w/ `key_char` set; this is the way to type into a focused text input (line 243).
  - `simulate_mouse_move/down/up/click(window, Point<Pixels>, MouseButton/Modifiers)` — click = down+up at same pos (lines 257-322).
  - `simulate_event<E: InputEvent>(window, event)` — generic; **use this for scroll** since there is no dedicated `simulate_scroll_wheel` helper — construct `ScrollWheelEvent { position, delta, modifiers, touch_phase }` (type at `crates/gpui/src/window.rs:5336` area) and pass it here.
  - `dispatch_action(window, action: impl Action)` — for keybinding/menu-driven actions (line 334).
- **Screenshot**: `capture_screenshot(window: AnyWindowHandle) -> Result<RgbaImage>` (`image::RgbaImage`), gated `#[cfg(any(test, feature="test-support"))]` (line 384). Internally calls `window.render_to_image()` (`crates/gpui/src/window.rs:2123`) — direct Metal texture read, does **not** require the window to be visible on any display (works because it's still compositor-rendered offscreen).
- **State/assertion primitives**: `update()/read()` (App-level), `update_window()/read_window()`, `read_entity()` via the `AppContext` impl, `wait_for(entity, predicate, timeout)` (polls w/ `run_until_parked` + 10ms timer, async), `wait_for_animations()` (32ms + park), `run_until_parked()`, `advance_clock(Duration)`.
- Also has `has_global/read_global/set_global/update_global`, `write_to_clipboard/read_from_clipboard`.

### Sibling APIs (context)
- `TestAppContext` (`crates/gpui/src/app/test_context.rs`) has the **same method names** (`simulate_click`, `simulate_keystrokes`, `simulate_input`, `simulate_mouse_move`, `simulate_event`) but drives the mocked `TestPlatform` — no real rendering, used with `#[gpui::test]` macro. This is the standard, fast, cross-platform-ish way to drive UI + assert entity state; screenshots aren't meaningful there.
- `HeadlessTestAppContext` (`crates/gpui/src/app/headless_app_context.rs`) also exposes `capture_screenshot` — a lighter headless variant; not the one asked about but worth noting as alternative if the platform can be swapped to a null/software renderer [unverified — didn't inspect its `Platform` backing in detail].

## 2. Existing usage pattern (only one found repo-wide)

`crates/gpui_platform/src/gpui_platform.rs` (tests module, ~line 71-150) is **the only place `VisualTestAppContext::new(...)` is actually instantiated in this repo**:
```rust
use gpui::{AppContext, Empty, VisualTestAppContext};
// Note: All VisualTestAppContext tests are ignored by default because they require
// [macOS/real rendering — comment truncated, not fully read]
let mut cx = VisualTestAppContext::new(current_platform(false));
```
Tests there are `#[ignore]`-by-default (grep-confirmed comment, exact body not read — verify before copying). No crate under `crates/ui/**` or `examples/ui_gallery/**` currently uses `VisualTestAppContext` or `open_offscreen_window`. `crates/ui/src/components/context_menu.rs` has `#[gpui::test]` (grep hit only, content unread) — likely the best in-repo model for `TestAppContext`-based click-driven component tests; inspect it directly before writing new tests.
`crates/ui/src/components/data_table/tests.rs` is pure logic unit tests (no GPUI context at all) — not a UI-driving example, despite being named in the request.

## 3. Suggested test skeleton (sketch only, not implementation)

```rust
#[test]
#[ignore] // real macOS render, run explicitly / not in default CI
fn gallery_expand_and_type() {
    let mut cx = VisualTestAppContext::with_asset_source(
        gpui_platform::current_platform(true), // headless=true
        Arc::new(examples_ui_gallery::Assets), // real assets for icons
    );
    let window = cx.open_offscreen_window_default(|window, cx| {
        cx.new(|cx| GalleryApp::new(window, cx))
    }).unwrap();

    // type into a focused input
    cx.simulate_input(window.into(), "hello");
    let text = cx.read_window(&window, |view, cx| view.read(cx).input_value.clone()).unwrap();
    assert_eq!(text, "hello");

    // click a tab
    cx.simulate_click(window.into(), point(px(120.), px(40.)), Modifiers::default());
    cx.run_until_parked();
    let active = cx.read_window(&window, |view, cx| view.read(cx).active_tab).unwrap();
    assert_eq!(active, Tab::Forms);

    // scroll
    cx.simulate_event(window.into(), ScrollWheelEvent { position: .., delta: .., modifiers: .., touch_phase: .. });

    // optional screenshot
    let img: image::RgbaImage = cx.capture_screenshot(window.into()).unwrap();
    img.save("/tmp/gallery.png").unwrap();
}
```
Prefer **entity-state assertions** (`read_window`/`read_entity`) as primary check; screenshot is a secondary/manual-debug aid, not for pixel-diff assertions unless a golden-image system is added (none exists here).

## 4. Where to put it / how to run

- Put new visual tests in `examples/ui_gallery/tests/` (integration test binary) if testing the assembled `GalleryApp`, or `crates/ui/src/components/<component>/tests.rs` (unit, in-crate) if testing one component in isolation — mirror `data_table/tests.rs` location convention for component-local tests.
- `examples/ui_gallery/Cargo.toml` — verify `[dev-dependencies] gpui = { features = ["test-support"] }` is present (confirmed present in `crates/ui/Cargo.toml:36`, not yet checked for `examples/ui_gallery/Cargo.toml` — **open question**).
- Feature: `gpui`'s `test-support` cargo feature must be enabled (gates `capture_screenshot`, per visual_test_context.rs:383).
- Run: `cargo test -p ui_gallery -- --ignored` (or whatever the example crate's package name is) since real-render tests should be `#[ignore]`-by-default like the `gpui_platform` precedent, run explicitly.
- **macOS/Metal only** — `current_platform` only wires a real platform under `#[cfg(target_os = "macos")]`; on Linux/Windows CI this harness cannot run as-is.

## Open questions
- Exact `#[ignore]` reason text / full test bodies in `crates/gpui_platform/src/gpui_platform.rs` not read — verify before modeling new tests on it.
- Content of `crates/ui/src/components/context_menu.rs` `#[gpui::test]` not read — confirm it's a `TestAppContext` click-simulation example.
- Whether `examples/ui_gallery/Cargo.toml` has `gpui` dev-dep with `test-support` feature — unchecked.
- No dedicated `simulate_scroll_wheel` helper exists; confirm `ScrollWheelEvent` field names/`ScrollDelta` construction from `crates/gpui/src/window.rs` before using `simulate_event`.
- Whether CI runs on macOS runners with GPU/Metal access at all (needed for `capture_screenshot`/offscreen compositor render) — unverified, likely no if CI is Linux-only.

## Trade-offs
- `VisualTestAppContext` (real render): true screenshot capability, but macOS-only, GPU-dependent, slower, `#[ignore]`-by-default precedent in this repo — best for occasional visual/manual verification, not routine CI gating.
- `TestAppContext` (mocked `TestPlatform`): same click/keystroke/scroll API, no real screenshots, faster and the established pattern (`#[gpui::test]`) — best for asserting UI logic/state on every commit; use for the bulk of "does click change state" tests, reserve `VisualTestAppContext` only where an actual pixel screenshot is the point.
