# Phase 6 — Harness Regression Coverage + Final Verify

## Context links

- `./phase-01-investigation-harness-foundation.md` through `./phase-05-missing-tailwind-variants.md`
- `./plan.md` Verify section

## Overview

Closes the plan: extends `examples/ui_gallery/tests/visual_harness.rs` to cover every
interactive surface introduced across Phases 2-5 (not just the original 6 bugs), then runs the
full verify sequence (`make check`, `cargo fmt`, harness suite, manual `cargo run` confirmation).

## Key Insights

- Phase 2 already added one harness test per bug; Phase 4 added one for the Examples page's
  live filter. This phase's job is coverage completeness, not re-inventing the harness.
- Reserve `VisualTestAppContext` (real render, screenshot) for cases where a pixel/entity-state
  check genuinely needs the real compositor path; prefer the faster `TestAppContext`
  (`#[gpui::test]`, mocked platform) for pure click/state assertions per researcher-02's
  trade-off note — don't force every new test through the slow offscreen path if a
  `TestAppContext` test proves the same thing.
- Final verify is the plan's actual "done" gate — nothing in Phases 1-5 is complete until this
  phase's full-suite run is green.

## Requirements

1. Inventory every new interactive control added in Phases 2-5 (SegmentedControl in Examples
   page if reused, Table sort click, form validation triggers, filter inputs, etc) and confirm
   each has at least one assertion (harness or `TestAppContext`) — not just "renders without
   panic," but "state changes correctly on interaction" for anything stateful.
2. Run full verify: `make check`, `make check-all` if it exists, `cargo fmt --all --check`,
   `cargo test -p ui_gallery -- --ignored` (full harness suite), `cargo test --workspace`
   (non-ignored suite, should be unaffected).
3. Manual final pass: `cargo run -p ui_gallery`, walk every page, confirm visually (user-facing
   sign-off step per the plan's Verify section — optional per user, but recommended given the
   scope of change).

## Architecture

Low-risk, no ADR — test-coverage completion + verification, no new production code paths.

## Related code files

- `examples/ui_gallery/tests/visual_harness.rs` (extended)
- Any `crates/ui/src/components/*/tests.rs` added for `TestAppContext`-based assertions on
  components touched in Phase 5

## Implementation Steps

1. Grep `examples/ui_gallery/src/**` for every `.on_click(`/`.on_change(`/`.on_key_down(` added
   since Phase 1 started (diff against this plan's starting commit) — cross-check each has test
   coverage.
2. Fill any coverage gap found with the cheapest sufficient test (`TestAppContext` preferred
   over `VisualTestAppContext` unless a screenshot is specifically the point).
3. Run `make check` (and `make check-all` if present in the `Makefile` — verify it exists
   before assuming).
4. Run `cargo fmt --all --check`.
5. Run `cargo test -p ui_gallery -- --ignored` — full harness suite green.
6. Run `cargo test --workspace` — confirm no regression to existing non-ignored tests.
7. `cargo run -p ui_gallery` — manual walkthrough of all pages (Elements/Forms/Feedback/
   Navigation/Data/Overlays/Layout/Examples), confirm no panic, no visual regression, all
   Phase 2 fixes hold up interactively.

## Todo

- [ ] Coverage inventory + gap-fill for Phases 2-5 interactive additions
- [ ] `make check` green
- [ ] `cargo fmt --all --check` green
- [ ] `cargo test -p ui_gallery -- --ignored` green (full harness suite)
- [ ] `cargo test --workspace` green (no regression)
- [ ] Manual `cargo run -p ui_gallery` walkthrough (user-facing sign-off, optional per plan)

## Success Criteria

- Every interactive control added across this plan (Phases 2-5) has at least one passing test
  proving its state-change behavior, not just render-without-panic.
- Full verify sequence green end to end.
- User has an opportunity to visually confirm via `cargo run -p ui_gallery` before the plan is
  marked complete.

## Risk Assessment

- Harness suite runtime could grow noticeably as tests accumulate (real-render tests are slow)
  — acceptable since it's `#[ignore]`-gated, run explicitly, not part of default `make check`.
- If CI has no macOS/Metal runner, this suite stays a local/manual gate only — flagged as an
  open question at the plan level, doesn't block this phase's local completion.

## Security Considerations

N/A — test/verification phase only.

## Next steps

Plan complete. Any deferred items (Phase 5's explicitly-out-of-bounds Phase-8 categories, or
any gap intentionally deferred with a documented reason) go to backlog, not silently dropped.
