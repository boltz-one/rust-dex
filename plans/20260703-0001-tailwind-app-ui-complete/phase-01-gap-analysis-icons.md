# Phase 01 — Gap-Analysis Matrix + Icon Vendoring (Foundation)

## Context Links

- Research: `plans/20260703-0001-tailwind-app-ui-complete/research/researcher-01-tailwind-appui-catalog.md` (full 31-category matrix + not-suitable-for-desktop list)
- Research: `plans/20260703-0001-tailwind-app-ui-complete/research/researcher-02-codebase-audit.md` (component status matrix, icon inventory)
- Prior plan: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/phase-01-design-tokens.md` (tokens already built, reference only, no rework)
- Plan: `./plan.md` (Cross-Cutting Requirements — read before touching any file)

## Overview

- Date: 2026-07-03
- Description: Lock the IN-SCOPE / DEFERRED matrix for every Tailwind Application UI category (feeds Phases 2-8's exact deliverable list), audit Button restyle actual completion state, and vendor the small set of genuinely-missing Heroicons the in-scope components need.
- Priority: P1 (blocks all other phases — do not start Phase 2-8 until this phase's matrix + icons are merged)
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- Tokens (`palette`, `semantic`, `shadow`, `focus_ring`, `severity`) are CONFIRMED present in `crates/ui/src/styles/` — verified via `ls`. Nothing to build here, only consume.
- Icon inventory verified via grep on `crates/icons/src/icons.rs` (268 variants): already has `ExclamationTriangle, XCircle, XCircleFilled, XMark, CheckCircle, Info, Star, StarFilled, Plus, SquarePlus, SquareMinus, Settings, MagnifyingGlass, ChevronLeft, ChevronRight, UserCheck, UserGroup, UserRoundPen, Close`. Grep-confirmed MISSING: plain `User`, `Home`, `Calendar`, `Heart`/`HeartFilled`, `MapPin`, plain `Minus`.
- Do NOT vendor icons speculatively — only vendor what an IN-SCOPE (Phase 2-7) component actually references. Deferred (Phase 8) components' icon needs (e.g. Calendar for date picker) are OUT of this phase's vendoring list unless Phase 8 is confirmed to run.
- Report-02's component matrix already did most of the audit legwork — this phase's job is to RATIFY it into a locked table (some entries may be stale; spot-check the ones marked 🟡/⬜ that this plan's phases will touch).

## Requirements

### Functional

1. **Scope matrix** (write into this file's "Locked Scope Matrix" section below, do not create a separate doc): every row from researcher-01's 31-category table, mapped to: IN-SCOPE (which Phase 2-8 owns it) or OUT/DEFERRED (Phase 8, or truly out — e.g. "Responsive containers" per researcher-01's not-suitable list). Every category must get exactly one destination phase.
2. **Button restyle audit**: read `crates/ui/src/components/button/button_like.rs` (TintColor/ButtonStyle enums) and `button.rs` `preview()`. Determine: are primary(bg palette::primary(600))/secondary(semantic border+surface)/soft/ghost/danger variants ACTUALLY wired to `palette`/`semantic`, or still using old Zed colors? Write the finding as a checklist in this phase's Todo (feeds Phase 2 step 1, do not fix here — Phase 2 owns the fix).
3. **Icon vendoring**: for each in-scope component in Phase 2-7 that needs an icon NOT already in `IconName`, vendor the Heroicon SVG (MIT, outline or solid matching existing style — check `crates/icons` embed pattern first) + add the `IconName` variant. Expected additions based on current phase scope: plain `User` (Avatar/EmptyState fallback), `Home` (Breadcrumb/VerticalNav example), plain `Minus` (SegmentedControl/Stepper). Confirm against actual Phase 2-7 requirements below before vendoring — do not add unused variants.
4. Cross-check: for every NEW component listed in report-02's "NOT YET IN CRATES/UI" table, confirm it's assigned to exactly one phase (2-8) in the Locked Scope Matrix — no orphans.

### Non-functional

- This phase produces documentation (the matrix, appended to this file) + `crates/icons` changes only. No `crates/ui/src/components/*` edits (that's Phase 2-7's job) — keeps this phase a clean, fast, non-blocking merge.

## Architecture

No new component files. Only:
```
crates/icons/src/icons.rs   (MODIFY — add missing IconName variants)
crates/icons/assets/...     (MODIFY — add missing SVGs, path matches existing embed pattern)
```

## Related Code Files

**Read (audit, no modify):**
- `crates/ui/src/components/button/button_like.rs`, `button.rs`
- `crates/ui/src/styles/{palette,semantic,shadow,focus_ring,severity}.rs`

**Modify:**
- `crates/icons/src/icons.rs`
- `crates/icons/assets/` (or wherever existing SVGs live — inspect first)

## Implementation Steps

1. Copy researcher-01's 31-category table into this file's "Locked Scope Matrix" section; for each row, write the owning phase number (2-8) or "OUT" with a one-line reason (reuse researcher-01's "Not Suitable for Desktop" reasons where applicable).
2. Cross-check every row against report-02's "NOT YET IN CRATES/UI" and "Component Status Matrix" tables — make sure every missing/pending component ends up in exactly one Phase 2-8 bucket (no gaps, no duplicates).
3. Read `button_like.rs` + `button.rs` fully; write a short "Button restyle finding" note in Todo list (e.g. "primary variant uses palette::primary(600) ✓, ghost variant still hardcoded #xxxxxx ✗ — Phase 2 must fix").
4. Read Phase 2-7 requirement lists below in this plan (already drafted) to build the exact icon-need list; grep `crates/icons/src/icons.rs` again to reconfirm which are genuinely missing (list may have grown since this plan was drafted).
5. Inspect `crates/icons`'s existing SVG embed pattern (how `IconName::Star` maps to an asset file) before adding new ones — match exactly, do not invent a new pattern.
6. Vendor each missing Heroicon SVG (heroicons.com, MIT) + add `IconName` variant, following existing naming convention (PascalCase, matches Heroicon's kebab-case name).
7. `cargo check -p icons` clean.
8. `make check-all` + `cargo fmt --all --check` green (this phase touches icons + this doc only, should be a trivial pass).

## Todo List

- [ ] Write Locked Scope Matrix (all 31 categories → phase 2-8 or OUT)
- [ ] Cross-check report-02's missing-component + status tables against the matrix (no orphans)
- [ ] Audit Button restyle actual state (`button_like.rs`, `button.rs`) — write finding for Phase 2
- [ ] Build final icon-need list from Phase 2-7 requirements (re-grep `IconName` to reconfirm gaps)
- [ ] Inspect existing icon embed pattern in `crates/icons`
- [ ] Vendor missing Heroicon SVGs + register `IconName` variants
- [ ] `cargo check -p icons` clean
- [ ] `make check-all` + `cargo fmt --all --check` green

## Locked Scope Matrix (LOCKED 2026-07-03 — verified against real code)

Token API confirmed: `palette::{neutral,primary,info,success,warning,danger}(50..=950)->Hsla` (accent/status, mode-agnostic); `semantic::{background,surface,elevated_surface,border,border_muted,border_focused,text,text_muted,text_placeholder,hover_bg,active_bg,icon,icon_muted}(cx)->Hsla` (theme-driven neutral); `focus_ring_primary(content,focused)` / `focus_ring_error(...)` wrappers; `shadow.rs`, `severity.rs`. **Reference idiom = `components/badge.rs`** (role enum→`palette`, `#[derive(IntoElement, RegisterComponent)]`, builder, `RenderOnce`, `impl Component{scope/description/preview}`; `Documented` derive is OPTIONAL — omit unless the file already uses it).

| Category | Owner | Notes |
|---|---|---|
| Application Shells (stacked/sidebar) | P7 | NEW app_shell.rs composes done Navbar+Sidebar |
| Multi-column Grid | COVERED | caller-composed flex/grid; P7 documents |
| Page / Section Heading | P7 | NEW |
| Card Heading | P7 | verify Card header slot, extend only if gap |
| Description Lists | P4 | NEW |
| Stats Cards | P4 | NEW (Card wrapper) |
| Calendars | OUT→P8 | backlog, cut |
| Stacked List / List Container | P4 | RESTYLE list.rs + list/* |
| Tables | P4 | RESTYLE data_table |
| Grid List | COVERED | Card + caller layout; P4 documents |
| Feeds | P4 | NEW |
| Form Layouts / Action Panels | P3 | NEW form_field.rs + action_panel.rs |
| Input Groups | P3 | NEW |
| Select Menus | ✅DONE | select.rs already tokenized — verify only, no rebuild |
| Sign-in/Registration | OUT | web full-page; achievable via Modal+FormField composition, no dedicated component |
| Textareas | ✅DONE | text_input.rs `.multiline` |
| Radio Groups | ✅DONE | radio.rs; SegmentedControl(new) is P3 |
| Checkboxes / Toggles | P3 | RESTYLE toggle.rs |
| Comboboxes | P3 | NEW (compose Select+TextInput+DropdownMenu) |
| Alerts | ✅DONE | alert.rs/callout — verify only |
| Empty States | P4 | NEW |
| Navbars | ✅DONE | navbar.rs |
| Pagination | P6 | NEW |
| Vertical Navigation | P6 | NEW (flat list, distinct from Sidebar) |
| Sidebar Navigation | ✅DONE | sidebar.rs |
| Breadcrumbs | P6 | NEW |
| Tabs | P6 | RESTYLE (add underline/pills if missing) |
| Progress Bars | P6 | RESTYLE |
| Command Palettes | OUT→P8 | backlog, cut |
| Modal Dialogs | P5 | RESTYLE modal.rs |
| Drawers/Slide-overs | P5 | NEW drawer.rs |
| Notifications/Toasts | P5 | RESTYLE announcement_toast + NEW toast stack |
| Avatars | P2 | RESTYLE avatar.rs |
| Avatar Groups | P2 | RESTYLE facepile.rs |
| Badges | ✅DONE | badge.rs ALREADY has soft/solid/outline + roles + `dot` — **P2 Badge-dot is a NO-OP, skip it** |
| Dropdowns | P5 | RESTYLE dropdown family |
| Buttons | P2 | **FULL remap — see finding below (bigger than plan assumed)** |
| Button Groups | P2 | NEW button_group.rs |
| Containers | P7 | NEW |
| Cards | ✅DONE | card.rs — P7 polish only |
| Media Objects | P4 | NEW |
| Dividers | P2 | RESTYLE divider.rs |

### Button restyle finding (feeds P2 — REVISED SCOPE)

`button/button_like.rs` uses the **Zed `ThemeColors` system end-to-end** (`cx.theme().colors()`, `cx.theme().status()`) — **zero `palette::`/`semantic::` usage**. `ButtonStyle` enum = `Filled | Tinted(Accent|Error|Warning|Success) | Outlined | OutlinedGhost | OutlinedCustom | Subtle | Transparent`. It IS theme-driven (adapts light/dark) but has **no Tailwind solid-primary** button: `Filled` = neutral surface fill; `Tinted(Accent)` = faint `status().info_background`, NOT solid `primary(600)`.

**P2 approach (NON-BREAKING — do NOT rename/remove enum variants; `icon_button/split_button/toggle_button/copy_button` + crate-wide callers depend on them):**
- Remap `TintColor::button_like_style` so Accent→solid `palette::primary(600)` bg + `white()` text, Error→`danger(600)`, Warning→`warning(600)`, Success→`success(600)`; `hovered`→shade(700). This yields Tailwind solid accent buttons with zero caller changes.
- Add `Button` convenience builders `.primary()/.danger()` (set `ButtonStyle::Tinted(...)`) + a Soft treatment (`primary(50)` bg / `primary(700)` text) via a new `TintColor` render branch or a `Button`-level flag — implementer's choice, keep minimal.
- Secondary = existing `Outlined` (maps `border_variant`→acceptable). Ghost = `Subtle`/`Transparent`. Focus via `focus_ring_primary`.
- **Effort flag:** this is a full token remap of the Tint path, NOT a one-variant touch-up. Larger than plan's original estimate.

## Success Criteria

- This file contains a complete Locked Scope Matrix with every Tailwind Application UI category assigned to exactly one Phase (2-8) or marked OUT with a reason.
- Button restyle finding documented (blocks/unblocks Phase 2 step 1 assumption).
- `cargo check -p icons` and `make check-all` green.
- `cargo fmt --all --check` green.
- No `crates/ui/src/components/*.rs` files touched by this phase (verify via `git diff --stat`).

## Risk Assessment

- **Risk:** Over-vendoring icons "just in case" bloats `crates/icons` and creates unused-variant warnings. **Mitigation:** step 4 explicitly re-derives the icon-need list from the ACTUAL Phase 2-7 requirement text (not a speculative Heroicons full-set import).
- **Risk:** Scope matrix disagreements surface mid-implementation (a Phase 2-7 owner disagrees with this phase's category assignment). **Mitigation:** matrix lives in this file, is a fast/cheap edit — amend and note the change in Next Steps rather than blocking.

## Security Considerations

None — icon SVGs are static assets from a well-known MIT-licensed source (heroicons.com); no dynamic SVG injection.

## Next Steps

- Phases 2-8 start only after this phase's Locked Scope Matrix + icon vendoring is merged.
- If Button restyle finding shows MORE work than expected, flag in Phase 2's Todo list rather than silently expanding this phase's scope.
