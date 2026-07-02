---
title: "Tailwind UI Component Gallery + UI Kit Extension"
description: "Restyle rust-dex's GPUI ui crate to Tailwind Application UI design tokens and add missing components, showcased in a new ui_gallery example binary"
status: pending
priority: P2
effort: 28h
branch: main
tags: [frontend, ui, gpui, design-system]
created: 2026-07-02
---

# Tailwind UI Component Gallery + UI Kit Extension

## Overview

`crates/ui` has ~44 Zed/One-Dark-styled components. This plan restyles them to Tailwind
Application UI tokens (colors, spacing, radius, shadow-approx, typography, focus ring) and
adds missing components (text input, textarea, select, radio — checkbox/switch already exist
and get restyled). Deliverable: new `examples/ui_gallery` binary crate with sidebar navigation
showcasing every component, visually verified against tailwindcss.com/Tailwind UI screenshots.

`crates/app` (shippable hello-world) is untouched — `make dev`/`make check` behavior and
`default-members=["crates/app"]` stay as-is. Gallery only builds via `cargo run -p ui_gallery`.

## Phases

| # | Phase | Status | Effort | Link |
|---|-------|--------|--------|------|
| 1 | Design tokens + styling helpers | ✅ Done | 4h | [phase-01](./phase-01-design-tokens.md) |
| 2 | Core components (button/badge/card/alert) | 🟡 Partial — Badge/Card/Alert new; Button restyle pending | 4h | [phase-02](./phase-02-core-components.md) |
| 3 | Form controls | 🟡 Partial — TextInput/Textarea/Select/RadioButton done; Checkbox/Switch restyle pending | 4h | [phase-03](./phase-03-form-controls.md) |
| 4 | Composite/overlay components | ⬜ Pending — restyle Table/Modal/Dropdown/Tabs/Tooltip/Popover/Toast | 4h | [phase-04](./phase-04-composite-overlay.md) |
| 5 | Navigation + gallery app | ✅ Done — Navbar/Sidebar + ui_gallery (4 pages) + light/dark toggle | 4h | [phase-05](./phase-05-navigation-gallery.md) |

**Notes:** `semantic::*` made appearance-aware (palette-derived) so dark/light both work without authoring a second Theme; gallery navbar has a working light/dark toggle (flips `SystemAppearance`). Heroicons vendored via `icons::Assets` (rust-embed), wired through `application().with_assets(icons::Assets)`. Verified: `make check-all` + `make check` + `cargo fmt --all --check` green; `cargo run -p ui_gallery` opens a working window; no brand identifiers in code.

## Dependencies

- Phase 2-5 depend on Phase 1 tokens/helpers being in place.
- Phase 5 gallery app depends on Phase 2-4 components existing (can scaffold gallery shell early, wire showcases incrementally).
- Visual verify (Playwright + macOS offscreen screenshot) required at end of every phase — macOS-only, accepted limitation.

## Cross-Cutting Requirements (apply to EVERY phase)

- **Generic naming — NO brand-specific identifiers in code (user directive).** No `tw`/`tailwind`/`slate`/`blue`-style names in modules, types, fns, or fields. The design system is generic + rebrandable: `palette` (role ramps), `semantic` (theme-driven neutrals), `shadow`, `focus_ring`. Tailwind appears ONLY as the value-source reference in prose/doc-comments. **Palette is ROLE-based:** `palette::neutral/primary/success/warning/danger/info(shade)` — hex sourced from Tailwind internally (neutral←slate, primary/info←blue, success←green, warning←amber, danger←red; documented in `palette.rs`). ⚠️ Phase docs below still write color specs as Tailwind shade names (e.g. "bg blue-600", "border gray-300") — these are **spec VALUES**, not code identifiers: implement them as `palette::primary(600)` (accent) or `semantic::border(cx)` (neutral). Mapping: gray/slate/zinc→`semantic::*` (theme), blue→`primary`, red→`danger`, green→`success`, amber→`warning`.
- **Dark + light both (decision Q4).** **Neutrals** (surfaces, borders, text, hover bg) come from `semantic::*` reading `cx.theme().colors()` (light+dark aware), NOT hardcoded grays. **Accents/status** come from `palette` role ramps (mode-agnostic). Rule of thumb: a component NEVER hardcodes a neutral for bg/border/text — it calls `semantic::*`. Dark mode then works "for free" via the theme system.
- **Focus ring = true offset (decision Q2).** `focus_ring()` renders a **wrapping ring layer with a transparent gap** (outer `rounded` border-2 `palette::primary(500)` + inner offset padding), not a plain thick border. API wraps the element (returns a ring container) rather than mutating its own border.
- **Heroicons (decision Q3).** Vendor the needed Heroicons SVGs (MIT license) into `crates/icons` assets + register new `IconName` entries. Minimum set: `info`/`check-circle`/`exclamation-triangle`/`x-circle` (alerts+toast), `chevron-down`/`chevron-up-down` (select), `x-mark` (modal/toast close), `check` (checkbox). Phase 01 owns the vendoring; later phases just reference the new `IconName`s. Prefer exact Heroicon match over closest existing icon.
- **OKLCH→hsla (decision Q1, ACCEPTED).** Use Tailwind v3 hex → `hsla()`; accept minor precision loss vs runtime OKLCH. No further action.
- **Visual verify BOTH modes.** Every phase's visual-verify step screenshots the component in **light AND dark**; Tailwind UI reference (light) compared to app-light; app-dark checked for contrast/legibility (no washed-out or invisible elements).

## Key Codebase Facts (from research, do not re-derive)

- Workspace: `Cargo.toml` — add `"examples/ui_gallery"` to `members`, do NOT touch `default-members = ["crates/app"]`.
- Component pattern: `#[derive(IntoElement, Documented, RegisterComponent)]` struct + builder methods + `impl Component` with `fn preview()`. See `crates/ui/src/components/button/button.rs`.
- Theme colors: `crates/theme/src/styles/colors.rs` `ThemeColors` struct (Refineable, `Hsla` fields) — Tailwind palette added here as new fields, not a separate JSON system.
- Shadow: GPUI has `BoxShadow` (real, used via `ElevationIndex::shadow()` in `crates/ui/src/styles/elevation.rs`) — NOT a Tailwind-style `.shadow_*()` method. Must build a helper.
- Existing reusable bases: `Checkbox`/`Switch` (`toggle.rs`), `Callout` (alert base), `CountBadge` (badge base), `Modal`, `DropdownMenu`, `Tab`/`TabBar`, `Tooltip`, `Popover`, `announcement_toast.rs` (toast base), `data_table/table_row.rs` (table base).
- Genuinely missing: text input, textarea, select, radio button. Navbar/sidebar container also missing (gallery needs one anyway).
- Bootstrap pattern: `crates/app/src/main.rs` lines 67-99 (`application().run(...)` + `theme::init(LoadThemes::JustBase, cx)` + `theme::set_theme_settings_provider(...)` + `cx.open_window`).
- Visual verify: `crates/gpui/src/app/visual_test_context.rs` `VisualTestAppContext::open_offscreen_window()` → `RgbaImage`, macOS/Metal only.

## Resolved Decisions (user sign-off 2026-07-02)

- ✅ **OKLCH precision:** ACCEPTED — use v3 hex → `hsla()`, minor precision loss OK.
- ✅ **Focus ring:** Use **wrapper div for a true gapped ring** (not a plain thick border). See Cross-Cutting Requirements + Phase 01.
- ✅ **Icons:** **Add Heroicons SVGs** to `crates/icons` (MIT). Phase 01 vendors the minimum set + registers `IconName`s.
- ✅ **Dark/light:** **Support BOTH.** Neutrals from `ThemeColors` (theme-driven), accents from `palette`. Gallery gets a light/dark toggle; visual-verify covers both modes.

## Remaining Open Items (non-blocking, revisit during impl)

- **Visual verify loop performance:** VisualTestContext screenshot loop speed unknown for 40+ components × 2 modes — assume 1 screenshot per variant group per mode; batch into one offscreen frame if slow.
- **Theme neutral coverage:** confirm `ThemeColors` has all neutral roles needed (surface, elevated-surface, border, border-muted, text, text-muted, hover-bg). If missing, Phase 01 adds the role to the theme rather than hardcoding a palette gray.
