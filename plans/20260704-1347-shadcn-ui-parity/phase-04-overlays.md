---
title: "Phase 4 — Overlays"
status: pending
effort: 10h
---

# Phase 4: Overlays

[← plan.md](./plan.md) | Prev: [phase-03](./phase-03-forms.md) | Next: [phase-05](./phase-05-data-nav.md)

## Context
shadcn "Overlays" category, 11 in-scope + 1 explicit skip (Toast, superseded by Sonner in current shadcn — shadcn itself deprecated it, so we don't build a separate Toast, only Sonner in Phase 6). Command (cmdk) is grouped here per user's phase split even though research's raw catalog lists it under "Advanced/Heavy" — it's buildable now on existing `combobox.rs`+`list.rs`, no true blocker.

## Component Table

| Component | Codebase file | Action | Notes |
|---|---|---|---|
| Dialog | `modal.rs` | Align | Verify Root/Trigger/Content/Overlay/Header/Title/Description/Footer/Close anatomy all reachable |
| Alert Dialog | `modal.rs` (thin variant) | New (thin) | Same anatomy as Dialog + Action/Cancel — a preset/builder on top of `modal.rs`, not a parallel implementation |
| Sheet | `drawer.rs` | Align | shadcn Sheet = side:top/right/bottom/left variant of Dialog; confirm `drawer.rs` exposes all 4 sides |
| Drawer | `drawer.rs` | Align | Drag-to-dismiss gesture is the one 🟡 sub-part — verify present or note as a follow-up, don't block the rest of the phase on it |
| Popover | `popover.rs` | Align | Verify Root/Trigger/Content anatomy |
| Hover Card | `tooltip.rs` + `popover.rs` (compose) | New (compose) | Open-on-hover-with-delay + richer content than Tooltip; compose existing two rather than a 3rd overlay primitive (DRY — reuse the deferred+anchored+occlude pattern already proven in both files) |
| Tooltip | `tooltip.rs` | Align | Verify Provider/Root/Trigger/Content anatomy |
| Dropdown Menu | `dropdown_menu.rs` | Align | Verify Item/CheckboxItem/RadioItem/Sub/Separator/Label all present |
| Context Menu | `context_menu.rs` / `right_click_menu.rs` | Align | Verify parity with Dropdown Menu's anatomy (shadcn's Context Menu mirrors Dropdown Menu 1:1) |
| Menubar | none | New | Root/Menu/Trigger/Content/Item/Sub; combine `navbar.rs` (top-bar layout) + `dropdown_menu.rs` (per-menu content) — the new work is cross-menu keyboard navigation (arrow-left/right moves between adjacent open menus) |
| Command | none | New | Text input + live filter + keyboard up/down/enter + grouped sections; build on `combobox.rs` (input+list+keyboard nav already solved there) + `list.rs`; reuse Phase-3's fuzzy-match decision from Combobox rather than picking a second matching strategy |
| Sonner (toast) | — | **Deferred to Phase 6** | Grouped with other heavy/stateful items (stack/queue/timers) — see phase-06 |
| Toast (Radix) | — | **Skip (documented)** | shadcn itself deprecated this in favor of Sonner; don't build a parallel implementation, note the skip |

## Key Insights
- 7 of 11 in-scope items are pure alignment on existing, already-working overlay files — the deferred+anchored+occlude pattern is proven 4+ times already (`modal.rs`, `popover.rs`, `dropdown_menu.rs`, `context_menu.rs`); Menubar/Hover Card/Command must reuse it, not reinvent.
- Only Menubar (cross-menu keyboard nav) and Command (fuzzy filter + grouped virtualization) are genuine new engineering; Alert Dialog and Hover Card are thin compositions.
- Command's filtering should NOT diverge from whatever matching approach Phase 3's Combobox settles on (crate vs. hand-rolled) — one matching strategy across the kit, not two (DRY).

## Requirements
- Alert Dialog: must not duplicate `modal.rs` internals — a builder/preset (`Modal::alert()` or a thin `AlertDialog` wrapper calling into `Modal`) that fixes title/description/action/cancel slots.
- Sheet: confirm all 4 sides (top/right/bottom/left) work via `drawer.rs`'s existing side parameter, or add if only some sides are wired.
- Menubar: only one menu open at a time; arrow-key nav between adjacent top-level menu items while any menu is open (shadcn behavior).
- Command: keyboard up/down wraps or clamps at list ends (verify shadcn's actual behavior — clamp, not wrap, is the typical cmdk default); Enter selects highlighted item; grouped items render group labels as non-selectable separators.

## Architecture
- `AlertDialog` and `Sheet`: no new files if a builder/preset on `modal.rs`/`drawer.rs` suffices — prefer that over new files (YAGNI).
- `HoverCard`: new thin file `hover_card.rs` composing `tooltip.rs`'s hover-delay logic with `popover.rs`'s richer content container.
- `Menubar`: new `menubar.rs`, top-level layout borrowed from `navbar.rs`, each menu's dropdown content delegates entirely to `dropdown_menu.rs`.
- `Command`: new `command.rs`, input+list wiring delegates to `combobox.rs` internals where possible (extract a shared internal filter/keyboard-nav helper only if it doesn't change `combobox.rs`'s public API).

## Related Files
- `crates/ui/src/components/{modal,drawer,popover,popover_menu,tooltip,dropdown_menu,context_menu,right_click_menu,navbar,combobox,list}.rs`
- New: `crates/ui/src/components/{hover_card,menubar,command}.rs`

## Implementation Steps
1. Align Dialog/Sheet/Drawer/Popover/Tooltip/DropdownMenu/ContextMenu against shadcn anatomy tables; log any missing sub-parts.
2. Build Alert Dialog as a `Modal` preset/builder.
3. Build Hover Card composing Tooltip+Popover.
4. Build Menubar (layout first, then cross-menu keyboard nav).
5. Build Command (reuse Combobox's filter strategy from Phase 3; add grouped-section rendering + keyboard nav).
6. Document Toast skip (one paragraph, cite Sonner in Phase 6 as the replacement).
7. Gallery entries (overlays page) for all 11 in-scope items; `#[gpui::test]` for: Alert Dialog action/cancel wiring, Menubar cross-menu arrow-nav, Command filter+keyboard-select.

## Todo
- [ ] Dialog/Sheet/Drawer/Popover/Tooltip/DropdownMenu/ContextMenu aligned
- [ ] Alert Dialog (Modal preset)
- [ ] Hover Card (compose)
- [ ] Menubar (layout + keyboard nav)
- [ ] Command (filter + keyboard nav + groups)
- [ ] Toast skip documented
- [ ] Gallery overlays page updated
- [ ] Harness tests: Alert Dialog, Menubar nav, Command filter/select
- [ ] `cargo build -p ui` / `cargo test -p ui` clean

## Success Criteria
- 11 in-scope overlays present + gallery-visible; Toast documented as intentionally skipped.
- Menubar and Command each have a passing interaction `#[gpui::test]`.
- All overlays use the existing deferred+anchored+occlude primitive — no new overlay-positioning code path introduced.

## Risk & Dependencies
- Depends on Phase 3's Combobox fuzzy-filter decision (Command reuses it).
- Risk: Menubar cross-menu keyboard nav is the highest-complexity item in this phase — budget it first, verify feasibility early rather than late.

## Security
Command palette input: same defensive-truncation posture as Input OTP (Phase 3) — no injection risk (local UI filter only), but guard against unbounded-length paste into the filter input.

## Next
[phase-05-data-nav.md](./phase-05-data-nav.md)
