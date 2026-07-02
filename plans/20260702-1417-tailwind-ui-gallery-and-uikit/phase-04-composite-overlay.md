# Phase 04 — Composite / Overlay Components

## Context Links

- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-01-tailwind-spec.md` (§2.6-2.9, 2.13)
- Phase 01: `./phase-01-design-tokens.md` (shadow, focus_ring, palette)
- Existing: `crates/ui/src/components/data_table/`, `modal.rs`, `dropdown_menu.rs`, `tab.rs`, `tab_bar.rs`, `tooltip.rs`, `popover.rs`, `notification/announcement_toast.rs`

## Overview

- Date: 2026-07-02
- Description: Restyle Table (striped/bordered/hover), Modal/Dialog, DropdownMenu, Tabs (underline/pills), Tooltip, Popover, and Toast to Tailwind Application UI patterns.
- Priority: P2
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- **Cross-cutting (see plan.md):** all the neutral values below (bg gray-50, border gray-200, text, hover bg, tooltip bg gray-900) read from `semantic` so dark mode works; only accents from `palette`. Close/dismiss icons use Heroicon `x-mark`; toast type icons use `info`/`check-circle`/`exclamation-triangle`/`x-circle`. Visual-verify BOTH modes (esp. modal backdrop + tooltip contrast in dark).
- All 7 target components ALREADY EXIST — this phase is restyle-only, zero net-new components. Lowest structural risk phase.
- `data_table/table_row.rs` exists but no top-level `Table`/`DataTable` wrapper confirmed yet — check `data_table.rs` (parent file) for the container that composes rows into a full table with header.
- `Modal` already has `ModalHeader`, `ModalRow`, `ModalFooter`, `Section`, `SectionHeader` (rich structure) — restyle is about color/spacing/shadow (`Shadow::Xl` for modal per researcher-01 §2.7), not structural changes.
- `notification/announcement_toast.rs` is the toast base — check if it already supports auto-dismiss timer and stacking; Tailwind spec (researcher-01 §2.13) wants both — if missing, this phase adds them (moderate scope, not a rebuild).
- `Tab`/`TabBar` need two style modes (underline, pills) — check if `TabPosition`/`TabCloseSide` enums already cover a "pills" look or if a new style enum field is needed.

## Requirements

### Functional

**Table** (`crates/ui/src/components/data_table/`):
- Header: bg gray-50, border-b gray-200, font-semibold text-sm.
- Rows: border-b gray-200, py-3 px-4, text-sm; striped variant (alternate bg white/gray-50); hover variant (row hover:bg-gray-100).
- Restyle existing `table_row.rs`; check `data_table.rs` for header/container to restyle too.

**Modal/Dialog** (`crates/ui/src/components/modal.rs`):
- Overlay: bg-black/50 backdrop (check GPUI supports semi-transparent full-window overlay — likely yes via `absolute()` + `inset_0()` + `bg(black().opacity(0.5))`).
- Container: bg surface, rounded-lg, shadow-xl (`Shadow::Xl` from Phase 01), width variants sm(448px)/md(560px)/lg(672px).
- Header: border-b gray-200, flex justify-between, close icon top-right (reuse `crates/icons` close/x icon).
- Body: p-6. Footer: border-t gray-200, flex gap-3 justify-end, p-4.

**DropdownMenu** (`crates/ui/src/components/dropdown_menu.rs`):
- Menu: bg surface, border gray-200, rounded-md, shadow-lg (`Shadow::Lg`).
- Items: px-4 py-2, text-sm, hover:bg-gray-100, cursor-pointer.
- Separator: border-t gray-200, my-1. Disabled item: text-gray-400, cursor-not-allowed (check `Disableable` trait usage already wired).

**Tabs** (`crates/ui/src/components/tab.rs`, `tab_bar.rs`):
- Underline style: gap-8, border-b gray-200; active: text-blue-600, border-b-2 blue-600, py-4; inactive: text-gray-500, hover:text-gray-700.
- Pills style: gap-2, bg-gray-100 rounded-lg p-1; active: bg white/surface, text-gray-900, shadow-sm; inactive: text-gray-600, hover:bg-gray-50.
- If no style-mode enum exists on `Tab`/`TabBar`, add one (e.g. `TabBarStyle::Underline | Pills`) — small enum addition, not a rebuild.

**Tooltip** (`crates/ui/src/components/tooltip.rs`):
- bg gray-900, text white, text-xs, px-2 py-1, rounded-md, shadow-lg.
- Arrow + placement (top/bottom/left/right) — check if already supported; restyle colors only if positioning logic already exists.

**Popover** (`crates/ui/src/components/popover.rs`):
- Same visual treatment as DropdownMenu (bg surface, border, rounded-md, shadow-lg) — restyle for consistency.

**Toast/Notification** (`crates/ui/src/components/notification/announcement_toast.rs`):
- bg surface, border gray-200, rounded-lg, shadow-lg, p-4, max-width 384px.
- Icon (left, colored per type) + title + message + close button (right).
- Color variants: success/error/warning/info accent (reuse `Severity` pattern from Phase 02's Alert).
- Auto-dismiss timer (5s) + stacking (gap-3, max 3 visible) — ADD if not already present; check existing timer/dismiss logic first.

### Non-functional

- Each restyle stays additive to existing structure — no breaking API changes to public builder methods unless a genuinely missing feature (style-mode enum, auto-dismiss) requires a new optional method (default to current behavior).
- Files stay under 200 lines; if adding auto-dismiss logic pushes `announcement_toast.rs` over, extract a small `toast_timer.rs`.

## Architecture

No new files expected except possibly:
```
crates/ui/src/components/notification/
└── toast_timer.rs   (NEW, only if auto-dismiss logic doesn't fit in announcement_toast.rs cleanly)
```
All other work is restyling existing files in place.

## Related Code Files

**Read first:**
- `crates/ui/src/components/data_table/data_table.rs` (or equivalent parent) — confirm table container structure.
- `crates/ui/src/components/tooltip.rs` — confirm arrow/placement logic already exists.
- `crates/ui/src/components/notification/announcement_toast.rs` — confirm auto-dismiss/stacking presence.

**Modify:**
- `crates/ui/src/components/data_table/table_row.rs` + parent table file
- `crates/ui/src/components/modal.rs`
- `crates/ui/src/components/dropdown_menu.rs`
- `crates/ui/src/components/tab.rs`
- `crates/ui/src/components/tab_bar.rs`
- `crates/ui/src/components/tooltip.rs`
- `crates/ui/src/components/popover.rs`
- `crates/ui/src/components/notification/announcement_toast.rs`

## Implementation Steps

1. Read all "Read first" files to confirm existing capability gaps (auto-dismiss, style-mode enums, table container).
2. Restyle Table (header + rows + striped + hover) using Phase 01 palette.
3. Restyle Modal (overlay opacity, container shadow-xl, header/body/footer spacing).
4. Restyle DropdownMenu (bg/border/shadow-lg/item hover/separator/disabled).
5. Add/restyle Tabs style-mode (underline vs pills) — add enum field if missing, wire both render paths.
6. Restyle Tooltip colors (bg gray-900/text white); verify arrow+placement still works after restyle.
7. Restyle Popover to match DropdownMenu visual treatment.
8. Restyle Toast; add auto-dismiss timer + stacking limit (max 3) if missing, reusing `Severity` color pattern from Phase 02 Alert.
9. Update all touched `fn preview()` functions.
10. `cargo check -p ui` clean.
11. Visual verify: Playwright screenshot Tailwind UI tables/modals/dropdowns/tabs/tooltips/notifications pages; render each component's `preview()` via `VisualTestAppContext`; compare and iterate.

## Todo List

- [ ] Read data_table container, tooltip, announcement_toast for existing capability
- [ ] Restyle Table (header/rows/striped/hover)
- [ ] Restyle Modal (overlay/shadow-xl/spacing)
- [ ] Restyle DropdownMenu (bg/border/shadow-lg/items)
- [ ] Tabs: add/restyle underline + pills modes
- [ ] Restyle Tooltip colors
- [ ] Restyle Popover to match DropdownMenu
- [ ] Toast: restyle + auto-dismiss + stacking (if missing)
- [ ] Update preview() functions
- [ ] cargo check -p ui clean
- [ ] Visual verify vs Tailwind UI tables/modals/dropdowns/tabs/tooltips/notifications

## Success Criteria

- All 7 components compile and render via `preview()` without regression to existing callers.
- Modal overlay renders semi-transparent backdrop correctly (visually confirmed).
- Tabs demonstrably support both underline and pills modes (shown in `preview()`).
- Toast auto-dismisses after 5s and stacks correctly when 3+ triggered (manually verified in gallery app, not just compiled).
- `make check-all` green.
- Visual comparison documented for each component.

## Risk Assessment

- **Risk:** Toast auto-dismiss timer may need `cx.spawn`/background task pattern not yet used elsewhere in `ui` crate — check for existing timer patterns in codebase (e.g. Zed's original toast likely had one; since this is vendored, the logic may already be present, just unstyled — re-verify in step 1 before assuming it's missing).
- **Risk:** Overlay/backdrop full-window dimming might conflict with GPUI's window compositing model (need `absolute()` positioning relative to window root, not local container) — verify Modal's existing overlay mechanism handles this already (it likely does, since Modal already exists and presumably works).

## Security Considerations

None — presentational/UI-state components.

## Next Steps

- Phase 05 gallery's "Overlays" and "Data" showcase pages depend on this phase's restyled `preview()` functions.
