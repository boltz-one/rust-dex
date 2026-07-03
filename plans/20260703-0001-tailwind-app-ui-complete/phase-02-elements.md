# Phase 02 — Elements (Button, Button Group, Badge, Avatar, Facepile, Chip, Divider)

## Context Links

- Research: `researcher-01-tailwind-appui-catalog.md` (Elements row: Avatars, Avatar Groups, Badges, Dropdowns, Buttons, Button Groups)
- Research: `researcher-02-codebase-audit.md` (Button 🟡, Avatar 🟡, Chip 🟡, Divider 🟡, Badge ✅ done)
- Phase 01: `./phase-01-gap-analysis-icons.md` (Button restyle finding, Locked Scope Matrix)
- Plan: `./plan.md` (Cross-Cutting Requirements)

## Overview

- Date: 2026-07-03
- Description: Finish Button restyle (per Phase 01 finding), add Button Group, add Badge dot variant, restyle Avatar/Facepile(=avatar group)/Chip/Divider to token spec.
- Priority: P1
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- Button is the highest-visibility component (used everywhere) — Phase 01's audit finding tells you exactly which variant(s) still need fixing; do NOT redo variants Phase 01 confirms are already correct.
- `facepile.rs` ALREADY EXISTS and IS the "Avatar Group" component (stacked avatars, overflow count) — reuse it, do NOT create a new `avatar_group.rs`.
- Badge (`badge.rs`) is marked ✅ done for solid/outline/roles — this phase only ADDS a `dot` variant (small colored circle + label), not a full rebuild.
- "Dropdowns" (trigger + menu) as an Elements-catalog item is really the Overlay `DropdownMenu` — already owned by Phase 5, do not duplicate here.

## Requirements

### Reuse Map

| Tailwind category | GPUI base | Action |
|---|---|---|
| Buttons | `components/button/{button_like,button,icon_button}.rs` | RESTYLE (finish per Phase01 finding) |
| Button Groups | none | NEW `components/button/button_group.rs` |
| Badges (dot variant) | `components/badge.rs` | EXTEND (add dot variant) |
| Avatars | `components/avatar.rs` | RESTYLE |
| Avatar Groups | `components/facepile.rs` | RESTYLE (already the right base) |
| Chips | `components/chip.rs` | RESTYLE |
| Dividers | `components/divider.rs` | RESTYLE |

### Functional

- **Button**: finish whichever variant(s) Phase 01 flagged as not-yet-token-correct. All 5 variants (primary/secondary/soft/ghost/danger) use `palette::primary(600/700)` (primary), `semantic::border/surface` (secondary), `palette::primary(50/700/100)` (soft), transparent+`palette::primary(600)` text (ghost), `palette::danger(600/700)` (danger). Sizes per `ButtonSize` enum unchanged (do not rename). Focus via `focus_ring()`. Update `preview()` to show full variant×size matrix.
- **Button Group** (new): segmented flex container, connected buttons (no gap, shared border, first/last child get outer rounding only) — `semantic::border` divider between buttons. Builder: `ButtonGroup::new().child(Button)...`.
- **Badge dot**: 6-8px colored circle (`palette::{role}(500)`) + label text, `rounded_full()`, reuse existing role-color match from solid/outline variants.
- **Avatar**: sizes per researcher-01 (24-64px), `rounded_full()`, initials/image/icon fallback, border via `semantic::border`, bg via `palette::neutral(200)` fallback tint per role if no image.
- **Facepile (avatar group)**: overlapping avatars (negative margin per researcher-01), overflow count badge (reuse `CountBadge` if it fits, else Badge dot-less numeric).
- **Chip**: label + optional icon + optional dismiss (`x-mark` icon), `rounded_full()` or `rounded_md()` per Tailwind spec, `semantic::surface`/`border`.
- **Divider**: horizontal/vertical, optional label (text centered with line on both sides), `semantic::border_muted`.

### Non-functional

- Keep files under 200 lines; `button_group.rs` new file, others are edits in place.
- No hardcoded hex/hsla in touched files — grep after editing to confirm zero literal color values remain.

## Architecture

```
crates/ui/src/components/
├── button/
│   ├── button_like.rs   (MODIFY — finish variant restyle per Phase01 finding)
│   ├── button.rs        (MODIFY — preview() update)
│   ├── icon_button.rs   (MODIFY — pass-through restyle check)
│   └── button_group.rs  (NEW)
├── badge.rs              (MODIFY — add dot variant)
├── avatar.rs             (MODIFY — restyle)
├── facepile.rs           (MODIFY — restyle, = avatar group)
├── chip.rs               (MODIFY — restyle)
└── divider.rs            (MODIFY — restyle)
```

## Related Code Files

**Modify:** all files listed above, plus `crates/ui/src/components.rs` (add `mod button_group;` + `pub use`), `crates/ui/src/prelude.rs` (export `ButtonGroup`).

## Implementation Steps

1. Apply Phase 01's Button restyle finding — fix only the flagged variant(s), verify others untouched.
2. Build `ButtonGroup` (new file) — connected button layout, shared border logic.
3. Add Badge dot variant (extend existing role-color match).
4. Restyle Avatar (sizes, fallback bg/text via palette, border via semantic).
5. Restyle Facepile (overlap margin, overflow count).
6. Restyle Chip (label+icon+dismiss).
7. Restyle Divider (horizontal/vertical/labeled).
8. Update every touched component's `fn preview()`.
9. `cargo check -p ui` clean; grep touched files for stray hex/hsla literals.
10. `cargo run -p ui_gallery` — confirm Elements page still renders without panic (full gallery wiring is Phase 9, but self-test via existing Elements page).

## Todo List

- [ ] Fix Button variant(s) per Phase01 finding
- [ ] Build ButtonGroup (new)
- [ ] Badge: add dot variant
- [ ] Restyle Avatar
- [ ] Restyle Facepile (avatar group)
- [ ] Restyle Chip
- [ ] Restyle Divider
- [ ] Update preview() for all 7
- [ ] cargo check -p ui clean, no stray color literals
- [ ] cargo run -p ui_gallery Elements page OK

## Success Criteria

- `make check` + `make check-all` + `cargo fmt --all --check` green.
- `cargo run -p ui_gallery` opens; Elements page shows Button (all variants/sizes), ButtonGroup, Badge (incl. dot), Avatar, Facepile, Chip, Divider without panic.
- Zero hardcoded hex/hsla literals in the 7 touched files (grep check).
- No regression to other components using `Button`/`Badge` elsewhere in the crate (`cargo check --workspace --all-targets`).

## Risk Assessment

- **Risk:** `ButtonGroup`'s shared-border/corner-rounding logic touches every child `Button`'s own rounding — risk of double borders. **Mitigation:** `ButtonGroup` overrides child corner radius via wrapper div clipping, not by mutating `Button`'s own style API.
- **Risk:** Facepile's existing overflow-count logic may already differ from Tailwind's exact spec (margin values). **Mitigation:** read `facepile.rs` fully before restyling; adjust values in place, don't restructure the count logic.

## Security Considerations

None — presentational components.

## Next Steps

- Phase 9 gallery wiring needs this phase's updated `preview()` functions for the Elements page.
- Phase 3's Multi-Select reuses Chip (this phase) — no blocking dependency, just note Chip's final API for Phase 3 to consume.
