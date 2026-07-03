---
title: "ui_gallery: fix interaction bugs + offscreen test harness + Tailwind expansion"
description: "Fix 6 confirmed interaction bugs in ui_gallery, build a GPUI offscreen (computer-use) test harness to prove fixes, then enrich the gallery's Tailwind coverage"
status: pending
priority: P2
effort: 22h
branch: feat/tailwind-app-ui-parity
tags: [frontend, ui, gpui, testing, gallery, bugfix]
created: 2026-07-03
lane: frontend/ui + test-infra, low-risk
---

# ui_gallery: Fix + Offscreen Harness + Tailwind Expansion

## Overview

`examples/ui_gallery` has 6 confirmed interaction bugs (1 component-level, 5 gallery-wiring)
found by root-cause research, and no automated way to prove a fix beyond manual `cargo run`.
This plan: (1) builds a `VisualTestAppContext`-based offscreen "computer-use" test harness
(macOS-only, real render, click/type/scroll simulation + screenshot), (2) fixes the 6 bugs
with a harness test proving each, (3) enriches every component's showcase with more
variants/states, (4) adds full composed-page examples, (5) closes remaining Tailwind
Application-UI gaps, (6) expands harness regression coverage + final verify.

## Phases

| # | Phase | Status | Effort | Link |
|---|-------|--------|--------|------|
| 1 | Investigation + harness foundation (BLOCKS 2-6) | ⬜ | 4h | [phase-01](./phase-01-investigation-harness-foundation.md) |
| 2 | Fix interactive bugs (6 bugs, each harness-proven) | ⬜ | 3h | [phase-02](./phase-02-fix-interactive-bugs.md) |
| 3 | Enrich showcase per component (variants/sizes/states) | ⬜ | 4h | [phase-03](./phase-03-enrich-showcase-variants.md) |
| 4 | Full-page composed examples (dashboard/settings/table/app-shell) | ⬜ | 4h | [phase-04](./phase-04-composed-page-examples.md) |
| 5 | Missing Tailwind variants/components vs catalog | ⬜ | 4h | [phase-05](./phase-05-missing-tailwind-variants.md) |
| 6 | Harness regression coverage + final verify | ⬜ | 3h | [phase-06](./phase-06-harness-regression-final-verify.md) |

## Dependencies

- Phase 1 BLOCKS 2-6 (harness + final bug matrix needed by all).
- Phase 2 before 3-5 (don't enrich a showcase on top of broken state-holding code).
- Phases 3-5 are content-additive, low file-conflict, but run sequentially (shared
  `gallery_app.rs` struct edits) rather than declared parallel-safe.
- Phase 6 depends on all prior phases' interactions/components existing.

## Cross-Cutting (apply to every phase)

- Generic `semantic::*` (neutral) / `palette::*` (accent) tokens only — no hardcoded hex/hsla,
  no brand ids.
- `crates/ui/src/components/*.rs` fixes must be **non-breaking** (additive only — new handler/
  builder method, never a signature change).
- Do NOT touch `crates/app` or workspace `default-members`. Gallery builds via
  `cargo run -p ui_gallery` only.
- Every interactive gallery demo uses a REAL `Entity`/state field on `GalleryApp` — never an
  entity/state created inside a `render`/`preview` function body.
- Verify per phase: `make check` + `cargo fmt --all --check` green; harness test(s) for
  anything touched pass via `cargo test -p ui_gallery -- --ignored` (macOS).

## Key Codebase Facts (verified, do not re-derive — details in phase-01/02)

- **Bug matrix FINAL**: grepped every `preview`/`*_preview` fn body in
  `crates/ui/src/components/**/*.rs` for `cx.new(` — only 3 hits (`multi_select.rs`,
  `combobox.rs`, `search_input.rs`). All other pages (`data/feedback/navigation/elements/
  layout.rs`) are stateless free `fn render` — no hidden 7th bug.
- **TextInput focus bug confirmed at GPUI level**: `crates/gpui/src/elements/div.rs:696`
  `track_focus()` does not auto-focus on click — explicit `window.focus(&handle)` required,
  missing from `crates/ui/src/components/text_input.rs`.
- `GalleryApp::new(cx: &mut Context<Self>) -> Self` (no `window` param).
- `examples/ui_gallery/Cargo.toml` has no `[dev-dependencies]` — add
  `gpui = { workspace = true, features = ["test-support"] }`.
- SegmentedControl component itself is correct; Forms page's `::preview()` call just never
  wires `.on_change` — gallery-wiring bug, not component bug. `TabBar`/`Tab` unused in gallery.
- Full bug list/file:line/fix direction: `research/researcher-01-bug-rootcause.md`.
  Harness API reference: `research/researcher-02-visualtest-harness.md`.
  Tailwind catalog (Phase 5): `plans/20260703-0001-tailwind-app-ui-complete/research/
  researcher-01-tailwind-appui-catalog.md` + its plan (~42 components already delivered).

## Resolved Decisions (2026-07-03, best-practice)

- ✅ **Scroll test = assert `ScrollHandle` offset (state), not pixels.** Verified in code:
  `ScrollWheelEvent { position: Point<Pixels>, delta: ScrollDelta, modifiers, touch_phase }`
  (`crates/gpui/src/interactive.rs:428`); `div().overflow_y_scroll().track_scroll(&handle)`
  (`div.rs:1198/1204`), `ScrollHandle::new()` (`div.rs:3394`). **Fix + test approach:** store a
  `ScrollHandle` on `GalleryApp`, wire `.id().overflow_y_scroll().track_scroll(&handle)` on each
  page's content wrapper; the harness scroll test dispatches a
  `ScrollWheelEvent { delta: ScrollDelta::Pixels(..), .. }` then asserts the `ScrollHandle`
  offset changed — deterministic state assert, no screenshot pixel diffing. `ScrollWheelEvent`
  shape is now known → no Phase-1 unknown remains.
- ✅ **Harness CI policy = macOS-gated + `#[ignore]`-by-default, never gates default CI.**
  All harness tests are `#[cfg(target_os = "macos")]` + `#[ignore]` (matches the existing
  `gpui_platform` precedent), run explicitly via `cargo test -p ui_gallery -- --ignored`
  locally/manually. Default `make check` + `cargo test` stay green cross-platform (harness
  skipped). A macOS CI lane MAY opt-in to `--ignored`; if none exists, manual-only is accepted
  and does not block. Building the harness proceeds regardless.
- Note: reading `gpui_platform`'s `#[ignore]` reason text / `context_menu.rs` test body in
  Phase 1 is now a nicety, not a blocker — the pattern (macOS cfg + `#[ignore]`) is already
  decided above.
