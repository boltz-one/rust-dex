# Goal: ui_gallery — fix interaction bugs + offscreen test harness + Tailwind expansion

## Mission
Fix 6 confirmed interaction bugs in `examples/ui_gallery`, build a GPUI `VisualTestAppContext` offscreen "computer-use" test harness that proves each fix, then enrich the gallery's Tailwind coverage (variants, composed pages, missing components). Done = `make check` + `cargo fmt --all --check` green AND `cargo test -p ui_gallery -- --ignored` passes on macOS with a test per bug.

## Context & Key Files
- Full plan: `plans/20260703-1848-ui-gallery-fix-expand-harness/plan.md`
- Phases: `phase-01-investigation-harness-foundation.md` (BLOCKS 2-6) → `phase-02-fix-interactive-bugs.md` → `phase-03-enrich-showcase-variants.md` → `phase-04-composed-page-examples.md` → `phase-05-missing-tailwind-variants.md` → `phase-06-harness-regression-final-verify.md`
- Research: `research/researcher-01-bug-rootcause.md` (bug matrix), `research/researcher-02-visualtest-harness.md` (harness API)
- Code: `examples/ui_gallery/src/{gallery_app.rs,main.rs,pages/*.rs}`, `crates/ui/src/components/text_input.rs`, `crates/gpui/src/app/visual_test_context.rs`

## Requirements
**Must do (order matters — Phase 1 first):**
- P1: build offscreen harness in `examples/ui_gallery/tests/`, `#[cfg(target_os="macos")]` + `#[ignore]`; add dev-dep `gpui = { workspace = true, features = ["test-support"] }`.
- P2: fix 6 bugs, each with a harness test proving it:
  1. [component] `text_input.rs` — add `.on_mouse_down`→`window.focus(&focus_handle)` (non-breaking; fixes Combobox/SearchInput too).
  2. [wiring] scroll — store `ScrollHandle` on `GalleryApp`, wrap content `.id().overflow_y_scroll().track_scroll(&self.scroll)`.
  3. [wiring] SegmentedControl — wire `.on_change` + `forms_segment: usize` state on `GalleryApp`.
  4-6. [wiring] MultiSelect/Combobox/SearchInput — persist their `Entity` on `GalleryApp`, never `cx.new(...)` inside a render/preview body.
- Every interactive gallery demo uses a REAL `Entity`/state field on `GalleryApp`.
- Neutrals via `semantic::*(cx)`, accents via `palette::*`; no hardcoded hex/hsla, no brand ids.
- P3-5: enrich per-component variants/states, add composed pages (dashboard/settings/table+toolbar/app-shell), close Tailwind Application-UI gaps (table sort/pagination UI, form validation states, nav variants).

**Must not:**
- Change any `crates/ui` component signature (fixes are additive/non-breaking only).
- Touch `crates/app` or workspace `default-members`.
- Create an entity/state inside a `render`/`preview` function body.

## Success Criteria
- `make check` exits 0; `cargo fmt --all --check` exits 0.
- `cargo test -p ui_gallery -- --ignored` passes on macOS; ≥1 harness test per Phase-2 bug (type→assert input content, click segment→assert active index, toggle multiselect→assert selection Vec, scroll→assert `ScrollHandle` offset changed).
- Default `cargo test -p ui_gallery` (non-ignored) stays green cross-platform.
- `cargo build -p ui_gallery` links; new composed pages + variants reachable.

## Out of Scope
- Golden-image/pixel-diff screenshot assertions (assert state, not pixels).
- Gating default CI on the macOS `--ignored` harness (manual/opt-in only).
- `crates/app` changes; Tailwind Marketing/Ecommerce blocks.
- Phase 8 backlog components from the prior plan.

## Verification
```bash
make check && cargo fmt --all --check
cargo test -p ui_gallery            # non-ignored, cross-platform, green
cargo test -p ui_gallery -- --ignored   # macOS: harness bug/regression tests
cargo run -p ui_gallery             # optional manual visual click-through
```
