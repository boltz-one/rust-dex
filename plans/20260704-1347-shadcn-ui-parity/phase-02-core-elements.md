---
title: "Phase 2 — Core elements"
status: pending
effort: 8h
---

# Phase 2: Core Elements

[← plan.md](./plan.md) | Prev: [phase-01](./phase-01-token-foundation.md) | Next: [phase-03](./phase-03-forms.md)

## Context
shadcn "Elements" category (16 components). Mostly small, visually-simple atoms — either align an existing file's variant naming to shadcn, or add a small new file. Depends on Phase 1's `ButtonVariant`/new semantic roles/`radius.rs`.

## Component Table

| Component | Codebase file | Action | Notes |
|---|---|---|---|
| Button | `button/button.rs` | Align | Wire the Phase-1 `.variant()/.size()` into gallery examples; verify visuals match shadcn `default/destructive/outline/secondary/ghost` |
| Button Group | `button/button_group.rs` | Align | Verify wrapper spacing/orientation matches shadcn `ButtonGroup` |
| Badge | `badge.rs` | Align | Current `BadgeVariant{Soft,Solid,Outline}` × `BadgeColor{Neutral,Primary,Success,Warning,Danger}` vs shadcn `default/secondary/destructive/outline` — add `BadgeColor::Secondary`/ensure `Solid+Danger`≈`destructive`, `Outline`≈`outline` are 1:1 reachable combos; additive only |
| Avatar | `avatar.rs` | Align | Verify sm/default/lg sizes + fallback/loading states present |
| Card | `card.rs` | Align | Verify Header/Title/Description/Content/Footer/Action anatomy sub-parts all exist as composable pieces |
| Separator | `divider.rs` | Align | Verify horizontal/vertical orientation prop |
| Skeleton | none | New | Trivial: animated pulsing rect, `div()` + opacity/bg animation (reuse `crates/ui/src/styles/animation.rs` pulse helper if one exists, else add minimal keyframe-less opacity oscillation) |
| Aspect Ratio | none | New | Trivial layout wrapper: fixed width/height ratio via `w_full().h(w * 1/ratio)` or gpui's aspect helper if present |
| Toggle | `toggle.rs` | Align | Verify `default/outline` variant + sm/default/lg sizes |
| Toggle Group | `toggle_button.rs` + `segmented_control.rs` | Align/Build | shadcn: single/multiple selection Root/Item; compose from existing two files rather than new one (DRY) |
| Kbd | none | New | Trivial styled `<kbd>`-like text chip, group variant for combos (e.g. "⌘K") |
| Spinner | none | New | Trivial: rotating icon/loader, sm/default/lg sizes |
| Typography | `typography.rs` | Align | Verify heading/paragraph/blockquote/code/list style presets all present |
| Empty | `empty_state.rs` | Align | Verify Header/Media/Title/Description/Content anatomy |
| Item | `list.rs` (closest primitive) | Align/Build | shadcn `Item` = generic Root/Media/Content/Actions row primitive; if `list.rs` is list-specific, add a standalone `item.rs` reusing the same internal row layout |
| Field | `form_field.rs` | Align | Verify Root/Label/Control/Description/Error anatomy; this underpins Phase 3's Form component |

## Key Insights
- All of these are ✅-feasible per research (native GPUI primitives, no exotic gesture/animation work). Fastest phase per component-count.
- `Toggle Group` and `Item` are the only two requiring real composition work; everything else is variant/token alignment or a small (<100 line) new file, per repo convention (`badge.rs` is 100 lines and is the reference pattern).
- Use Phase-1's `radius.rs` mapping and new semantic roles (`secondary_bg`, `muted_bg`, etc.) wherever a component needs shadcn's `secondary`/`muted`/`accent` look (e.g. `BadgeColor::Secondary`, `Skeleton`'s muted-bg pulse).

## Requirements
- No renames of existing public types — only additive enum variants / new builder methods / new files.
- Every component in this phase gets a `RegisterComponent` derive (mirror `badge.rs`) so it's discoverable in the component registry the gallery may use.
- New files follow `crates/ui/src/components/<name>.rs` flat pattern (not a subfolder) unless the component genuinely needs multiple files (only `button/`, `data_table/`, `progress/`, `label/`, `list/`, `notification/`, `icon/`, `collab/`, `ai/` currently use subfolders).

## Architecture
- `Skeleton`/`Spinner`/`Kbd`/`AspectRatio`: standalone files, `IntoElement`/`RenderOnce`, colors from `semantic`/`palette`, sizes from existing `LabelSize`-style size enum pattern if one is reused elsewhere (check `avatar.rs` for its `sm/default/lg` enum shape to mirror).
- `Item`: if built new, layout skeleton copied from `list.rs`'s row structure minus list-specific selection/virtualization logic.
- `Toggle Group`: new thin `toggle_group.rs` (or extend `segmented_control.rs`) exposing `Root`+`Item` builder API wrapping existing `toggle.rs` per-item rendering.

## Related Files
- `crates/ui/src/components/badge.rs` (pattern reference, also being aligned)
- `crates/ui/src/components/{avatar,card,divider,toggle,toggle_button,segmented_control,typography,empty_state,form_field,list}.rs`
- `crates/ui/src/components/button/{button,button_group}.rs`
- New: `crates/ui/src/components/{skeleton,aspect_ratio,kbd,spinner,item}.rs`, possibly `toggle_group.rs`
- `crates/ui/src/lib.rs` or module re-export file — register new modules

## Implementation Steps
1. For each "Align" row: open the file, diff its variant/size enum against the shadcn column in research catalog, add missing variants additively, update its gallery page entry.
2. For each "New" row: create the file per pattern, add to module tree, add gallery entry.
3. `Toggle Group`: decide compose-vs-extend, implement, gallery entry with an interactive `#[gpui::test]` (selection changes on click).
4. `Item`: decide new-file-vs-extend-list.rs, implement, gallery entry.
5. Run `cargo build -p ui` after every 3-4 components to catch issues early, not just at the end.

## Todo
- [ ] Button/ButtonGroup gallery examples using Phase-1 variant API
- [ ] Badge: add missing color/variant combos
- [ ] Avatar/Card/Separator/Toggle/Typography/Empty/Field: verify + note any gaps found
- [ ] Skeleton (new)
- [ ] Aspect Ratio (new)
- [ ] Kbd (new)
- [ ] Spinner (new)
- [ ] Toggle Group (compose)
- [ ] Item (compose/new)
- [ ] Gallery pages updated for all 16
- [ ] `cargo build -p ui` clean

## Success Criteria
- All 16 elements present, buildable, and shown in `examples/ui_gallery` (elements page).
- `cargo test -p ui` and `cargo fmt --all --check` pass.
- No existing public API renamed/removed.

## Risk & Dependencies
- Depends on Phase 1 semantic roles/radius.rs landing first.
- Low risk — no complex interaction/gesture code in this phase.

## Security
N/A — presentational components only.

## Next
[phase-03-forms.md](./phase-03-forms.md)
