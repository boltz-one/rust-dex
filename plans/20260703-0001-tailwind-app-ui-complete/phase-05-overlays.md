# Phase 05 — Overlays (Modal, AlertModal, Dropdown/ContextMenu, Popover, Tooltip, Toast restyle + Drawer, Toast Stack)

## Context Links

- Research: `researcher-01-tailwind-appui-catalog.md` (Overlays row: Modal Dialogs, Drawers/Slide-overs, Notifications/Toasts)
- Research: `researcher-02-codebase-audit.md` (Modal/Dropdown/Tooltip/Popover/Toast/DataTable ⬜ pending restyle)
- Prior plan: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/phase-04-composite-overlay.md` (detailed restyle spec for these 7 components — REUSE its spec values, do not re-derive)
- Plan: `./plan.md` (Cross-Cutting Requirements)

## Overview

- Date: 2026-07-03
- Description: Finish the restyle of Modal, AlertModal(=Confirm/Alert Dialog), DropdownMenu, ContextMenu, RightClickMenu, PopoverMenu, Popover, Tooltip, Toast (all confirmed still ⬜ pending per report-02); add net-new Drawer and Toast Stack.
- Priority: P2
- Implementation status: Pending
- Review status: Not reviewed

## Resolved Decision (2026-07-03)

- **Restyle depth = container-level + grep-driven child fix.** Default is container-level restyle. STEP 1 of this phase greps each overlay's file tree for hardcoded color literals (`hsla(`, `hsl(`, `rgb`, `#` hex, `gpui::rgb`) and fixes any found to `semantic::*`/`palette::*` (required by the no-hardcode-neutral cross-cutting rule). This is NOT a full child-by-child rebuild — only literals that break the token system get replaced.

## Key Insights

- The prior plan's Phase 04 (`phase-04-composite-overlay.md`) already wrote a full restyle spec (colors, spacing, shadow levels) for Modal/DropdownMenu/Tabs/Tooltip/Popover/Toast/Table — REUSE those exact values here (link above), this phase's job is to actually EXECUTE what report-02 confirms is still ⬜ not done (Table is Phase 4 of THIS plan, Tabs is Phase 6 of THIS plan — this phase only covers the Overlay-category subset: Modal, Dropdown family, Popover, Tooltip, Toast).
- `notification/alert_modal.rs` ALREADY EXISTS — this IS the Confirm/Alert Dialog base (modal variant with yes/no buttons), reuse it, do not build a new `confirm_dialog.rs`.
- 4 dropdown-family files exist (`dropdown_menu.rs`, `context_menu.rs`, `right_click_menu.rs`, `popover_menu.rs`) — restyle all 4 together for visual consistency (same bg/border/shadow-lg/item-hover treatment).
- Drawer/slide-over is genuinely new (no side-panel component exists) — structurally similar to Modal (header/body/footer) but positioned `fixed right-0` with slide animation instead of centered.
- Toast Stack is new (container managing multiple `announcement_toast.rs` instances, max-visible limit + auto-dismiss) — `announcement_toast.rs` itself is the per-toast unit, already exists.

## Requirements

### Reuse Map

| Tailwind category | GPUI base | Action |
|---|---|---|
| Modal Dialogs | `components/modal.rs` | RESTYLE |
| Confirm/Alert Dialog | `components/notification/alert_modal.rs` | RESTYLE |
| Drawers/Slide-overs | `components/modal.rs` (structural sibling) | NEW `components/drawer.rs` |
| Dropdowns (menu) | `components/dropdown_menu.rs`, `context_menu.rs`, `right_click_menu.rs`, `popover_menu.rs` | RESTYLE (all 4 consistently) |
| (Popover as generic overlay) | `components/popover.rs` | RESTYLE |
| Tooltip | `components/tooltip.rs` | RESTYLE |
| Notifications/Toasts | `components/notification/announcement_toast.rs` | RESTYLE + auto-dismiss/stacking if missing |
| Toast stack container | none | NEW `components/notification/toast_stack.rs` |

### Functional

(Values per prior plan's phase-04 spec, mapped to `palette`/`semantic` per this plan's naming rule — see that file for exact px/color numbers.)

- **Modal**: overlay `semantic`-derived dark scrim (or `black().opacity(0.5)` if overlay scrim isn't a themed neutral — confirm GPUI's overlay convention first), container `semantic::surface`, `rounded_lg`, `Shadow::Xl`, width variants sm/md/lg. Header `border_muted` bottom + close icon (`IconName::XMark` or `Close`). Body p-6. Footer `border_muted` top, gap-3 justify-end.
- **AlertModal (Confirm Dialog)**: restyle to same Modal token spec; confirm/cancel buttons use Phase 2's restyled `Button` (danger variant for destructive confirm).
- **Drawer** (new): same header/body/footer structure as Modal, positioned `fixed right-0`, width ~384px (`w-96`), slide-in animation via existing `AnimationDuration`/`Animated` mechanism (reuse, don't invent).
- **Dropdown family (4 files)**: `semantic::surface` bg, `border_muted`, `rounded_md`, `Shadow::Lg`; items px-4 py-2 text-sm `hover_bg`; separator `border_muted` my-1; disabled items `text_muted` + no hover (confirm `Disableable` trait already wired).
- **Popover**: same visual treatment as dropdown family for consistency.
- **Tooltip**: `palette::neutral(900)`-equivalent dark bg (or a dedicated "inverse surface" if `semantic` has one — check first) + white text, text-xs, px-2 py-1, `rounded_md`, `Shadow::Lg`. Keep existing arrow/placement logic, restyle colors only.
- **Toast restyle**: `semantic::surface`, `border_muted`, `rounded_lg`, `Shadow::Lg`, p-4, max-w-384px; icon (role-colored, reuse `Severity` pattern from Alert) + title + message + close button.
- **Toast Stack** (new): container managing N `announcement_toast.rs` instances, gap-3 vertical stack, max 3 visible (overflow queued), each toast auto-dismisses after 5s (check existing timer logic in `announcement_toast.rs` first — reuse if present, add if missing via existing `cx.spawn` background-task pattern already used elsewhere in the crate, do not invent a new async pattern).

### Non-functional

- Additive restyle only — no breaking public builder API changes unless a genuinely missing feature (Drawer, Toast Stack, auto-dismiss) requires a new method.
- If auto-dismiss logic pushes `announcement_toast.rs` over 200 lines, extract `notification/toast_timer.rs`.

## Architecture

```
crates/ui/src/components/
├── modal.rs                        (MODIFY)
├── drawer.rs                        (NEW)
├── dropdown_menu.rs                  (MODIFY)
├── context_menu.rs                    (MODIFY)
├── right_click_menu.rs                 (MODIFY)
├── popover_menu.rs                      (MODIFY)
├── popover.rs                            (MODIFY)
├── tooltip.rs                             (MODIFY)
└── notification/
    ├── alert_modal.rs                      (MODIFY)
    ├── announcement_toast.rs                (MODIFY)
    └── toast_stack.rs                        (NEW)
```

## Related Code Files

**Read first:** `modal.rs` (overlay/backdrop mechanism), `notification/announcement_toast.rs` (existing timer/dismiss logic), `crates/ui/src/styles/severity.rs` (reuse pattern from Phase 2's prior Alert work).

**Modify:** all 9 files listed under Architecture "MODIFY".

**Create:** `drawer.rs`, `notification/toast_stack.rs`.

## Implementation Steps

1. Read `modal.rs`, `announcement_toast.rs` fully — confirm overlay mechanism + existing timer/dismiss state before changing.
2. Restyle Modal (overlay/container/header/body/footer).
3. Restyle AlertModal to same Modal spec; wire danger-variant Button for destructive actions.
4. Build Drawer (Modal-sibling structure, side position, slide animation).
5. Restyle all 4 dropdown-family files consistently (same pass, cross-check visual parity).
6. Restyle Popover to match dropdown family.
7. Restyle Tooltip (colors only, keep placement logic).
8. Restyle Toast; add auto-dismiss timer if missing (reuse `cx.spawn` pattern found elsewhere).
9. Build Toast Stack (stacking container, max-3-visible, gap-3).
10. Update/add `preview()` for all 11 deliverables.
11. `cargo check -p ui` clean.
12. `cargo run -p ui_gallery` — manually trigger a Modal, a Dropdown, a Toast (via a temporary preview button) to confirm real open/close/dismiss behavior, not just static render.

## Todo List

- [ ] Read modal.rs + announcement_toast.rs existing mechanisms
- [ ] Restyle Modal
- [ ] Restyle AlertModal (confirm/cancel wired to Phase2 Button danger variant)
- [ ] Build Drawer
- [ ] Restyle DropdownMenu + ContextMenu + RightClickMenu + PopoverMenu (consistent pass)
- [ ] Restyle Popover
- [ ] Restyle Tooltip
- [ ] Restyle Toast + auto-dismiss (if missing)
- [ ] Build Toast Stack
- [ ] preview() for all 11
- [ ] cargo check -p ui clean
- [ ] Manual interaction test: Modal open/close, Dropdown open/select, Toast auto-dismiss+stack

## Success Criteria

- `make check` + `make check-all` + `cargo fmt --all --check` green.
- Modal overlay renders backdrop + correct shadow/spacing; Drawer slides in from the right.
- All 4 dropdown-family components visually consistent (same bg/border/shadow/hover treatment).
- Toast auto-dismisses after 5s and stacks (max 3 visible) — verified by manual interaction in `cargo run -p ui_gallery`, not just compile.
- No regression to existing callers of any of the 9 restyled files (`cargo check --workspace --all-targets`).

## Risk Assessment

- **Risk:** Toast auto-dismiss timer may need an async/spawn pattern not yet confirmed present. **Mitigation:** step 1 explicitly re-verifies before assuming missing (prior plan flagged this same risk and it may already be resolved).
- **Risk:** Restyling 4 dropdown-family files in the same pass risks inconsistent completion if done piecemeal. **Mitigation:** step 5 explicitly says "same pass, cross-check parity" — do not merge partial work.
- **Risk:** Drawer's slide animation reusing `AnimationDuration`/`Animated` may need a position-interpolation approach not previously used for a full-panel slide (as opposed to toggle-state alpha/scale). **Mitigation:** check `animation.rs` for any transform/offset-based animation helper before writing custom interpolation.

## Security Considerations

None — presentational overlay components.

## Next Steps

- Phase 9 gallery Overlays page needs all 11 deliverables' `preview()`.
- Phase 6 (Navigation) restyles Tabs separately — no overlap with this phase.
