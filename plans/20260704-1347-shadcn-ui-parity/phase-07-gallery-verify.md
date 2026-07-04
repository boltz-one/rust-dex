---
title: "Phase 7 — Gallery, harness, final verify"
status: pending
effort: 4h
---

# Phase 7: Gallery, Harness, Final Verify

[← plan.md](./plan.md) | Prev: [phase-06](./phase-06-advanced-heavy.md)

## Context
Closeout phase. Every component touched/added in phases 2-6 must be visible in `examples/ui_gallery` and, where interactive, covered by a `#[gpui::test]` in the visual harness. This phase does a final sweep rather than introducing new component work.

## Key Insights
- Gallery pages already exist per category (`elements.rs`, `forms.rs`, `feedback.rs`, `overlays.rs`, `navigation.rs`, `data.rs`, `layout.rs`, `examples.rs`) — new components slot into the matching existing page, no new page files needed unless a whole new category emerged (e.g. "Advanced" for Phase 6 items — check if `examples.rs` or a new `advanced.rs` page fits better).
- Harness pattern (`examples/ui_gallery/tests/visual_harness.rs`) uses `#[path]`-includes since `ui_gallery` is bin-only — new interactive tests follow the exact same `#[gpui::test]` + `TestAppContext` + `debug_selector`/`simulate_click` pattern already proven there (cite the file's own header comment for why plain `#[test]` + `VisualTestAppContext` is wrong — SIGABRT on macOS/Metal off-main-thread).

## Requirements
- Every net-new component from phases 2-6 appears in a gallery page (visual regression coverage even without an automated test).
- Every component with click/drag/keyboard interaction (Toggle Group, Slider, Input OTP, Combobox, Command, Menubar, Accordion, Data Table, Resizable, Navigation Menu, Sonner, Calendar, Date Picker, Carousel) has at least one `#[gpui::test]` exercising its primary interaction via real rendered bounds (`debug_selector` + `simulate_click`/`simulate_keystrokes`), not just a construction/render smoke test.
- Final gate: `make check`, `cargo fmt --all --check`, `cargo test -p ui`, `cargo test -p ui_gallery` all green; gallery binary builds and launches without panic.

## Architecture
No new architecture — this phase is integration/verification only. If any component needed a `debug_selector` addition to be testable (mirroring the precedent in `Tab`/`SegmentedControl`/`ActionPanel`), add it `#[cfg(any(test, feature = "test-support"))]`-gated, matching existing convention exactly.

## Related Files
- `examples/ui_gallery/src/pages/*.rs`
- `examples/ui_gallery/src/gallery_app.rs`
- `examples/ui_gallery/tests/visual_harness.rs`
- Any `crates/ui/src/components/*.rs` needing a `debug_selector` add for testability

## Implementation Steps
1. Sweep phases 2-6's component list against each gallery page — add any missing entries.
2. Sweep the interaction-required list above against `visual_harness.rs` — add missing `#[gpui::test]` cases.
3. Add `debug_selector` to any new interactive component that needs real-bounds click/keystroke simulation (test-only, no release-build impact).
4. Run full gate: `make check`, `cargo fmt --all --check`, `cargo test -p ui`, `cargo test -p ui_gallery`.
5. Manually launch `cargo run -p ui_gallery` (or equivalent) and visually spot-check each new/aligned page for obvious rendering breakage.
6. Fix any regressions found; re-run gate.

## Todo
- [ ] All phases 2-6 components present in gallery pages
- [ ] All interaction-required components have a passing `#[gpui::test]`
- [ ] `debug_selector` added where needed (test-gated)
- [ ] `make check` green
- [ ] `cargo fmt --all --check` green
- [ ] `cargo test -p ui` green
- [ ] `cargo test -p ui_gallery` green
- [ ] Manual gallery launch spot-check done, no visual regressions

## Success Criteria
- Full gate green.
- Every shipped component (aligned or new) from phases 1-6 is discoverable and interactable in the gallery.
- No component silently missing from gallery coverage.

## Risk & Dependencies
- Depends on all of phases 2-6 landing. If Phase 6's Chart is still pending user decision at this point, gallery/harness coverage for Chart is skipped with a note (not blocking the rest of this phase's gate).

## Security
Final gate is also the last checkpoint for the defensive-input notes flagged in phases 3/4/6 (Input OTP paste, Command filter input, Chart render bounds) — confirm those guards are actually exercised by at least one test, not just present in code.

## Next
None — final phase. Report back to plan.md for overall completion status.
