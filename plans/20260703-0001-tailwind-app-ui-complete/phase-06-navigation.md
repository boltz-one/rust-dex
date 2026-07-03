# Phase 06 — Navigation (Tabs/Progress restyle, Breadcrumb, Pagination, Vertical Nav, Stepper)

## Context Links

- Research: `researcher-01-tailwind-appui-catalog.md` (Navigation row: Pagination, Vertical Navigation, Breadcrumbs, Tabs, Progress Bars)
- Research: `researcher-02-codebase-audit.md` (Tab/TabBar ⬜ pending restyle; Progress 🟡 partial; Breadcrumb/Pagination/Stepper ⬜ missing)
- Prior plan: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/phase-04-composite-overlay.md` (Tabs underline/pills spec — reuse values)
- Plan: `./plan.md` (Cross-Cutting Requirements)

## Overview

- Date: 2026-07-03
- Description: Restyle Tabs (underline + pills modes) and Progress (bar + circular); build net-new Breadcrumb, Pagination, Vertical Nav, Stepper.
- Priority: P2
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- `Navbar`/`Sidebar` (full app-shell nav containers) are ALREADY DONE (report-02: ✅ Done) — this phase does NOT touch them. "Vertical Navigation" here means a lighter-weight nav-LINKS-ONLY component (used inside a page, e.g. settings sub-nav), distinct from the full `Sidebar` container — new file, no conflict with `sidebar.rs`.
- `tab.rs`/`tab_bar.rs` need a style-mode enum (underline vs pills) per prior plan's finding — check if it already exists before adding.
- Progress (`progress.rs` + `progress/{progress_bar,circular_progress}.rs`) is 🟡 partial — restyle colors/track only, structure exists.
- Breadcrumb/Pagination/Stepper are genuinely new, simple components — no existing base to compose beyond `Icon`/`Button`/`Label`.

## Requirements

### Reuse Map

| Tailwind category | GPUI base | Action |
|---|---|---|
| Tabs | `components/tab.rs`, `tab_bar.rs` | RESTYLE (add underline/pills mode if missing) |
| Progress Bars | `components/progress.rs` + `progress/*.rs` | RESTYLE |
| Breadcrumbs | none | NEW `components/breadcrumb.rs` |
| Pagination | none | NEW `components/pagination.rs` |
| Vertical Navigation | none (distinct from `sidebar.rs`) | NEW `components/vertical_nav.rs` |
| Stepper (multi-step) | none | NEW `components/stepper.rs` |

### Functional

- **Tabs restyle**: underline mode — gap-8, `border_muted` bottom container line, active tab `palette::primary(600)` text + 2px bottom border in same color, inactive `semantic::text_muted` + `hover:text` on hover, py-4. Pills mode — gap-2, `semantic::elevated_surface` bg rounded-lg p-1 container, active tab `semantic::surface` bg + `Shadow::Sm` + `text`, inactive `text_muted` + `hover_bg`. Add a `TabBarStyle::Underline | Pills` enum field if not present (small addition, verify `tab_bar.rs` first).
- **Progress restyle**: bar — track `semantic::border_muted`/neutral-200-equivalent bg, fill `palette::primary(600)`, `rounded_full`, h-2. Circular — same fill/track color mapping applied to the existing circular SVG/path logic, no structural change.
- **Breadcrumb** (new): flex row, `ChevronRight`/`/` separator icon, links `text_muted` + `hover:text`, current/last item `text` (non-link, bold-ish).
- **Pagination** (new): prev/next buttons (reuse `IconButton` from Phase 2's Button family) + numbered page buttons (reuse `Button` ghost/soft variant for active/inactive), disabled state on prev/next at bounds.
- **Vertical Nav** (new): simple `v_flex()` list of nav links, `rounded_md`, px-4 py-2, active `semantic::elevated_surface` bg + `text`, inactive `text_muted` + `hover_bg` — no collapse/nested logic (that's `Sidebar`'s job, already done), keep this component minimal (flat link list only).
- **Stepper** (new): horizontal row of step circles connected by a line; states: completed (`palette::primary(600)` filled + check icon), current (`palette::primary(600)` outline ring, `focus_ring`-style), upcoming (`semantic::border_muted` outline + `text_muted` number).

### Non-functional

- Files under 200 lines; Pagination/Stepper are pure presentational + `on_click` callbacks, no internal page-state ownership (caller owns current page/step).

## Architecture

```
crates/ui/src/components/
├── tab.rs                  (MODIFY)
├── tab_bar.rs                (MODIFY — add style-mode enum if missing)
├── progress.rs                 (MODIFY)
├── progress/{progress_bar,circular_progress}.rs (MODIFY)
├── breadcrumb.rs                  (NEW)
├── pagination.rs                    (NEW)
├── vertical_nav.rs                    (NEW)
└── stepper.rs                          (NEW)
```

## Related Code Files

**Read first:** `tab_bar.rs` (check for existing style-mode field), `progress.rs`.

**Modify:** `tab.rs`, `tab_bar.rs`, `progress.rs`, `progress/*.rs`, `crates/ui/src/components.rs`, `crates/ui/src/prelude.rs`.

**Create:** `breadcrumb.rs`, `pagination.rs`, `vertical_nav.rs`, `stepper.rs`.

## Implementation Steps

1. Read `tab_bar.rs` fully — confirm underline/pills mode presence or absence.
2. Restyle Tabs (add style-mode enum only if missing, wire both render paths).
3. Restyle Progress (bar + circular track/fill colors).
4. Build Breadcrumb.
5. Build Pagination (reusing Phase 2's `IconButton`/`Button`).
6. Build Vertical Nav (flat link list, distinct from `Sidebar`).
7. Build Stepper (completed/current/upcoming states).
8. Update/add `preview()` for all 6 deliverables.
9. `cargo check -p ui` clean.
10. `cargo run -p ui_gallery` — confirm Tabs switch content on click, Pagination buttons disable correctly at bounds.

## Todo List

- [ ] Read tab_bar.rs for existing style-mode
- [ ] Restyle Tabs (underline + pills)
- [ ] Restyle Progress (bar + circular)
- [ ] Build Breadcrumb
- [ ] Build Pagination
- [ ] Build Vertical Nav
- [ ] Build Stepper
- [ ] preview() for all 6
- [ ] cargo check -p ui clean
- [ ] Manual click-test Tabs switching + Pagination bounds

## Success Criteria

- `make check` + `make check-all` + `cargo fmt --all --check` green.
- Tabs demonstrably support both underline and pills modes (shown in `preview()`).
- Pagination's prev/next disable correctly at first/last page (manual test).
- Vertical Nav does not duplicate `Sidebar`'s collapse/nested functionality (confirmed by code review — it's a flat list).
- No regression to existing `Tab`/`TabBar`/`Progress` callers.

## Risk Assessment

- **Risk:** Adding a `TabBarStyle` enum field could be a breaking change if `TabBar::new()` doesn't default sensibly. **Mitigation:** default to `Underline` (current visual behavior) so existing callers are unaffected.
- **Risk:** Vertical Nav and Sidebar could drift into overlapping responsibility over time (code duplication risk). **Mitigation:** doc-comment on `vertical_nav.rs` explicitly stating "flat link list for in-page sub-nav; for the app-level collapsible sidebar use `Sidebar`".

## Security Considerations

None — presentational navigation components.

## Next Steps

- Phase 9 gallery Navigation page needs this phase's 6 deliverables plus already-done Navbar/Sidebar self-demo.
- Phase 7's Application Shell composes `Navbar`+`Sidebar` (already done) — may optionally also demo this phase's `VerticalNav` as a sub-nav example, not a hard dependency.
