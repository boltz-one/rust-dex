# Phase 04 — Data Display & Lists (DataTable restyle, List restyle, Description List, Stats Card, Media Object, Empty State)

## Context Links

- Research: `researcher-01-tailwind-appui-catalog.md` (Data Display + Lists rows: Description Lists, Stats Cards, Stacked List, Tables, Grid List, Feeds, Empty States)
- Research: `researcher-02-codebase-audit.md` (DataTable ⬜ pending, List 🟡 partial)
- Phase 01: `./phase-01-gap-analysis-icons.md`
- Plan: `./plan.md` (Cross-Cutting Requirements)

## Overview

- Date: 2026-07-03
- Description: Restyle DataTable and List (= Stacked List / List Container) to token spec; build net-new Description List, Stats Card, Media Object, Empty State.
- Priority: P2
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- `list.rs` + `list/{list_item,list_header,list_sub_header,list_separator,list_bullet_item}.rs` ALREADY cover Tailwind's "Stacked List" and "List Container" categories — this is a restyle, not new components. Do not build a separate `stacked_list.rs`.
- `data_table.rs` + `data_table/table_row.rs` exist with a `tests.rs` already present — restyle only (header/row/striped/hover colors), do not restructure the table container API.
- `Card` (from prior plan, done) is the base for Stats Card — Stats Card = `Card` + a specific metric/label layout, not a new container primitive.
- "Grid List" (card-based grid) from researcher-01 is just a caller-composed layout of existing `Card`s in a `h_flex()`/grid wrap — no new component needed, note this in the Locked Scope Matrix as "covered by existing Card + caller layout", not a gap.
- "Feeds" (timeline/activity log) is genuinely new — small component, border-left timeline + avatar + text per entry.

## Requirements

### Reuse Map

| Tailwind category | GPUI base | Action |
|---|---|---|
| Tables | `components/data_table.rs` + `data_table/table_row.rs` | RESTYLE |
| Stacked List / List Container | `components/list.rs` + `components/list/*.rs` | RESTYLE |
| Grid List | `components/card.rs` (caller-composed) | NONE — document as covered |
| Description Lists | none | NEW `components/description_list.rs` |
| Stats Cards | `components/card.rs` | NEW `components/stats_card.rs` (thin wrapper) |
| Media Objects | none | NEW `components/media_object.rs` |
| Empty States | none | NEW `components/empty_state.rs` |
| Feeds | none | NEW `components/feed.rs` |

### Functional

- **DataTable restyle**: header `semantic::elevated_surface` bg + `border_muted` bottom + font-semibold text-sm; rows `border_muted` bottom, py-3 px-4, text-sm; striped variant alternates `surface`/`elevated_surface`; hover variant → `semantic::hover_bg`. Confirm sorting/selection/pagination hooks (if present) still work post-restyle.
- **List restyle**: `list_item.rs` row padding/hover per Tailwind stacked-list spec (`border_muted` divider, `hover_bg` on interactive items); `list_header`/`list_sub_header` typography via existing `typography.rs` scale; `list_separator` uses `semantic::border_muted`.
- **Description List** (new): key-value rows, stacked (mobile-style: label above value) or horizontal (label left, value right) mode — `border_muted` top divider between rows, py-4.
- **Stats Card** (new): `Card` base + big metric number (large bold text) + label (text_muted) + optional trend indicator (up/down arrow icon + `palette::success/danger`). Caller composes multiple into a grid.
- **Media Object** (new): flex row, image/avatar left + text block right, gap-4, items-start.
- **Empty State** (new): centered column, icon (48px, `semantic::text_muted`), heading, description, optional action `Button` — py-12.
- **Feed** (new): vertical timeline, `border-l` `semantic::border_muted` connecting line, each entry = avatar + text + timestamp, mb-4 spacing.

### Non-functional

- Files under 200 lines; DataTable/List restyles are edits in place (no size growth beyond current).

## Architecture

```
crates/ui/src/components/
├── data_table.rs              (MODIFY)
├── data_table/table_row.rs     (MODIFY)
├── list.rs                      (MODIFY)
├── list/*.rs                     (MODIFY as needed)
├── description_list.rs            (NEW)
├── stats_card.rs                   (NEW)
├── media_object.rs                  (NEW)
├── empty_state.rs                    (NEW)
└── feed.rs                            (NEW)
```

## Related Code Files

**Read first:** `data_table.rs` (confirm sort/selection/pagination hook shape before restyling), `list/list_item.rs`.

**Modify:** `data_table.rs`, `data_table/table_row.rs`, `list.rs`, `list/*.rs`, `crates/ui/src/components.rs`, `crates/ui/src/prelude.rs`.

**Create:** `description_list.rs`, `stats_card.rs`, `media_object.rs`, `empty_state.rs`, `feed.rs`.

## Implementation Steps

1. Read `data_table.rs` + `table_row.rs` fully (including `tests.rs`) — confirm existing hooks before restyling.
2. Restyle DataTable header/rows/striped/hover.
3. Restyle List + list/* sub-components.
4. Build DescriptionList (stacked + horizontal modes).
5. Build StatsCard (Card + metric/label/trend).
6. Build MediaObject.
7. Build EmptyState.
8. Build Feed.
9. Update/add `preview()` for all 8 deliverables.
10. `cargo check -p ui` clean; run existing `data_table` tests (`cargo test -p ui data_table`) to confirm no regression.

## Todo List

- [ ] Read data_table.rs + table_row.rs + tests.rs
- [ ] Restyle DataTable
- [ ] Restyle List + list/* sub-components
- [ ] Build DescriptionList
- [ ] Build StatsCard
- [ ] Build MediaObject
- [ ] Build EmptyState
- [ ] Build Feed
- [ ] preview() for all 8
- [ ] cargo check -p ui clean + existing data_table tests pass

## Success Criteria

- `make check` + `make check-all` + `cargo fmt --all --check` green.
- `cargo test -p ui` (data_table tests) still passes post-restyle.
- `cargo run -p ui_gallery` — Data page (wired Phase 9) shows all 8 deliverables without panic.
- Grid List documented in Locked Scope Matrix as "covered, no new component" (not silently dropped).

## Risk Assessment

- **Risk:** DataTable restyle could break existing sort/selection callback wiring if colors are tied to state classes in a way not immediately obvious. **Mitigation:** step 1 read-first + run existing tests before AND after restyle.
- **Risk:** List restyle touches 5 sub-files (`list_item`, `list_header`, `list_sub_header`, `list_separator`, `list_bullet_item`) — risk of inconsistent token application across them. **Mitigation:** restyle all 5 in the same pass, cross-check spacing/color consistency at the end.

## Security Considerations

None — presentational/data-display components, no data fetching in this layer.

## Next Steps

- Phase 9 gallery needs a "Data" page wiring this phase's 8 deliverables.
- Phase 7's "Card polish" is independent of this phase's StatsCard (StatsCard uses Card's CURRENT stable API, doesn't need to wait for Phase 7's polish pass).
