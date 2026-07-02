# Phase 02 — Core Components (Button, Badge, Card, Alert)

## Context Links

- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-01-tailwind-spec.md` (§2.1, 2.3, 2.4, 2.5)
- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-02-gpui-codebase.md` (§3 component pattern)
- Phase 01: `./phase-01-design-tokens.md` (tokens this phase consumes)
- Existing: `crates/ui/src/components/button/{button.rs,button_like.rs,icon_button.rs}`, `crates/ui/src/components/count_badge.rs`, `crates/ui/src/components/callout.rs`

## Overview

- Date: 2026-07-02
- Description: Restyle `Button` (6 variants × 5 sizes + icon + states) and `CountBadge`→Badge variants using Tailwind tokens; build new `Card` component (none exists); restyle `Callout` as the 4-intent `Alert`.
- Priority: P1
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- **Cross-cutting (see plan.md):** neutral bg/border/text below (e.g. "bg white/surface", "border gray-300", "text gray-900", "hover bg gray-50") must be read from `semantic::{surface,border,text,hover_bg}` (theme-driven, dark+light), NOT hardcoded palette grays. Only accent colors (blue/red/green-600 etc.) come from `palette`. Focus states use Phase 01's gapped `focus_ring()` wrapper. Icons use the new Heroicon `IconName`s. Visual-verify in BOTH light and dark.
- `ButtonStyle` enum already has `Filled | Tinted(TintColor) | Outlined | OutlinedGhost | ...` (5 variants seen, verify full list in `button_like.rs` line ~125) — maps reasonably to Tailwind primary/secondary/soft/outline/ghost. Danger = `Tinted(TintColor::Negative)` or similar existing tint, confirm `TintColor` variants before assuming a new one is needed.
- `ButtonSize` enum: `Large | Medium | Default | Compact | None` (5 sizes) — maps to Tailwind xs/sm/md/lg/xl but names differ; do NOT rename the enum (breaks 44 existing call sites) — instead adjust the `rems()`/padding values per size to match Tailwind's px specs (researcher-01 §2.1), and add a "Restyle mapping" doc comment (`Compact≈xs, Default≈sm, Medium≈md, Large≈lg`, need a 5th for `xl` — check if gap exists).
- `CountBadge` is a numeric badge (likely just a pill with a number), NOT a general-purpose status/label badge. Confirm by reading its full source; if it's number-only, a NEW `Badge` component (solid/soft/outline/dot variants, text label) must be created — do not force-fit CountBadge.
- `Callout` already has `BorderPosition` enum and likely severity-based coloring — check if it already supports info/success/warning/error via `Severity` (from `crates/ui/src/styles/severity.rs`, exported in prelude). If so, this is a restyle-only task (adjust colors/spacing to Tailwind alert spec); if severity isn't wired, add it.
- No `Card` component exists anywhere in `components/`. This is net-new: simple `div()`-based struct, NOT over-engineered — header/body/footer as builder methods taking `impl IntoElement`.

## Requirements

### Functional

**Button** (`crates/ui/src/components/button/button_like.rs`, `button.rs`):
- Restyle `ButtonStyle::Filled` → primary (bg blue-600, hover blue-700, white text, shadow-sm, rounded-md).
- Restyle `ButtonStyle::Outlined` → secondary (bg white/surface, border gray-300, text gray-900, hover bg gray-50).
- Add/restyle a soft variant (bg blue-50, text blue-700, hover blue-100) — extend `TintColor` if no soft-style tint exists.
- Restyle `ButtonStyle::OutlinedGhost` or equivalent → ghost (transparent, text blue-600, hover bg blue-50).
- Danger variant via existing negative/error `TintColor` — restyle to red-600/red-700.
- Sizes: adjust `ButtonSize::rems()` px values to match xs(6/12) sm(8/14) md(10/16) lg(12/18) — add xl(14/20) if enum has room (5th variant `None`/unused slot) or document as out-of-scope with a note (do not break enum shape without checking all 44 call sites first).
- States: disabled (opacity 50%, cursor not-allowed — check existing `Disableable` trait wiring), loading (reuse existing spinner if `LoadingLabel` or similar exists — grep first), focus (apply `focus_ring()` from Phase 01).
- Icon button: confirm `icon_button.rs` restyle uses same variant/size system.

**Badge** (new `crates/ui/src/components/badge.rs` OR extend `count_badge.rs` if it's generic enough — verify first):
- Variants: solid (bg `{color}-100`, text `{color}-800`), soft (bg `{color}-50`, text `{color}-700`), outline (border `{color}-300`, text `{color}-700`, bg white/surface), dot (6px colored circle + text).
- Colors: gray/blue/red/green/amber, rounded-full, px-2 py-1, text-xs font-medium.
- `#[derive(IntoElement, Documented, RegisterComponent)]` following Button's pattern exactly.

**Card** (new `crates/ui/src/components/card.rs`):
- Base: bg surface, border gray-200 equivalent, rounded-lg, shadow-sm (via Phase 01 `Shadow::Sm`), padding 24px (p-6).
- Variants: elevated (shadow-md), bordered (no shadow, border only), flat (no border, no shadow).
- Builder methods: `.header(impl IntoElement)`, `.child(...)` (body via `ParentElement`), `.footer(impl IntoElement)`.
- Optional hover state (shadow-md on hover) only if a simple `.hoverable()` flag is trivial — do not build complex interactivity here (Card is a container, not a button).

**Alert** (restyle `crates/ui/src/components/callout.rs`, rename considerations avoided — keep `Callout` as the struct name if renaming breaks callers, just restyle):
- 4 intents: info (blue), success (green), warning (amber), error (red) — border-left 4px accent + bg-50 + text-800, using `Severity` enum if already wired, else add `Severity`-based color match.
- Layout: icon (16px, `crates/icons` closest match to Tailwind's info/check/exclamation/x-circle) + text + optional dismiss icon button.

### Non-functional

- Every component file stays under 200 lines; if `button_like.rs` restyle pushes it over, extract size/variant color-mapping into a small `button_style.rs` sibling.
- Reuse Phase 01 tokens exclusively — no new hardcoded hex/hsla in these files.
- Every component keeps its `RegisterComponent` + `fn preview()` — update `preview()` to demo all variants × sizes (needed for gallery in Phase 05).

## Architecture

```
crates/ui/src/components/
├── button/
│   ├── button_like.rs   (MODIFY — variant/size color+padding restyle)
│   ├── button.rs        (MODIFY — preview() update)
│   └── icon_button.rs   (MODIFY — restyle pass-through)
├── badge.rs              (NEW or MODIFY count_badge.rs — decide after reading source)
├── card.rs               (NEW)
└── callout.rs            (MODIFY — Tailwind alert restyle)
```

## Related Code Files

**Read first (decide new-vs-modify):**
- `crates/ui/src/components/count_badge.rs`
- `crates/ui/src/components/callout.rs`
- `crates/ui/src/styles/severity.rs`
- `crates/ui/src/components/button/button_like.rs` (full `TintColor` enum)

**Modify:**
- `crates/ui/src/components/button/button_like.rs`
- `crates/ui/src/components/button/button.rs`
- `crates/ui/src/components/button/icon_button.rs`
- `crates/ui/src/components/callout.rs`
- `crates/ui/src/prelude.rs` (export `Card`, `Badge` if new)

**Create (pending source check):**
- `crates/ui/src/components/card.rs`
- `crates/ui/src/components/badge.rs` (if `count_badge.rs` isn't reusable as general badge)

## Implementation Steps

1. Read `count_badge.rs`, `callout.rs`, `severity.rs`, and full `TintColor` enum in `button_like.rs` — decide restyle-only vs net-new for Badge/Alert.
2. Restyle `ButtonStyle::Filled`/`Outlined`/ghost variant colors — accents via `palette::primary(600)`/`primary(700)`, neutrals via `semantic::border(cx)`/`surface(cx)` — keep enum shape.
3. Adjust `ButtonSize::rems()`/padding to Tailwind px spec; add xl if room, else document gap.
4. Wire `focus_ring()` (Phase 01) into `ButtonLike` focus state render path.
5. Build/restyle Badge with 4 variants × 5 colors using Phase 01 palette + `rounded_full()`.
6. Build `Card` component with header/body/footer builders + 3 shadow variants via `Shadow`.
7. Restyle `Callout`/Alert with 4 `Severity`-driven color sets + left border accent + icon.
8. Update every touched component's `fn preview()` to show full variant/size matrix.
9. `cargo check -p ui` clean.
10. Visual verify: Playwright screenshot `tailwindui.com/components/application-ui/elements/buttons` (and badges/alerts pages) as reference; render each component's `preview()` in an offscreen GPUI window via `VisualTestAppContext`, screenshot; compare side-by-side, iterate on color/spacing until visually close.

## Todo List

- [ ] Read count_badge.rs / callout.rs / severity.rs / TintColor — decide new vs modify
- [ ] Restyle Button variants (primary/secondary/soft/ghost/danger) with Tailwind colors
- [ ] Restyle Button sizes to Tailwind px spec
- [ ] Wire focus_ring into ButtonLike
- [ ] Badge: 4 variants × colors, rounded-full
- [ ] Card: header/body/footer + 3 variants (elevated/bordered/flat)
- [ ] Alert: 4 severities, left-border accent, icon
- [ ] Update all preview() functions
- [ ] cargo check -p ui clean
- [ ] Visual verify vs Tailwind UI buttons/badges/alerts pages

## Success Criteria

- `cargo check -p ui` and `make check-all` both green.
- Button renders all variant×size combos correctly in `preview()` without panics.
- Badge/Card/Alert compile, render, and are exported via `ui::prelude::*`.
- Visual comparison (Playwright screenshot vs rendered GPUI screenshot) shows matching color families, spacing proportions, and border-radius — documented with a short before/after note.
- No regression in existing components that use `Button`/`Callout`/`CountBadge` elsewhere in the crate (grep call sites, `cargo check --workspace --all-targets`).

## Risk Assessment

- **Risk:** `ButtonSize` enum has only 5 slots but Tailwind needs 5 (xs-xl) — naming/count might already align by luck, or might be off by one. **Mitigation:** step 3 explicitly documents the mapping; if misaligned, prefer adjusting px values over renaming/adding enum variants (avoid breaking call sites).
- **Risk:** `count_badge.rs` might be tightly coupled to a specific numeric-only use case (e.g. unread counts) elsewhere in a caller not in this crate — changing it risks breaking unrelated call sites. **Mitigation:** step 1 read decides; prefer net-new `badge.rs` if any doubt.
- **Risk:** Restyling `Callout` colors might clash with existing non-Tailwind callers expecting current One-Dark palette. **Mitigation:** grep `Callout::new` call sites across workspace before restyling; since this is a UI-kit-wide restyle (per user's stated intent), expected to update all callers, but must not silently break compile.

## Security Considerations

None — presentational components, no data handling.

## Next Steps

- Phase 03 (form controls) will reuse `focus_ring()` and palette from Phase 01, and Badge's color-variant pattern as a template for input error states.
- Phase 05 gallery needs finalized `preview()` functions from this phase for the Elements showcase page.
