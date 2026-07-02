# Phase 01 — Design Tokens + Styling Helpers

## Context Links

- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-01-tailwind-spec.md` (section 1: tokens)
- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-02-gpui-codebase.md` (sections 1, 2)
- Skill: `.claude/skills/gpui-ui-design/references/layout-styling.md`
- Existing: `crates/theme/src/styles/colors.rs`, `crates/ui/src/styles/{color,elevation,spacing,typography,animation}.rs`

## Overview

- Date: 2026-07-02
- Description: Add Tailwind color palette, spacing/radius/shadow/typography constants, and a focus-ring helper trait to the `ui` crate as the foundation every later phase builds on.
- Priority: P1 (blocks all other phases)
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- `ThemeColors` (crates/theme/src/styles/colors.rs) is a `Refineable` struct of `Hsla` fields — semantic tokens (border, background, element_background, status colors), NOT a raw palette. Tailwind's raw palette (slate-50..950, blue-500, etc.) is a different concern: a **static swatch table**, not per-theme semantic colors.
- Decision: put the raw palette in `crates/ui/src/styles/palette.rs` (new file, plain Rust consts/fn, NOT in `ThemeColors`). Reason: these are fixed reference swatches (a design-token sheet), not theme-swappable semantics; adding 60+ fields to `ThemeColors` would bloat the theme system and force JSON schema updates for values that never change.
- **Generic naming (per user directive): palette is organized by ROLE, not brand color name.** Ramps: `neutral`, `primary`, `success`, `warning`, `danger`, `info` — each a `50..950` shade scale. Hex values are sourced from the Tailwind spec internally (neutral←slate, primary/info←blue, success←green, warning←amber, danger←red — mapping documented as a doc comment in `palette.rs`), but the public API is `palette::primary(600)` / `palette::neutral(200)` (or const fns `primary_600()`), never `blue_600()`. This keeps the design system reusable/rebrandable — swapping the underlying hex ramp re-themes every component without renaming call sites.
- GPUI has NO `.shadow_*()` Styled method. Real primitive is `gpui::BoxShadow { color, offset, blur_radius, spread_radius }`, consumed via a `shadow(Vec<BoxShadow>)` element method (verify exact method name in `crates/gpui/src/styled.rs` during implementation — `ElevationIndex::shadow()` in `crates/ui/src/styles/elevation.rs` returns `Vec<BoxShadow>` and is consumed somewhere; grep call sites first).
- No focus-ring primitive exists. **Decision Q2: build a TRUE gapped ring via a wrapper layer** (not a thick border). `fn focus_ring(content, focused, color) -> impl IntoElement` wraps the focusable content: outer `div().rounded_lg().border_2()` in ring color (`palette::primary(500)` default, `palette::danger(500)` for error) + inner `p(px(2.))` transparent offset gap + the content. When `!focused`, render the wrapper transparent (border_transparent, same padding) so layout doesn't shift. This yields Tailwind's `ring-2 ring-offset-2` look (gap between element and ring). Because it wraps, it's a helper fn / element (returns a container), not a `Self`-mutating `Styled` ext.
- **Decision Q4 — dark/light semantic strategy (CRITICAL, applies to all phases):** components must NOT hardcode neutral grays. Define a `semantic` helper module: `fn surface(cx)`, `elevated_surface(cx)`, `border(cx)`, `border_muted(cx)`, `text(cx)`, `text_muted(cx)`, `hover_bg(cx)` — each reads `cx.theme().colors()` (light+dark aware). Only accent/status colors (`palette::primary/success/warning/danger/info`) come from the palette. First: audit `ThemeColors` (`crates/theme/src/styles/colors.rs`) for each neutral role; if a role is missing, add it to the theme (both light+dark fallback theme) rather than hardcoding.
- **Decision Q3 — Heroicons:** vendor MIT-licensed Heroicons SVGs into `crates/icons` assets + register new `IconName` entries. Minimum set this phase: `info`, `check-circle`, `exclamation-triangle`, `x-circle`, `chevron-down`, `chevron-up-down`, `x-mark`, `check`. Inspect `crates/icons` to learn how existing SVGs are embedded + how `IconName` maps to asset paths before adding.
- No animation `.transition_*()` shorthand. `AnimationDuration` (Instant/Fast/Slow, in `animation.rs`) already maps loosely to Tailwind's 50/150/300ms — reuse as-is, do not invent new duration enum.

## Requirements

### Functional

- New module `crates/ui/src/styles/palette.rs`: **role-based ramps** `neutral`, `primary`, `success`, `warning`, `danger`, `info`, each full `50..950` scale (values from researcher-01 §1.1, mapped neutral←slate, primary/info←blue, success←green, warning←amber, danger←red — document mapping in-file). API `palette::primary(600)` etc. Pull the full ramp per role (not just 500-700) — Phase 2 badges need 50/100/800 shades.
- New module `crates/ui/src/styles/spacing.rs` (or extend existing `spacing.rs`): radius ramp (none/xs/sm/md/lg/xl/2xl/3xl/full) as `px()` constants; confirm existing `rounded_*()` GPUI methods already cover this — if GPUI's `rounded_sm/md/lg/xl/2xl/3xl/full()` already match Tailwind's px values (they likely do, GPUI is Tailwind-inspired), DOCUMENT the mapping instead of duplicating; only add missing sizes (`xs` = 2px if absent).
- New module `crates/ui/src/styles/shadow.rs`: `enum Shadow { Sm, Md, Lg, Xl }` + `fn box_shadows(self) -> Vec<BoxShadow>` returning hardcoded values from researcher-01 §1.5 (converted rgba → Hsla). Provide `trait StyledShadow: Styled { fn shadow_level(self, level: Shadow) -> Self }` if GPUI exposes a shadow-setter method; otherwise document as element-level `.shadow(...)` param passed at construction (check `Div` API — may require `.shadow(smallvec![...])` builder call, not a trait ext).
- New `crates/ui/src/styles/focus_ring.rs`: **gapped-ring wrapper** `fn focus_ring(content: impl IntoElement, focused: bool, color: Hsla) -> impl IntoElement` — outer `div().rounded_lg().border_2()` (color when focused, transparent otherwise) + inner `p(px(2.))` gap + content; layout stable in both states. Provide `focus_ring_error()` convenience (`palette::danger(500)`). Doc-comment the ring-offset approach (decision Q2).
- New `crates/ui/src/styles/semantic.rs` (**decision Q4**): `surface/elevated_surface/border/border_muted/text/text_muted/hover_bg` fns reading `cx.theme().colors()` — the neutral layer every component uses so dark/light both work. Extend `ThemeColors`/fallback theme only if a neutral role is genuinely absent.
- Heroicons (**decision Q3**): add the 8 SVGs listed in Key Insights to `crates/icons` + `IconName` variants, matching the crate's existing embed pattern.
- Typography: confirm existing `TextSize`/`StyledTypography` (`crates/ui/src/styles/typography.rs`) sizes (`text_ui_sm/lg/xl` etc.) map close enough to Tailwind xs/sm/base/lg/xl/2xl/3xl (researcher-01 §1.4). Add any missing size variant; do not replace existing enum (avoid breaking ~44 existing components that use it).
- Export everything through `crates/ui/src/styles.rs` (`pub use palette::*;` etc.) and re-export needed items via `crates/ui/src/prelude.rs` so later phases just `use ui::prelude::*`.

### Non-functional

- File size: one `palette.rs` (~150 lines for 6 roles × ~11 shades) fits under 200; if not, split neutral vs accent roles into two files.
- Naming: NO brand-specific identifiers anywhere (no `tw`/`tailwind`/`slate`/`blue` in public API). Roles + semantic names only. Tailwind may appear in prose/doc-comments as the value-source reference, never as a code identifier.
- No `unwrap()`; palette is pure data, no fallible operations expected.
- Must not modify `ThemeColors` struct or theme JSON schema (per palette-location decision above) — zero risk to existing theme loading.

## Architecture

```
crates/ui/src/styles/
├── palette.rs   (NEW) — role ramps: neutral/primary/success/warning/danger/info (Hsla, mode-agnostic)
├── shadow.rs    (NEW) — Shadow enum + BoxShadow vectors
├── focus_ring.rs         (NEW) — gapped-ring wrapper fn (Q2) + focus_ring_error
├── semantic.rs        (NEW) — neutral roles from cx.theme() for dark/light (Q4)
├── color.rs              (existing, untouched — semantic Color enum)
├── elevation.rs          (existing, untouched — ElevationIndex)
├── spacing.rs            (existing — extend only if radius gap found)
├── typography.rs         (existing — extend only if size gap found)
└── animation.rs          (existing, untouched — reused as-is)
```

Palette values are `fn` returning `Hsla` (via `gpui::hsla()` from hex, or `rgb()` helper if GPUI has one — check `crates/gpui/src/color.rs` for a hex-to-hsla helper before hand-converting 60+ values manually) rather than `const` if `hsla()` isn't const-fn-compatible.

## Related Code Files

**Create:**
- `crates/ui/src/styles/palette.rs`
- `crates/ui/src/styles/shadow.rs`
- `crates/ui/src/styles/focus_ring.rs`
- `crates/ui/src/styles/semantic.rs` (Q4 neutral layer)

**Modify:**
- `crates/ui/src/styles.rs` — add `pub mod palette; shadow; focus_ring; semantic;` + re-exports
- `crates/ui/src/prelude.rs` — re-export palette/shadow/focus-ring/semantic items needed by components
- `crates/icons/` — add 8 Heroicons SVGs + `IconName` variants (Q3)
- `crates/theme/src/styles/colors.rs` + fallback theme — only if a neutral role is missing (Q4 audit)
- `crates/ui/src/styles/spacing.rs` — only if radius gap found
- `crates/ui/src/styles/typography.rs` — only if size gap found

## Implementation Steps

1. Grep `crates/gpui/src/color.rs` and `crates/gpui/src/styled.rs` for existing hex→Hsla helper and any `.shadow(...)` builder method; document findings as code comments before writing palette (avoid guessing API).
2. Write `palette.rs`: role ramps `neutral/primary/success/warning/danger/info`, each shade as a fn (e.g. `neutral(50)..neutral(950)` or const `primary_500()`), function form if `hsla()` needs runtime float math. Document the role→source-hex mapping (neutral←slate, primary/info←blue, success←green, warning←amber, danger←red) as an in-file comment.
3. Write `shadow.rs`: `Shadow` enum + `box_shadows()` match arm per researcher-01 §1.5 values, alpha converted from `rgba(0,0,0,X)` to `hsla(0.,0.,0.,X)`.
4. Write `focus_ring.rs`: gapped-ring wrapper `focus_ring(content, focused, color)` + `focus_ring_error()` (returns wrapping element, not a `Styled` ext).
4b. Write `semantic.rs`: neutral-role fns (`surface/elevated_surface/border/border_muted/text/text_muted/hover_bg`) reading `cx.theme().colors()`; audit `ThemeColors` first, add missing role to theme if absent.
4c. Vendor 8 Heroicons SVGs + `IconName` variants into `crates/icons` (inspect existing embed pattern first).
5. Wire exports in `styles.rs` and `prelude.rs`.
6. Compare existing `rounded_*()` px values (grep `crates/gpui/src/styled.rs`) against researcher-01 §1.3 table; note any mismatch as a doc comment (do not silently redefine).
7. Compare `TextSize` rem values (`typography.rs`) against researcher-01 §1.4; add missing size if any gap (e.g. no `3xl`).
8. `cargo check -p ui` — must compile clean, zero warnings from new files.
9. Visual verify: write a tiny throwaway `#[cfg(test)]` or scratch binary rendering a swatch grid (all palette colors as colored boxes) in an offscreen window; screenshot via `VisualTestAppContext`; open Playwright to `https://tailwindcss.com/docs/colors`, screenshot the color reference table; compare visually (hue/tone match, not pixel-exact) — fix any obviously wrong hsla conversion.

## Todo List

- [ ] Grep GPUI for hex-to-hsla helper + shadow builder method, document findings
- [ ] Create `palette.rs` with full neutral + semantic ramps
- [ ] Create `shadow.rs` with sm/md/lg/xl `BoxShadow` vectors
- [ ] Create `focus_ring.rs` gapped-ring wrapper (Q2) + `focus_ring_error`
- [ ] Audit `ThemeColors` neutral roles; create `semantic.rs` (Q4); add missing role to theme if needed
- [ ] Vendor 8 Heroicons SVGs + `IconName` variants into `crates/icons` (Q3)
- [ ] Wire `styles.rs` module declarations + re-exports
- [ ] Wire `prelude.rs` re-exports
- [ ] Reconcile radius ramp (extend `spacing.rs` only if gap)
- [ ] Reconcile typography ramp (extend `typography.rs` only if gap)
- [ ] `cargo check -p ui` + `cargo check -p icons` clean
- [ ] Visual verify: palette swatch screenshot vs tailwindcss.com/docs/colors; render a focus-ring + semantic-surface demo in BOTH light and dark

## Success Criteria

- `cargo check -p ui --features gpui_platform/runtime_shaders` (or plain `cargo check -p ui`) compiles with zero errors/new warnings.
- All 8 color families (slate/gray/zinc/blue/indigo/red/green/amber) have full shade ramps accessible as `ui::palette::<color>_<shade>()`.
- `focus_ring()` wrapper renders a visible gapped ring around content when focused, layout-stable when unfocused (scratch demo).
- `semantic::{surface,border,text,...}` return correct colors in BOTH light and dark (`cx.theme()` driven); scratch demo legible in both modes.
- 8 Heroicons render via their new `IconName` variants (`cargo check -p icons` clean, icons visible in a scratch render).
- Visual swatch screenshot side-by-side with tailwindcss.com reference shows matching hues (manual visual check, documented with before/after note in phase file or PR).
- No existing component broken: `cargo check --workspace --all-targets` (i.e. `make check-all`) still green.

## Risk Assessment

- **Risk:** `ThemeColors` extension temptation creeping in during later phases (someone adds palette fields there instead of using `palette` module) → drifts from this phase's architecture decision. **Mitigation:** document decision clearly in this file + plan.md; code review checks for it.
- **Risk:** Hex→HSLA conversion math errors (manual conversion of 60+ values). **Mitigation:** use a small conversion script (Python/Rust one-off) rather than hand-computing each value; verify a handful against known references (e.g. blue-500 `#3b82f6` should be H≈217°, S≈91%, L≈60%).
- **Risk:** GPUI `.shadow()` API might not exist as assumed by researcher-02 (only `ElevationIndex::shadow()` confirmed, its consumer unclear). **Mitigation:** step 1 grep must confirm before writing `shadow.rs`; if no consumer method exists, this phase must find/add one (check `Div`'s `Styled` impl or `StyleRefinement` struct in `crates/gpui/src/style.rs` for a `box_shadow` field).

## Security Considerations

None — pure styling/data, no user input, no I/O, no auth surface.

## Next Steps

- Phase 02 (core components) depends on `palette`, `shadow`, `focus_ring` all compiling and exported.
- If shadow API investigation (step 1) reveals a different mechanism than assumed, update phase-02/04 implementation steps accordingly before starting them.
