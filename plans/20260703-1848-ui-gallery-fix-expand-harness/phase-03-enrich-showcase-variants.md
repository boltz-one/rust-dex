# Phase 3 — Enrich Showcase Per Component

## Context links

- `./plan.md` Key Codebase Facts
- `./phase-02-fix-interactive-bugs.md` (state model must be correct first)
- `plans/20260703-0001-tailwind-app-ui-complete/research/researcher-01-tailwind-appui-catalog.md`
  (variant/anatomy reference per category)
- `plans/20260703-0001-tailwind-app-ui-complete/plan.md` (prior plan's component list — this
  phase adds breadth to those same ~60 components, not new categories)

## Overview

Each of the ~60 components in `crates/ui/src/components/` currently shows 1-3 static examples
via `Component::preview()`. Tailwind Plus shows many variants per category (sizes, colors,
states, with/without icon, disabled, loading, etc). This phase widens each page's showcase
depth — same components, more real-data variety — without adding new component files.

## Key Insights

- No new components in this phase — pure showcase breadth. New/missing components are Phase 5.
- Every page is either a plain `fn render(window, cx)` (Elements/Feedback/Navigation/Data/
  Layout) or a `GalleryApp` method (`render_forms`/`render_overlays`) — enrichment means editing
  the `section("X", ...)` calls in `examples/ui_gallery/src/pages/*.rs`, and where a component's
  own `::preview()` is too thin, editing that `preview()` fn in `crates/ui/src/components/*.rs`
  to show more variants (non-breaking — `preview()` has no external callers besides the gallery
  and the component doc registry).
- Prefer widening an existing `preview()` (single source of truth, also improves the component
  doc registry) over duplicating variant code directly in the gallery page.
- Real data only — no lorem-ipsum walls; use plausible domain content (e.g. table rows with
  real-looking names/emails/statuses) matching the existing gallery's tone.

## Requirements

- Per component category (Elements/Forms/Feedback/Navigation/Data/Overlays/Layout), audit
  current `preview()` breadth against the Tailwind catalog's "Core Variants/Anatomy" column and
  add missing variants: sizes (sm/md/lg), color/severity variants (primary/success/warning/
  danger/info), state variants (default/hover-implied/disabled/loading/invalid), with-icon vs
  text-only where applicable.
- Do not change any component's public builder API — only its `preview()` body content and,
  where the gallery page composes multiple `preview()` calls, the page composition.
- Keep each `preview()` under a reasonable size (if it grows past ~40 lines, consider a private
  helper fn in the same file — still no new files needed).

## Architecture

Low-risk, no ADR — purely additive showcase content, no behavior change to any consumer outside
the gallery/doc-registry rendering path.

## Related code files

- `examples/ui_gallery/src/pages/{elements,forms,feedback,navigation,data,overlays,layout}.rs`
- `crates/ui/src/components/*.rs` — widen `preview()` bodies only, component-by-component as
  audited (exact file list depends on Step 1's audit output; expect most of the ~60 files to
  need at least a small addition)
- `examples/ui_gallery/tests/visual_harness.rs` (extend smoke test to still assert basic render
  after content growth — no per-variant assertions needed unless a variant is interactive,
  which is out of scope here per Phase 2's already-covered interactive set)

## Implementation Steps

1. For each page file, list current `section(...)` calls and cross-reference against the
   Tailwind catalog row for that category — note gaps (e.g. Badge missing `dot` variant, Alert
   missing one of the 4 severities, Avatar missing a size).
2. Widen the corresponding component's `preview()` to add the missing variant(s), following
   that file's existing style (grouped `v_flex`/`h_flex` rows, `Label::new` captions where the
   existing pattern already uses them).
3. Re-run `cargo run -p ui_gallery` after each category to visually confirm no layout breakage
   (wrapping row grows too wide, overlaps sidebar, etc) — this is what Phase 2's scroll fix
   makes viewable if a page grows tall.
4. Run harness smoke test (Phase 1) after each category to confirm no panic on render.

## Todo

- [ ] Elements page: audit + widen (Button/Badge/Avatar/Facepile/Chip/Divider/Card variants)
- [ ] Forms page: audit + widen (Checkbox/Switch/Radio/InputGroup/FileInput states)
- [ ] Feedback page: audit + widen (Alert severities/variants)
- [ ] Navigation page: audit + widen (Breadcrumb/Pagination/Stepper states)
- [ ] Data page: audit + widen (Table sort indicator visual, List density, StatsCard variants)
- [ ] Overlays page: audit + widen (Toast severities, Tooltip placements)
- [ ] Layout page: audit + widen (AppShell/Card/Container variants)
- [ ] `make check` + `cargo fmt --all --check` green after each page
- [ ] Harness smoke test still green after full pass

## Success Criteria

- Every category shows at least the Tailwind catalog's documented "Core Variants" (sizes/
  states/severities) that are meaningful for a desktop app.
- No component's public builder signature changed.
- `cargo run -p ui_gallery` renders every page without panic or visible overlap/clipping.

## Risk Assessment

- Widening `preview()` bodies risks visual clutter — keep each addition captioned/grouped
  consistently with the file's existing pattern rather than free-form.
- Some "state" variants (hover, focus) aren't statically renderable — skip those, they're
  already covered by Phase 2's interactive fixes; this phase only adds statically-visible
  variants.

## Security Considerations

N/A — static showcase content only.

## Next steps

Phase 4 composes several of these now-richer components into realistic full-page examples.
