---
title: "Phase 1 — Token/theme foundation + gap matrix lock"
status: pending
effort: 6h
---

# Phase 1: Token/Theme Foundation

[← plan.md](./plan.md) | Next: [phase-02](./phase-02-core-elements.md)

## Context
Every later phase reads token roles and the Button variant-alias API that don't exist yet. This phase is the BLOCKING foundation: add missing shadcn-named semantic roles, a radius scale, and the additive Button API, without touching any of the ~130 existing `ButtonStyle`/`.primary()`/`.danger()`/`.soft()` call sites. Lane low-risk: purely additive tokens/aliases, no ADR needed.

## Key Insights (from research, do not re-derive)
- shadcn CSS vars: `--background/-foreground`, `--card/-foreground`, `--popover/-foreground`, `--primary/-foreground`, `--secondary/-foreground`, `--muted/-foreground`, `--accent/-foreground`, `--destructive(-foreground)`, `--border`, `--input`, `--ring`, `--radius` (+ `--sidebar-*`, `--chart-1..5` — out of scope here, sidebar/chart handled in their own phases if needed).
- Codebase has: `background/surface/elevated_surface/border/border_muted/border_focused/text/text_muted/text_placeholder/hover_bg/active_bg/icon/icon_muted` in `semantic.rs`, and `neutral/primary/success/warning/danger/info` ramps in `palette.rs`. Missing: `secondary` (always-visible neutral-solid bg+fg, NOT reuse primary), `muted` background (text side already covered by `text_muted`), `accent` as a standalone bg role (currently only exists as `hover_bg`/`active_bg` interaction state), `ring` (map to existing `border_focused`), `card`/`popover` (alias `surface`/`elevated_surface` — no new colors needed), `destructive` (alias `palette::danger`), radius scale.
- `ButtonStyle` renaming would touch **102 `ButtonStyle::` + 18 `.primary()` + 6 `.danger()` + 4 `.soft()`** call sites (~130 total, grep-verified) — ruled out. Additive alias API only.
- No `radius.rs` exists; components call gpui's own `.rounded_sm()/_md()/_lg()/_full()` directly (30+ verified call sites e.g. `card.rs:70`, `badge.rs:82`, `combobox.rs:78`). gpui's own utilities already ARE the Tailwind/shadcn-style radius scale — `radius.rs` should be a thin **doc/reference module** mapping shadcn's `--radius-sm/-md/-lg/-xl` names to which gpui method to call, not a new corner-radius runtime system (KISS/YAGNI — don't rebuild what gpui already provides).

## Requirements
1. Add new semantic roles to `crates/ui/src/styles/semantic.rs` (theme-aware, follow existing `fn name(cx: &App) -> Hsla` pattern):
   - `secondary_bg(cx)` / `secondary_fg(cx)` — new neutral-solid role (e.g. `neutral(100)`/`neutral(700)` light, theme-driven dark equivalent via `cx.theme().colors()` if a matching field exists, else compose from existing `ThemeColors` fields — verify against `crates/theme/src/styles/colors.rs` first).
   - `muted_bg(cx)` — background-only companion to existing `text_muted`.
   - `accent_bg(cx)` / `accent_fg(cx)` — standalone (non-hover-only) accent chip role, distinct from `hover_bg`.
   - `card(cx)` = alias of `surface(cx)`. `popover(cx)` = alias of `elevated_surface(cx)`. (Thin wrapper fns, not new colors — avoids DRY violation.)
   - `ring(cx)` = alias of `border_focused(cx)`.
   - `input_border(cx)` = alias of `border(cx)` unless `text_input.rs` reveals it should differ (verify open question #3 from token research: is `--input` visually distinct from `--border` in this codebase's rendering, or a purely nominal gap).
2. `destructive` role: do NOT add a new palette ramp — document `palette::danger` as the destructive color source (doc comment only, in `palette.rs` module doc or a short `## shadcn mapping` note).
3. New `crates/ui/src/styles/radius.rs`: doc-only reference table (const names or doc comments, e.g. `pub const RADIUS_SM_NOTE`, or just a module-doc mapping table) that maps shadcn `--radius-sm/-md/-lg/-xl` (base `--radius` with calc offsets) to the existing `rounded_sm()/rounded_md()/rounded_lg()/rounded_xl()` gpui builder calls new/aligned components should use. **Before finalizing**: re-verify the actual current shadcn `--radius-*` calc formula (research flagged the WebFetch summary as imprecise/unverified) — check `npx shadcn@latest init` output or shadcn's published `globals.css` rather than trusting the fetched summary numbers.
4. Button variant-alias API (additive, `crates/ui/src/components/button/button.rs` and/or `button_like.rs`):
   - Add `pub enum ButtonVariant { Default, Destructive, Outline, Secondary, Ghost, Link }` and `pub enum ButtonSizeAlias { Sm, Default, Lg, Icon }` (names TBD-final in impl, avoid colliding with existing `ButtonSize`).
   - Add `Button::variant(self, ButtonVariant) -> Self` that internally maps to existing `ButtonStyle`/`TintColor`: `Default→Filled+Tinted(Accent)` (mirror `.primary()`), `Destructive→Tinted(Error)` (mirror `.danger()`), `Outline→Outlined`, `Secondary→` **new** internal styling using the new `secondary_bg`/`secondary_fg` semantic roles (this is the one genuinely new visual, per research: `Subtle` is transparent-until-hover, not shadcn's always-visible muted-solid secondary), `Ghost→Transparent`, `Link→` delegate to existing `ButtonLink` component (document that calling `.variant(Link)` on `Button` either panics/warns-and-no-ops or is type-restricted — decide during impl; prefer compile-time: don't add `Link` to `ButtonVariant`, document `ButtonLink` as the shadcn-`link` equivalent instead, since it's already an architecturally separate component).
   - Add `Button::size(self, ButtonSizeAlias) -> Self` mapping `Sm→Compact, Default→Default, Lg→Medium|Large (pick one, document), Icon→` document that icon-only sizing is `IconButton`, not a `Button` size (architectural split, not a gap — keep as-is, just document in the new API's doc comment).
   - Existing `.primary()/.danger()/.soft()/.style()` (private) and all `ButtonStyle`/`TintColor` usages remain untouched and functional. Zero call-site edits.
5. Update the phase-06 open question tracker: confirm `--destructive-foreground` pairing exists in current shadcn template before Phase 3/4 destructive-styled components rely on a foreground pairing beyond what `palette::danger` + existing text-color logic already provides.

## Architecture
- All new fns/enums live in existing files (`semantic.rs`, `palette.rs` doc, new `radius.rs`, `button.rs`) — no new component crate, no new top-level module beyond `radius.rs`.
- `radius.rs` exported via `crates/ui/src/styles/mod.rs` (check current re-export pattern, mirror it — likely `pub mod radius;` + `pub use radius::*;` or similar to how `shadow.rs`/`elevation.rs` are exposed).
- `ButtonVariant`/`ButtonSizeAlias` exported via `crate::prelude::*` alongside existing `ButtonStyle`/`ButtonSize` (check `crates/ui/src/lib.rs` or `prelude.rs` for the re-export list pattern).

## Related Files
- `crates/ui/src/styles/semantic.rs` (add roles)
- `crates/ui/src/styles/palette.rs` (doc-only destructive mapping note)
- `crates/ui/src/styles/radius.rs` (new)
- `crates/ui/src/styles/mod.rs` or equivalent re-export point (verify exact file — read `crates/ui/src/styles.rs`/`mod.rs` first)
- `crates/ui/src/components/button/button.rs` (variant/size alias methods)
- `crates/ui/src/components/button/button_like.rs` (reference existing `ButtonStyle`/`TintColor`/`ButtonSize` at lines ~63, ~131, ~472 — do not modify these enums, only read)
- `crates/theme/src/styles/colors.rs` (`ThemeColors` struct — check for existing fields usable for `secondary`/`accent` before adding new theme-level fields; prefer deriving from `palette::neutral`/`palette::primary` if `ThemeColors` doesn't already carry a matching field, to avoid a theme-crate change)

## Implementation Steps
1. Read `crates/theme/src/styles/colors.rs` fully — confirm whether `ThemeColors` has fields matching `secondary`/`muted`/`accent` roles already (if yes, wire semantic fns to them; if no, derive from `palette.rs` ramps directly in `semantic.rs`, staying additive).
2. Add `secondary_bg/secondary_fg/muted_bg/accent_bg/accent_fg/card/popover/ring/input_border` fns to `semantic.rs` with doc comments citing the shadcn var they mirror.
3. Add doc-only destructive mapping note to `palette.rs`.
4. Create `radius.rs` with the verified `--radius-*` mapping table (re-check the calc formula first — don't trust the unverified WebFetch numbers from research).
5. Add `ButtonVariant`, `ButtonSizeAlias` enums + `.variant()`/`.size()` builder methods to `Button` in `button.rs`, delegating to existing private `fn style()`/`ButtonStyle`/`TintColor` — no changes to `button_like.rs` internals.
6. Wire new items into whatever prelude/mod re-export pattern the crate uses (grep existing `pub use` chains for `ButtonStyle` to mirror placement).
7. Add a doc-comment or short `docs/` note (per repo convention, check if `docs/design-guidelines.md` exists) capturing the final shadcn-var → codebase-role map as the source of truth for phases 2-6.
8. `cargo build -p ui` — confirm zero warnings/errors, zero changes needed in any other crate.

## Todo
- [ ] Confirm `ThemeColors` fields for secondary/muted/accent (read colors.rs)
- [ ] Add 8 new semantic role fns
- [ ] Add destructive doc note in palette.rs
- [ ] Create radius.rs with verified calc formula
- [ ] Add `ButtonVariant`/`ButtonSizeAlias` + `.variant()`/`.size()` on `Button`
- [ ] Re-export new items via existing prelude pattern
- [ ] `cargo build -p ui` clean, zero other-crate diffs
- [ ] Short shadcn-var→role map note for later phases to cite

## Success Criteria
- `cargo build -p ui` and `cargo test -p ui` pass with zero changes outside `crates/ui/src/styles/*` and `crates/ui/src/components/button/button.rs` (plus theme.rs only if step 1 requires it).
- Grep confirms all pre-existing `ButtonStyle::`/`.primary()`/`.danger()`/`.soft()` call sites (~130) are unchanged (`git diff --stat` shows no other crate touched).
- New `ButtonVariant`/`ButtonSizeAlias` compiles and a throwaway example (`Button::new("id","Save").variant(ButtonVariant::Secondary)`) renders without panic in a quick `cargo check` (full visual verify happens in Phase 7 gallery).

## Risk & Dependencies
- Risk: if `ThemeColors` lacks fields for secondary/accent, adding fields there ripples into `crates/theme` — mitigate by deriving from `palette.rs` in `semantic.rs` instead (no theme-crate change) unless dark-mode-specific tuning is truly required.
- Dependency: Phases 2-6 all read the roles/API added here — do not start them before this phase's `cargo build -p ui` is green.

## Security
N/A — pure styling/token additions, no user input, no new I/O.

## Next
[phase-02-core-elements.md](./phase-02-core-elements.md)
