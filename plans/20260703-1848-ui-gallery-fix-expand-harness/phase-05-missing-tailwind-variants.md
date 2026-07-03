# Phase 5 — Missing Tailwind Variants/Components

## Context links

- `plans/20260703-0001-tailwind-app-ui-complete/research/researcher-01-tailwind-appui-catalog.md`
  (full catalog, 31+ categories)
- `plans/20260703-0001-tailwind-app-ui-complete/plan.md` (prior plan's delivered scope — 42
  deliverables across Phases 2-7,9; Phase 8 explicitly CUT to backlog/YAGNI)
- `./phase-03-enrich-showcase-variants.md`, `./phase-04-composed-page-examples.md` (gaps found
  there feed into this phase's checklist)

## Overview

The prior Tailwind-parity plan delivered ~42 components/restyles and explicitly cut 6 advanced
ones (Calendar, Command palette, Color picker, Carousel, Kanban, Virtualized list) as
YAGNI-backlog. This phase re-audits against the full catalog for anything still missing at the
**variant** level (not the cut advanced categories, unless the user explicitly reopens them) —
e.g. Table sort-indicator/pagination-in-table, form validation states (error/success message
under a field), nav variants (pills vs underline tabs), and anything flagged during Phases 3-4.

## Key Insights

- Do NOT reopen Phase 8's cut list (Calendar/Command palette/Color picker/Carousel/Kanban/
  Virtualized list) without explicit user go — stated as CUT/YAGNI in the prior plan, this
  plan's scope is variant-completion, not re-litigating that decision.
- Most catalog categories are already ✓ Core per `researcher-01-tailwind-appui-catalog.md`'s
  matrix — this phase's job is a targeted diff, not a full rebuild.
- Two things belong here specifically per the user's brief: Table sort/pagination UI, form
  validation states, nav variants — treat these as the confirmed starting checklist, then add
  whatever else Step 1's audit finds.
- If a gap is component-level (e.g. Table has no sort-indicator affordance at all), fix in
  `crates/ui/src/components/*.rs` non-breakingly (additive prop/builder method, default
  preserving current behavior). If it's showcase-level only (component supports it, gallery
  doesn't show it), that's actually Phase 3's job — re-route there if found.

## Requirements

1. Diff current `crates/ui/src/components/` against the catalog for: Table column sort
   indicator + clickable header, Table pagination footer (compose with existing `Pagination`
   component, don't rebuild), FormField error/success message + icon state, Tabs pills-vs-
   underline style variant (if `Tab`/`TabBar` need a style prop), Breadcrumb/VerticalNav active-
   state variants not yet covered.
2. For each confirmed gap, add the minimal non-breaking builder addition (e.g.
   `Table::sortable(column)` + an `on_sort` callback, `FormField::error(message)` /
   `FormField::success(message)`), then show it in the relevant gallery page (Data/Forms).
3. Keep additions consistent with the existing component's builder style (see `badge.rs` as the
   reference idiom per project convention).

## Architecture

Low-risk, no ADR — additive builder methods on existing components (default = current
behavior), non-breaking by construction. One line: every new method is opt-in via a builder
call, existing callers unaffected.

## Related code files

- `crates/ui/src/components/data_table.rs`, `table_row.rs`, `pagination.rs` (sort + pagination)
- `crates/ui/src/components/forms.rs` / `form_field.rs` (validation states)
- `crates/ui/src/components/tab.rs`, `tab_bar.rs` (style variant, if gap confirmed)
- `crates/ui/src/components/breadcrumb.rs`, `vertical_nav.rs` (active-state variants, if gap
  confirmed)
- `examples/ui_gallery/src/pages/{data,forms,navigation}.rs` (showcase the new variants)

## Implementation Steps

1. Re-read the catalog matrix row-by-row against current `crates/ui/src/components/` file list;
   produce a short gap list (expect it to be small — most of the catalog is already ✓ Core per
   the prior plan).
2. Implement each confirmed gap as an additive builder method, following the reuse-over-rewrite
   rule (compose with an existing component first — e.g. Table pagination footer = existing
   `Pagination` component placed under `Table`, not a new pagination implementation).
3. Add a `preview()` update (or new `section()` in the relevant page) for each new variant.
4. If any gap is interactive (e.g. sort-by-column click), add gallery wiring following Phase
   2's Entity/state-on-`GalleryApp` rule, plus a harness test.

## Todo

- [ ] Gap-audit catalog vs current components, produce final checklist
- [ ] Table sort indicator + `on_sort` wiring (if confirmed gap) + gallery demo + harness test
- [ ] Table pagination footer composition (if confirmed gap) + gallery demo
- [ ] FormField error/success state (if confirmed gap) + gallery demo
- [ ] Nav variant gaps (if any confirmed) + gallery demo
- [ ] `make check` + `cargo fmt --all --check` green

## Success Criteria

- Checklist from Step 1 fully closed or explicitly deferred with a one-line reason (not silently
  dropped).
- No `crates/ui` public signature broken (additive-only).
- Any new interactive variant has a harness test per Phase 2's pattern.

## Risk Assessment

- Risk of scope creep back into Phase 8's cut categories — explicitly out of bounds here, flag
  to the user rather than implementing if the gap-audit surfaces one of those 6.
- Table sort/pagination composition must not duplicate `Pagination`'s own state management —
  reuse its existing page-index handling rather than re-deriving it inside `Table`.

## Security Considerations

N/A — presentation-layer additions, no new data parsing/validation logic beyond display-level
error/success message rendering.

## Next steps

Phase 6 adds regression harness coverage for everything Phases 3-5 introduced and runs the
final full-suite verify.
