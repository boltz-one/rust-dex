# Phase 08 — Deferred/Low-Priority (Calendar/Date Picker, Command Palette, Color Picker, Carousel, Kanban, Virtualized List)

## Context Links

- Research: `researcher-01-tailwind-appui-catalog.md` ("Not Suitable for Desktop" table: Calendars, Command Palettes)
- Research: `researcher-02-codebase-audit.md` ("Tailwind Application UI Components NOT YET IN CRATES/UI" table: Date Picker, Color Picker, Carousel, Kanban, Virtualized List)
- Plan: `./plan.md` (Cross-Cutting Requirements, Open Question 4)

## Overview

- Date: 2026-07-03
- Description: The 6 lowest-value / highest-complexity Tailwind Application UI categories, deliberately sequenced LAST and explicitly separable per user instruction ("có thể cắt" — this bucket may be skipped entirely without affecting Phases 1-7/9's completeness). If executed, each is a FULL, real implementation (no mocks/placeholders) — this phase is about SEQUENCING, not about shipping partial versions.
- Priority: P3 (BACKLOG — CUT from this plan per user decision 2026-07-03, YAGNI)
- Implementation status: 🔵 Backlog — NOT executed in this plan. Do not start without an explicit user go. Kept as a documented backlog spec so the work is ready if the app later needs any of these.
- Review status: Not reviewed

## Key Insights

- Each of these 6 items is a genuinely complex, self-contained component with no shared base in the current codebase — they do not block or get blocked by Phases 2-7, hence safe to defer without stalling the rest of the plan.
- Calendar/Date Picker needs the `Calendar` Heroicon (not vendored in Phase 01, since Phase 01 only vendors icons for in-scope Phase 2-7 components) — if this phase runs, vendor `Calendar` icon as this phase's own first step.
- Command Palette composes `Modal` (Phase 5) + `TextInput` (existing) + `List`/`DropdownMenu` filtering — technically buildable now, but deliberately low-priority per researcher-01 (more relevant to code-editor UX than typical business-app UI).
- Kanban and Virtualized List are the highest-effort items here (drag-drop reordering, viewport-windowed rendering) — if only partial capacity exists, do these two last within this phase.

## Requirements

### Reuse Map

| Tailwind category | GPUI base | Action |
|---|---|---|
| Calendar / Date Picker | none | NEW `components/calendar.rs` + `components/date_picker.rs` |
| Command Palette | `components/modal.rs` + `components/text_input.rs` + `components/list.rs` | NEW `components/command_palette.rs` (composition) |
| Color Picker | none | NEW `components/color_picker.rs` |
| Carousel | none | NEW `components/carousel.rs` |
| Kanban | `components/list.rs` (columns) | NEW `components/kanban.rs` |
| Virtualized List | `components/list.rs` | NEW `components/virtualized_list.rs` (or extend `list.rs` with a windowed-render mode if that's cleaner — decide during implementation) |

### Functional

- **Calendar**: month grid, day cells, prev/next month nav, today/selected-day highlight (`palette::primary`), hover state.
- **Date Picker**: `TextInput`-styled trigger + `Calendar` in a `Popover`/`DropdownMenu` overlay, selecting a day sets the input display value.
- **Command Palette**: `Modal` (centered, top-anchored) containing a `TextInput` (search) + filtered `List` of commands, keyboard up/down navigation + Enter to execute — real keyboard handling required (no mock filtering).
- **Color Picker**: swatch grid (from `palette` role ramps, for in-app theme-adjacent pickers) + optional custom hex input (`TextInput`).
- **Carousel**: horizontal item strip with prev/next controls (reuse `IconButton`) + optional dot pagination (reuse Phase 6's `Pagination` visual dots if convenient).
- **Kanban**: multiple columns (each a `List`-like container) with drag-drop reordering — real drag-drop via GPUI's drag/drop APIs (grep `crates/gpui` for existing drag-drop primitives before implementing from scratch).
- **Virtualized List**: viewport-windowed rendering (only render visible rows + a buffer) for large datasets — real windowing logic, not a fixed small-N fake.

### Non-functional

- No mocks: if drag-drop (Kanban) or windowed rendering (Virtualized List) turns out to need capability GPUI doesn't have, STOP and report the gap rather than shipping a fake/visual-only version — this is a "genuinely missing information/dependency" case per the Iron Rules, not a scope-reduction call.

## Architecture

```
crates/ui/src/components/
├── calendar.rs           (NEW)
├── date_picker.rs          (NEW)
├── command_palette.rs        (NEW)
├── color_picker.rs             (NEW)
├── carousel.rs                   (NEW)
├── kanban.rs                       (NEW)
└── virtualized_list.rs               (NEW)
```

## Related Code Files

**Read first:** `crates/gpui` drag-drop APIs (grep `drag` / `Drop` traits) before Kanban; `crates/gpui` scroll/viewport APIs before Virtualized List.

**Create:** all 7 files listed above.

**Modify:** `crates/icons/src/icons.rs` (add `Calendar` icon), `crates/ui/src/components.rs`, `crates/ui/src/prelude.rs`.

## Implementation Steps

1. Confirm go/no-go with user before starting (plan.md Open Question 4).
2. Vendor `Calendar` Heroicon.
3. Build Calendar + Date Picker.
4. Build Command Palette (composes Modal+TextInput+List, real keyboard nav).
5. Build Color Picker.
6. Build Carousel.
7. Grep `crates/gpui` for drag-drop primitives; build Kanban (real drag-drop, or STOP+report if capability missing).
8. Grep `crates/gpui` for viewport/scroll windowing primitives; build Virtualized List (real windowing, or STOP+report if capability missing).
9. `preview()` for all 7.
10. `cargo check -p ui` clean.

## Todo List

- [ ] Confirm go/no-go with user
- [ ] Vendor Calendar icon
- [ ] Build Calendar
- [ ] Build Date Picker
- [ ] Build Command Palette (real keyboard nav)
- [ ] Build Color Picker
- [ ] Build Carousel
- [ ] Grep gpui drag-drop APIs; build Kanban (or report gap)
- [ ] Grep gpui viewport APIs; build Virtualized List (or report gap)
- [ ] preview() for all 7
- [ ] cargo check -p ui clean

## Success Criteria

- `make check` + `make check-all` + `cargo fmt --all --check` green (if phase runs).
- Command Palette: real keyboard up/down/Enter navigation verified in `cargo run -p ui_gallery`.
- Kanban: real drag-drop reordering OR an explicit reported capability gap (not a fake/static version).
- Virtualized List: real windowed rendering verified with a large (1000+) item dataset in the gallery, OR an explicit reported capability gap.
- If phase is skipped entirely (user's "may cut" option exercised), plan.md's Phase 8 row updated to "Cancelled" — Phases 1-7/9 remain fully complete regardless.

## Risk Assessment

- **Risk:** Kanban drag-drop or Virtualized List windowing may hit a genuine GPUI capability gap (missing primitive). **Mitigation:** explicitly scoped as a STOP-and-report condition (not a silent scope cut) — steps 7/8 call this out.
- **Risk:** Given "may cut" framing, this phase could be perpetually deprioritized. **Mitigation:** plan.md Open Question 4 forces an explicit go/no-go decision rather than indefinite limbo.

## Security Considerations

- Command Palette: if ever wired to execute arbitrary app commands, input validation of the selected command is the CALLER's responsibility — this component only handles UI selection, not command execution security (document in file).

## Next Steps

- If this phase is cancelled, Phase 9's final verify still covers Phases 1-7 completely — no dependency from Phase 9 back to this phase.
