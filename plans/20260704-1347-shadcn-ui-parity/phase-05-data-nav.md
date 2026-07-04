---
title: "Phase 5 — Data & Navigation"
status: pending
effort: 8h
---

# Phase 5: Data & Navigation

[← plan.md](./plan.md) | Prev: [phase-04](./phase-04-overlays.md) | Next: [phase-06](./phase-06-advanced-heavy.md)

## Context
shadcn "Data & Navigation" + "Layout" categories minus Card (done in Phase 2). Mostly alignment of already-solid existing components (Tabs/Breadcrumb/Pagination/Sidebar/Progress), plus one investigate-first item (Direction/RTL) whose feasibility is genuinely unverified per research.

## Component Table

| Component | Codebase file | Action | Notes |
|---|---|---|---|
| Tabs | `tab.rs` / `tab_bar.rs` | Align | Verify Root/List/Trigger/Content anatomy + active/disabled states |
| Breadcrumb | `breadcrumb.rs` | Align | Verify Root/List/Item/Link/Separator/Ellipsis anatomy |
| Pagination | `pagination.rs` | Align | Verify Root/Content/Item/Link/Prev/Next/Ellipsis anatomy |
| Navigation Menu | none | New | Root/List/Item/Trigger/Content/Link/Viewport/Indicator; animated viewport + hover-intent timers — closest existing pattern is `navbar.rs`, but this needs its own file given the viewport-transition behavior is distinct from a static navbar |
| Accordion | `disclosure.rs` (split) | Align | shadcn splits Accordion (single/multiple exclusive-open semantics) from Collapsible (single open/close) — verify `disclosure.rs`'s current API already distinguishes these or needs a thin `Accordion` wrapper enforcing single/multiple-open group behavior |
| Collapsible | `disclosure.rs` | Align | The simpler, ungrouped open/closed primitive — verify it's usable standalone (not only via Accordion's grouping) |
| Table | `data_table.rs` (new lightweight sibling) | New | shadcn has a plain static `Table` (Root/Header/Body/Footer/Row/Head/Cell/Caption) distinct from the sortable/filterable Data Table — add a lightweight `table.rs` for the static case rather than forcing every static-table use through `data_table.rs`'s heavier machinery (YAGNI in reverse: don't make simple callers pay for sort/filter they don't need) |
| Data Table | `data_table.rs` | Align/Extend | Verify sorted/filtered/selected state hooks exist; TanStack-parity sort/filter/pagination logic must be reimplemented in Rust if missing — scope to the states shadcn's own Data Table demo covers (single-column sort, text filter, row selection, pagination), not a generic query-builder |
| Scroll Area | `scrollbar.rs` | Align | GPUI has native scroll already — this is mostly a styled wrapper region (Root/Viewport/Scrollbar/Thumb/Corner) around existing scroll, not new scroll physics |
| Sidebar | `sidebar.rs` | Align (verify only) | Already the most complex composite, already ported per research — just confirm Provider/Sidebar/Header/Content/Footer/Group/Menu/Trigger/Rail anatomy is complete, likely near-zero new code |
| Resizable | none | New | GPUI already does flex layout + can track pointer-drag deltas on a divider — simpler than the web version (no DOM reflow cost); Root/Panel/Handle anatomy |
| Direction (RTL/LTR) | none | Investigate first | GPUI text/layout RTL/bidi support is **unverified** (research open question) — this is a genuine "missing information" case: spend up to 1h confirming GPUI's actual bidi capability (check `crates/gpui` text layout internals/docs) before deciding build-vs-document-limitation; if GPUI has no bidi support, document that as the concrete blocker (not a vague "too hard") and skip the Provider component |

## Key Insights
- Sidebar needs the least work in this whole plan — research already calls it "most complex composite, already ported."
- Table vs. Data Table is a real architectural fork shadcn itself makes — don't conflate them into one file just because `data_table.rs` exists (violates KISS for simple callers who don't need sort/filter).
- Direction/RTL is the only item in this phase gated on missing information rather than effort — resolve the GPUI-capability question first, cheaply, before committing build time.

## Requirements
- Accordion vs Collapsible: the distinction that must survive alignment is group-exclusive-open (Accordion, with single/multiple mode) vs. standalone open/close (Collapsible) — don't merge them into one API that loses this shadcn-meaningful difference.
- Data Table: sort/filter/selection/pagination state must be plain Rust structs/enums driving `data_table.rs`'s render, not an attempt to port TanStack's generic column-def system wholesale.
- Resizable: divider drag must respect min/max panel size constraints (shadcn's `react-resizable-panels` supports this) — clamp on drag, not just after release.

## Architecture
- `table.rs`: new, minimal — Root/Header/Body/Footer/Row/Head/Cell/Caption as thin styled `div()` wrappers, no state beyond what's passed in (fully controlled, mirrors shadcn's own plain-Table simplicity).
- `navigation_menu.rs`: new, viewport-transition state (which submenu's content is shown, animated width/height) — check `animation.rs` for existing transition helpers to reuse before hand-rolling.
- `resizable.rs`: new, divider drag reuses whatever pointer-delta pattern Phase 3's Slider settled on (same math family: clamp a drag delta to a range) — cite Slider's implementation rather than re-deriving.

## Related Files
- `crates/ui/src/components/{tab,tab_bar,breadcrumb,pagination,disclosure,scrollbar,sidebar}.rs`, `data_table/` dir
- New: `crates/ui/src/components/{table,navigation_menu,resizable}.rs`
- `crates/gpui` text-layout internals (read-only investigation for Direction/RTL question)

## Implementation Steps
1. Align Tabs/Breadcrumb/Pagination/Scroll Area/Sidebar/Progress-adjacent items against shadcn anatomy; log gaps.
2. Verify/split Accordion vs Collapsible API in `disclosure.rs`.
3. Build `table.rs` (static, lightweight).
4. Align/extend `data_table.rs` for sort/filter/select/pagination state.
5. Build Navigation Menu (viewport + hover-intent).
6. Build Resizable (divider drag, reuse Slider's drag-math pattern from Phase 3).
7. Spend ≤1h investigating GPUI RTL/bidi capability; document finding; build Direction provider only if GPUI supports it, else document the blocker.
8. Gallery entries (data/navigation/layout pages); `#[gpui::test]` for: Accordion single/multiple-open semantics, Data Table sort/filter/select, Resizable drag-clamp, Navigation Menu hover-intent open.

## Todo
- [ ] Tabs/Breadcrumb/Pagination/Scroll Area/Sidebar aligned
- [ ] Accordion/Collapsible split verified in disclosure.rs
- [ ] Table (new, lightweight) built
- [ ] Data Table sort/filter/select/pagination aligned
- [ ] Navigation Menu built
- [ ] Resizable built
- [ ] Direction/RTL investigated, decision documented (build or documented-skip)
- [ ] Gallery pages updated
- [ ] Harness tests: Accordion, Data Table, Resizable, Navigation Menu
- [ ] `cargo build -p ui` / `cargo test -p ui` clean

## Success Criteria
- 12 of 13 items present + gallery-visible (Direction/RTL either built or documented-skip with a concrete GPUI-capability reason, not a vague deferral).
- Data Table demo covers sort + filter + row-select + pagination together, matching shadcn's own demo scope.
- Resizable divider respects min/max clamps during drag, verified by test.

## Risk & Dependencies
- Depends on Phase 3's Slider drag-math pattern for Resizable (schedule after Phase 3, or duplicate the small clamp helper if run in parallel).
- Risk: Navigation Menu's animated viewport is the highest-effort item — if `animation.rs` lacks a reusable transition helper, this could exceed budget; timebox and fall back to a non-animated (instant-swap) viewport if needed, documented as a known simplification (not silently shipped as "done" with animation implied).

## Security
N/A — presentational/layout components, Data Table filter is local in-memory text match (no query injection surface).

## Next
[phase-06-advanced-heavy.md](./phase-06-advanced-heavy.md)
