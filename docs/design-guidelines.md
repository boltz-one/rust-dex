# Design Guidelines — boltz UI/UX

Source of truth for the boltz design system. Phases 01–06 implement against these values; do not re-derive.

## Overview

Dark-only. Target = **Zed 1:1**: tokens, spacing, type, and component style take Zed as the exact standard. **One Dark kept** (Zed-accurate, not rebuilt) — never re-derive hex. Accent = Zed blue. Motion = `AnimationDuration::Fast` (150ms) for hover/expand/popover. Every value below is concrete (hex/rem/px/ms).

## Color Tokens

Resolved from `default_dark()` (= `one_dark` theme, `crates/theme/src/fallback_themes.rs:120`). Hex computed precisely from the HSL via `colorsys` (rounded to `#rrggbb`).

| Field | HSL | Hex | Used for |
|------|-----|-----|----------|
| `background` | hsla(215°,12%,15%) | `#22252b` | chrome bg, status/title bar |
| `surface_background` | hsla(215°,12%,15%) | `#22252b` | surfaces |
| `elevated_surface_background` | hsla(225°,12%,17%) | `#262931` | elevated surfaces |
| `editor_background` | hsla(220°,12%,18%) | `#282c33` | editor / message area |
| `border` | hsla(225°,13%,12%) | `#1b1d23` | default border |
| `border_variant` | hsla(228°,8%,25%) | `#3b3d45` | subtle border |
| `border_focused` | hsla(223°,78%,65%) | `#6088eb` | focus ring (accent blue) |
| `border_selected` | hsla(222.6°,77.5%,65.1%) | `#6189eb` | selected border |
| `text` | hsla(221°,11%,86%) | `#d7dadf` | primary text |
| `text_muted` | hsla(218°,7%,46%) | `#6d737e` | secondary text |
| `text_placeholder` | hsla(220°,6.6%,44.5%) | `#6a6f79` | placeholder text |
| `text_accent` | hsla(222.6°,77.5%,65.1%) | `#6189eb` | accent text (Zed blue) |
| `icon` | hsla(222.9°,9.9%,86.1%) | `#d8dadf` | default icon |
| `icon_muted` | hsla(220°,12.1%,66.1%) | `#9ea5b3` | muted icon |
| `element_background` | hsla(223°,13%,21%) | `#2f333d` | element bg |
| `element_hover` | hsla(225°,11.8%,26.7%) | `#3c404c` | hover bg |
| `element_active` | hsla(220°,11.8%,20%) | `#2d3139` | active/press bg |
| `element_selected` | hsla(224°,11.3%,26.1%) | `#3b3f4a` | selected bg |
| `element_selection_background` | player.local().selection @ α0.25 | translucent | selection highlight |
| `drop_target_background` | hsla(220°,8.3%,21.4%) | `#32353b` | drop target |
| `panel_background` | hsla(215°,12%,15%) | `#22252b` | panel bg |
| `tab_bar_background` | hsla(215°,12%,15%) | `#22252b` | tab bar bg |
| `tab_active_background` | hsla(220°,12%,18%) | `#282c33` | active tab bg |
| `toolbar_background` | hsla(220°,12%,18%) | `#282c33` | toolbar bg |
| `scrollbar_track_background` | transparent | transparent | scrollbar track |
| `scrollbar_thumb_background` | transparent | transparent | scrollbar thumb |
| `scrollbar_thumb_hover_background` | hsla(225°,11.8%,26.7%) | `#3c404c` | scrollbar thumb hover |

### WCAG contrast (vs `background` `#22252b`)

| Token | Ratio | Target | Result |
|-------|-------|--------|--------|
| `text` `#d7dadf` | 10.92:1 | ≥4.5:1 | PASS |
| `text_muted` `#6d737e` | 3.21:1 | ≥3:1 | PASS |
| `text_placeholder` `#6a6f79` | 3.03:1 | ≥3:1 | PASS |
| `icon` `#d8dadf` | 10.95:1 | ≥3:1 | PASS |
| `icon_muted` `#9ea5b3` | 6.19:1 | ≥3:1 | PASS |
| `text_accent` `#6189eb` | 4.58:1 | ≥3:1 | PASS |

All pass — no token adjustment needed.

## Type Scale

### UI text (`TextSize`, `crates/ui/src/styles/typography.rs`) — unchanged

| Size | Rem | Px @16 | Helper |
|------|-----|--------|--------|
| Large | 1.0 | 16 | `text_ui_lg` |
| Default | 0.825 | 14 | `text_ui` |
| Small | 0.75 | 12 | `text_ui_sm` |
| XSmall | 0.625 | 10 | `text_ui_xs` |

### Markdown headings (`crates/markdown/src/style.rs:271-297`) — adds weight

| Level | Agent rem | Agent weight | Preview rem |
|-------|-----------|--------------|-------------|
| h1 | 1.15 | 600 (SEMIBOLD) | 1.45 |
| h2 | 1.1 | 600 (SEMIBOLD) | 1.3 |
| h3 | 1.05 | 600 (SEMIBOLD) | 1.1 |
| h4 | 1.0 | 500 (MEDIUM) | 1.01 |
| h5 | 0.95 | 500 (MEDIUM) | 0.95 |
| h6 | 0.875 | 500 (MEDIUM) | 0.85 |

The `font_weight` column is the fix — current code sets `font_size` only, producing a flat hierarchy.

### Line-height

Markdown body line-height changes from `×1.75` to `×1.5` (`style.rs:186`). UI text line-height unchanged. Applies to both Agent and Preview `MarkdownFont` variants (shared `line_height` binding).

### Headline component (`HeadlineSize`, unchanged)

XS=0.88rem, S=1.0rem, M=1.125rem, L=1.27rem, XL=1.43rem; line-height 1.6.

## Spacing

`DynamicSpacing` (`crates/ui/src/styles/spacing.rs`) is the **only** allowed spacing API. Scale (compact, default, comfortable):

| Variant | Compact | Default | Comfortable |
|---------|---------|---------|-------------|
| Base00 | 0 | 0 | 0 |
| Base01 | 1 | 1 | 2 |
| Base02 | 1 | 2 | 4 |
| Base03 | 2 | 3 | 4 |
| Base04 | 2 | 4 | 6 |
| Base06 | 3 | 6 | 8 |
| Base08 | 4 | 8 | 10 |
| Base12 | 10 | 12 | 14 |
| Base16 | 14 | 16 | 18 |
| Base20 | 18 | 20 | 22 |
| Base24 | 24 | 24 | 24 |
| Base32 | 32 | 32 | 32 |
| Base40 | 40 | 40 | 40 |
| Base48 | 48 | 48 | 48 |

**Policy:** `DynamicSpacing` only in new/touched code. Raw `px_N()`/`gap_N()`/`p_N()` Tailwind-style helpers are banned in touched files. No repo-wide sweep — normalize only where a file is already being edited (YAGNI).

## Component State Matrix

Canonical pattern = `ButtonLike` (`crates/ui/src/components/button/button_like.rs:263-420`). Each primitive computes per-state style via `hovered()`/`active()`/`focused()`/`disabled()` returning a styles struct backed by semantic tokens.

| State | Background | Border | Text/Icon | Motion |
|-------|-----------|--------|-----------|--------|
| default | `element_background` / transparent | `border` / transparent | `text` / `icon` | — |
| hover | `element_hover` / `ghost_element_hover` | `border` | `text` / `icon` | Fast 150ms |
| active/press | `element_active` / `ghost_element_active` | `border_variant` | `text` / `icon` | Fast 150ms |
| selected | `element_selected` | `border_selected` | `text` | — |
| focus | `element_background` / `ghost_element_background` | `border_focused` | `text` | — |
| disabled | `element_disabled` (reduced alpha) | `border_disabled` | `text_disabled` / `icon_disabled`, no `cursor_pointer` | — |
| loading | `element_background` + `IconName::LoadCircle` (spinning) | `border` | `text` | Fast 150ms spin |

Phase-04 primitives that **must match** this matrix: Tab, Card, DropdownMenu, Badge, Label, TextInput, Divider. Divider has no hover/active/loading (static) — states N/A.

## Motion

`AnimationDuration` (`crates/ui/src/styles/animation.rs`): Instant=50ms, **Fast=150ms**, Slow=300ms. Curve = `ease_out_quint` (used by `DefaultAnimations`).

| Interaction | Duration | Curve |
|-------------|----------|-------|
| Button hover bg fade | 150ms | ease_out_quint |
| Tab switch | 150ms | ease_out_quint |
| Disclosure / tool-call card expand | 150ms | ease_out_quint |
| Thinking block expand | 150ms | ease_out_quint |
| Streaming indicator appear | 150ms | ease_out_quint |

No new animation primitives. Static elements (Divider) get no motion.

## EmptyState (drop-in contract for phase-06)

`EmptyState` (`crates/ui/src/components/empty_state.rs`) is the canonical empty-placeholder component. Phase-06 wires it into the agent transcript for the zero-entries case.

```rust
EmptyState::new("No conversation yet")
    .icon(IconName::Sparkle)         // agent-appropriate icon (any referenced IconName)
    .description("Start a conversation to see messages here.")
    .action(Button::new("new", "New Thread"))  // optional CTA
```

API: `new(heading)` → `.icon(IconName)` → `.description(text)` → `.action(element)`. All optional except heading. Centered, muted icon + MEDIUM-weight heading + Small muted description. Drop it in wherever a list/transcript is empty — no new component needed.

## Icon Policy

- Source: Heroicons (MIT), `optimized/24/outline/<name>.svg`. License already documented at `crates/icons/src/icons.rs:292`.
- Filename = snake_case = `to_snake_case(IconNameVariant)`. `IconName` derives `strum` `#[strum(serialize_all = "snake_case")]` + `IntoStaticStr`, so `path()` already maps variants to `icons/<snake>.svg`.
- Every `IconName` variant referenced in `base/crates/ui/src` or `terminal/src` must resolve to a real embedded SVG. Phase-01 enforces with a `#[test]` asserting each referenced variant has an asset (iterates referenced set, not the full enum).
- Known gap: ~215 of 269 enum variants have **no** SVG. Documented as a known gap — not safe to use until backfilled. This plan backfills only the live-broken set (`LoadCircle`, `ToolHammer`, `ToolThink`, `Sparkle`) + the agent-UX set phase-05 needs (YAGNI).

## Decisions (resolved — do not re-litigate)

1. Zed 1:1 — tokens/spacing/type/component style take Zed as exact standard.
2. Dark-only — `ThemeColors::dark()` is default + only theme; no light mode.
3. One Dark kept — Zed-accurate; never re-derive hex.
4. Accent = Zed blue (`#6189eb` / `border_focused` `#6088eb`).
5. Full 7-phase scope; no phase cut or deprioritized.
6. Icon backfill = Heroicons subset (live-broken + agent-UX), not all 269.
7. Motion v1 = `AnimationDuration::Fast` (150ms) for hover/expand/popover; no new animation primitives.
