# Phase 1 — Investigation + Test Harness Foundation

## Context links

- `plans/20260703-1848-ui-gallery-fix-expand-harness/research/researcher-01-bug-rootcause.md`
- `plans/20260703-1848-ui-gallery-fix-expand-harness/research/researcher-02-visualtest-harness.md`
- Plan-level "Key Codebase Facts" in `./plan.md` (bug matrix already finalized there — this
  phase's investigation work is DONE, listed below for traceability, not to be redone)

## Overview

Two things land in this phase: (a) the bug matrix is now FINAL (verified below, no more
"unread pages" risk), (b) a working offscreen `VisualTestAppContext` harness scaffold exists
in `examples/ui_gallery/tests/` with one smoke test that opens `GalleryApp` offscreen and
captures a screenshot. This phase BLOCKS all others — Phase 2 needs the harness to prove each
bug fix, Phases 3-6 need it for regression coverage.

## Key Insights

- Bug matrix closed: grepped every `preview`/`*_preview` fn body under
  `crates/ui/src/components/**/*.rs` for `cx.new(` — only 3 files hit
  (`multi_select.rs`, `combobox.rs`, `search_input.rs`). Read `pages/data.rs`, `feedback.rs`,
  `navigation.rs`, `elements.rs`, `layout.rs` in full — all stateless free `fn render` calling
  only `Component::preview()`. No hidden 7th bug.
- TextInput focus bug confirmed at GPUI level: `crates/gpui/src/elements/div.rs:696`
  `track_focus()` does not auto-focus on click — explicit `window.focus(&handle)` needed.
- `examples/ui_gallery/Cargo.toml` has zero `[dev-dependencies]` — harness needs one added.
- `GalleryApp::new(cx: &mut Context<Self>) -> Self` — no `window` param (differs from
  researcher-02's sketch; use the real signature).
- Only known in-repo `VisualTestAppContext` usage: `crates/gpui_platform/src/gpui_platform.rs`
  tests module (`#[ignore]`-by-default, exact reason/body not yet read — read it in this phase
  before modeling the new test on it).
- `crates/ui/src/components/context_menu.rs` has a `#[gpui::test]` — likely the best in-repo
  model for `TestAppContext`-based click-driven assertions; read it in this phase too.

## Requirements

1. Read `crates/gpui_platform/src/gpui_platform.rs` test module in full + `context_menu.rs`'s
   `#[gpui::test]` in full — confirm exact harness construction pattern and any gotchas
   (asset source requirement, `run_until_parked` timing, etc).
2. Add `[dev-dependencies]` to `examples/ui_gallery/Cargo.toml`:
   `gpui = { workspace = true, features = ["test-support"] }` (mirror `crates/ui/Cargo.toml:36`).
3. Create `examples/ui_gallery/tests/visual_harness.rs` (integration test binary) with ONE
   `#[test] #[ignore]` smoke test: open `GalleryApp` via `open_offscreen_window`, assert the
   window opened (`Result::is_ok()`), call `capture_screenshot` and assert non-empty image
   dimensions (no golden-image diff — none exists in repo, out of scope).
4. `ScrollWheelEvent` shape ALREADY CONFIRMED (see plan.md Resolved Decisions):
   `ScrollWheelEvent { position: Point<Pixels>, delta: ScrollDelta, modifiers: Modifiers,
   touch_phase: TouchPhase }` at `crates/gpui/src/interactive.rs:428`; scroll uses a
   `ScrollHandle` + `div().overflow_y_scroll().track_scroll(&handle)`. No re-derivation needed —
   Phase 2/6 assert scroll via `ScrollHandle` offset (state), not pixels.
5. Document macOS/Metal-only limitation directly in the test file's module doc comment and in
   this phase's Success Criteria (no CI enforcement assumed).

## Architecture

Low-risk test-infra addition, no ADR needed — it's a new `#[ignore]`-gated integration test
crate-dev-dependency, zero production code path affected, additive-only Cargo.toml change.

Harness shape:
```
examples/ui_gallery/tests/visual_harness.rs
  mod support {
      // helper: fn open_gallery(cx: &mut VisualTestAppContext) -> WindowHandle<GalleryApp>
  }
  #[test]
  #[ignore] // real macOS render; run explicitly: cargo test -p ui_gallery -- --ignored
  fn smoke_offscreen_gallery_renders() { ... }
```

## Related code files

- `examples/ui_gallery/Cargo.toml` (add dev-dependency)
- `examples/ui_gallery/tests/visual_harness.rs` (new)
- `crates/gpui/src/app/visual_test_context.rs` (read-only reference)
- `crates/gpui_platform/src/gpui_platform.rs` (read-only reference, existing usage pattern)
- `crates/ui/src/components/context_menu.rs` (read-only reference, `TestAppContext` pattern)
- `crates/gpui/src/window.rs` (read-only, confirm `ScrollWheelEvent` shape)

## Implementation Steps

1. Read `crates/gpui_platform/src/gpui_platform.rs` tests module fully; note exact
   `VisualTestAppContext::new(...)` call, `#[ignore]` reason text, any setup/teardown quirks.
2. Read `crates/ui/src/components/context_menu.rs`'s `#[gpui::test]` fully; note the
   `TestAppContext` click/keystroke pattern used for asserting component state.
3. Read `crates/gpui/src/window.rs` around `ScrollWheelEvent` definition; record field names +
   how to build `ScrollDelta` (pixel vs line) for later scroll tests.
4. Add `[dev-dependencies]` block to `examples/ui_gallery/Cargo.toml`.
5. Write `examples/ui_gallery/tests/visual_harness.rs` with the smoke test per Requirements #3.
   Use `gpui_platform::current_platform(true)` (headless) per researcher-02's sketch, corrected
   for the real `GalleryApp::new(cx)` signature.
6. Run `cargo test -p ui_gallery -- --ignored` locally (macOS) to confirm the smoke test
   compiles and passes; if it fails on asset loading (icons/fonts), use
   `VisualTestAppContext::with_asset_source` with the gallery's actual asset source (check
   `main.rs`'s `icons::Assets` usage) — do not skip icons silently.

## Todo

- [ ] Read `gpui_platform.rs` test module + `context_menu.rs` `#[gpui::test]`
- [ ] Read `window.rs` `ScrollWheelEvent` definition, record shape
- [ ] Add dev-dependency to `examples/ui_gallery/Cargo.toml`
- [ ] Write `visual_harness.rs` smoke test
- [ ] Run `cargo test -p ui_gallery -- --ignored` green locally
- [ ] Run `cargo check -p ui_gallery` (non-ignored path unaffected) + `cargo fmt --all --check`

## Success Criteria

- `cargo test -p ui_gallery -- --ignored` opens the gallery offscreen and captures a screenshot
  without panicking, on macOS.
- `ScrollWheelEvent` construction is documented (in this file or a code comment) for Phase 2/6.
- `make check` still green (harness is dev-only, doesn't touch the default build path).

## Risk Assessment

- Real-render harness may be flaky/slow on CI or non-Metal environments — mitigated by
  `#[ignore]`-by-default (matches existing repo precedent), never gates default `make check`.
- Asset loading (fonts/icons) inside the offscreen window may need explicit `with_asset_source`
  — address in step 6, don't silently accept a blank/icon-less render as "done."

## Security Considerations

N/A — test-only code, no user input, no network/filesystem writes beyond an optional debug
screenshot path (keep any screenshot writes inside the OS temp dir, not the repo).

## Next steps

Phase 2 uses this harness to write one test per confirmed bug before/while fixing it.
