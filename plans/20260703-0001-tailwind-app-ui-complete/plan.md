---
title: "Tailwind Plus Application UI — Full Catalog Parity for GPUI UI Kit"
description: "Audit crates/ui against the full Tailwind Plus Application UI catalog, fix styling deviations, and build every missing component, showcased in ui_gallery"
status: completed (Phases 1-7,9; Phase 8 backlog/cut)
priority: P2
effort: 31h (core; Phase 8 backlog +6h if picked up)
branch: main
tags: [frontend, ui, gpui, design-system]
created: 2026-07-03
lane: frontend/ui/design-system, low-risk
---

# Tailwind Plus Application UI — Full Catalog Parity

## Overview

Prior plan (`plans/20260702-1417-tailwind-ui-gallery-and-uikit/`) built the design-token
foundation (`palette`/`semantic`/`shadow`/`focus_ring`/`severity`, all confirmed present in
`crates/ui/src/styles/`) and ~15 core components, but stopped short of full Tailwind Plus
Application UI catalog parity (31+ categories) and left Button/Checkbox/Switch/Modal/
Dropdown/Tabs/DataTable/Tooltip/Popover/Toast restyle unfinished. This plan closes every gap:
audits + fixes styling deviations on existing components, and builds all missing components
(input groups, combobox, breadcrumb, pagination, stepper, drawer, app shell, etc.), all
showcased in `examples/ui_gallery`. `crates/app` stays untouched.

**Scope: 42 in-plan deliverables** — 19 restyle/fix (existing files, wrong/incomplete tokens)
+ 23 net-new core (Phases 2-7). 6 advanced components (Phase 8) are CUT to backlog per YAGNI
(not core parity, not currently needed) — documented in phase-08 but NOT executed. See phase-01
for the locked IN/OUT matrix.

## Phases

| # | Phase | Status | Effort | Link |
|---|-------|--------|--------|------|
| 1 | Gap-analysis + icon vendoring (foundation, BLOCKS all) | ✅ Done | 2h | [phase-01](./phase-01-gap-analysis-icons.md) |
| 2 | Elements (Button restyle, Button group, Badge dot, Avatar, Facepile, Chip, Divider) | ✅ Done | 4h | [phase-02](./phase-02-elements.md) |
| 3 | Form controls (Checkbox/Switch restyle, Input group, Search, Combobox, Multi-select, Segmented control, Form layout, File input) | ✅ Done | 5h | [phase-03](./phase-03-form-controls.md) |
| 4 | Data display & lists (DataTable restyle, List restyle, Description list, Stats card, Media object, Empty state) | ✅ Done | 4h | [phase-04](./phase-04-data-display.md) |
| 5 | Overlays (Modal/AlertModal/Dropdown/ContextMenu/Popover/Tooltip/Toast restyle, Drawer, Toast stack) | ✅ Done | 5h | [phase-05](./phase-05-overlays.md) |
| 6 | Navigation (Tabs/Progress restyle, Breadcrumb, Pagination, Vertical nav, Stepper) | ✅ Done | 4h | [phase-06](./phase-06-navigation.md) |
| 7 | Layout/shells & headings (App shell, Page/Section heading, Container, Card polish, Feed) | ✅ Done | 4h | [phase-07](./phase-07-layout-shells.md) |
| 8 | Deferred/low-priority (Calendar, Command palette, Color picker, Carousel, Kanban, Virtualized list) | 🔵 Backlog — CUT (YAGNI); execute only on explicit go | 6h | [phase-08](./phase-08-deferred-advanced.md) |
| 9 | Gallery consolidation + final verify | ✅ Done | 3h | [phase-09](./phase-09-gallery-consolidation.md) |

## Dependencies

- Phase 1 BLOCKS 2-7: locks the IN/OUT scope matrix + vendors any missing Heroicons the in-scope
  components need. Do not start component work before Phase 1's matrix is written.
- Phases 2-7 are mutually independent — run in PARALLEL. Each owns a disjoint file set (see each
  phase's "Related Code Files"). Only shared touch point: `crates/ui/src/components.rs` (mod
  declarations) and `crates/ui/src/prelude.rs` (re-exports) — every phase APPENDS its own lines
  there; low merge-conflict risk (append-only), resolve via normal git merge, re-run `cargo check`
  after integrating all phase branches.
- Phase 8 is CUT from this plan (backlog, YAGNI) — do not execute without explicit user go.
- Phase 9 depends on 2-7 being merged — wires every new/restyled component into
  `examples/ui_gallery` pages and runs the final full-workspace verify.

## Cross-Cutting Requirements (apply to EVERY phase, 2-9)

- **Generic naming, no brand ids.** No `tw`/`tailwind`/`slate`/`blue` in code. Use `palette` (role: neutral/primary/success/warning/danger/info), `semantic` (theme-driven via `cx.theme().colors()`), `shadow`, `focus_ring`. "Tailwind" = prose value-source only.
- **Dark + light both.** Neutrals (surface/border/text/hover-bg) ALWAYS `semantic::*`; accents from `palette::{primary,success,warning,danger,info}`. Never hardcode a neutral gray.
- **Focus ring = true gapped ring** via existing `crates/ui/src/styles/focus_ring.rs` wrapper, not a plain thick border.
- **Component pattern:** `#[derive(IntoElement, Documented, RegisterComponent)]` + builder + `impl Component { fn preview() }` (template: `components/button/button.rs`). Every touched component gets a `preview()` update and a gallery showcase (wired Phase 9, written in your own phase).
- **Reuse over rewrite** — map to an EXISTING base first (each phase's "Reuse Map" table); net-new only if no base exists.
- **KHÔNG đụng `crates/app`** / `default-members = ["crates/app"]`. Gallery builds only via `cargo run -p ui_gallery`.
- **Verify per phase:** `make check` (+`make check-all`) green, `cargo fmt --all --check` green, `cargo run -p ui_gallery` opens + phase's components render without panic. No screenshot verify (user-confirmed).

## Key Codebase Facts (from research + verified spot-check, do not re-derive)

- Tokens CONFIRMED present: `crates/ui/src/styles/{palette,semantic,shadow,focus_ring,severity,spacing,typography,animation,elevation,color}.rs` (prior plan's Phase 1, done, nothing to redo).
- `crates/ui/src/components/` has 60+ files already; most Tailwind categories map to an EXISTING file needing restyle, not a new file — see per-phase Reuse Map.
- Icons: `crates/icons/src/icons.rs` (313 lines, 268 variants) already has `ExclamationTriangle, XCircle, XMark, CheckCircle, Info, Star, Plus, SquarePlus, SquareMinus, Settings, MagnifyingGlass, ChevronLeft/Right, UserCheck, UserGroup, Close`. Grep-confirmed MISSING: plain `User`, `Home`, `Calendar`, `Heart`, `MapPin`, plain `Minus` — Phase 1 vendors only what in-scope components need.
- Gallery: `examples/ui_gallery/src/gallery_app.rs` (184 lines), 4 pages (`Elements, Forms, Feedback, Navigation`) via `.preview()`. Phase 9 adds pages (Data, Overlays, Layout) — reuse `preview()`, no duplicate showcase code.
- Workspace: `Cargo.toml` `members` already includes `"examples/ui_gallery"`; `default-members = ["crates/app"]` unchanged (grep-confirmed).
- Restyle-pending files confirmed to exist (no new file needed): `button/{button_like,button,icon_button}.rs`, `toggle.rs` (Checkbox+Switch), `avatar.rs`, `facepile.rs` (=Avatar Group), `chip.rs`, `divider.rs`, `data_table.rs`+`table_row.rs`, `list.rs`+`list/*`, `modal.rs`, `notification/alert_modal.rs` (=Confirm Dialog), `dropdown_menu.rs`, `context_menu.rs`, `right_click_menu.rs`, `popover_menu.rs`, `popover.rs`, `tooltip.rs`, `notification/announcement_toast.rs`, `tab.rs`+`tab_bar.rs`, `progress.rs`+`progress/*`, `card.rs`, `navbar.rs`, `sidebar.rs`.

## Resolved Decisions (user sign-off 2026-07-03)

1. ✅ **Phase 8 CUT to backlog (YAGNI).** Calendar/Command palette/Color picker/Carousel/Kanban/Virtualized list are not core Application-UI parity and not currently needed. Documented in phase-08 but NOT executed; revisit only on explicit user go. Does not block Phases 1-7 or 9.
2. ✅ **Phase 5 restyle depth = container-level + grep-driven child fix.** Restyle at container level is the default; Phase 5 step 1 greps each overlay's file tree for hardcoded hex/hsla/rgb literals and fixes any found (required by the no-hardcode-neutral rule). NOT a full child-by-child rebuild.
3. ✅ **Combobox / Multi-Select = minimal.** Compose existing `Select`+`DropdownMenu`+`TextInput` with case-insensitive substring filter. No new overlay primitive, no fuzzy/async/remote data.

## Open Questions

1. Button restyle completion — Phase 1 must audit whether primary/secondary/soft/ghost/danger variants are ACTUALLY token-correct or only partially done (report says 🟡 Partial); Phase 2 fixes whatever gap Phase 1 finds. (Resolved during Phase 1 execution, not a blocker.)
