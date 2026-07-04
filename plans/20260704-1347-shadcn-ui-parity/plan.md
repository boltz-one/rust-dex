---
title: "shadcn/ui Parity Port for GPUI UI Kit"
description: "Align API/token/style of crates/ui with shadcn/ui catalog (~50 components), add what's missing, into the one existing kit"
status: pending
priority: P1
effort: 54h
branch: main
tags: [ui, gpui, design-system, shadcn, low-risk]
created: 2026-07-04
---

# shadcn/ui Parity Port — crates/ui

## Overview
Port shadcn/ui's full component catalog (~50, incl. heavy ones) into the existing `crates/ui` kit — one kit, no new crate. Where a shadcn component overlaps an existing one, ALIGN its variant/size/token API and visuals to shadcn naming (additive, non-breaking). Where missing, ADD it following the same architecture (`crates/ui/src/components/*.rs`, `IntoElement`/`RenderOnce`, `RegisterComponent`, `crate::prelude::*`, colors from `palette`/`semantic`, pattern ref `crates/ui/src/components/badge.rs`).

Lane: frontend/ui/design-system, low-risk — no ADR (additive component/token work, no data/security/infra impact).

## Phases

| # | Phase | Status | Effort | Link |
|---|-------|--------|--------|------|
| 1 | Token/theme foundation + gap matrix lock | ⬜ pending | 6h | [phase-01](./phase-01-token-foundation.md) |
| 2 | Core elements (16) | ⬜ pending | 8h | [phase-02](./phase-02-core-elements.md) |
| 3 | Forms (12 + 1 skip) | ⬜ pending | 10h | [phase-03](./phase-03-forms.md) |
| 4 | Overlays (11 + 1 skip) | ⬜ pending | 10h | [phase-04](./phase-04-overlays.md) |
| 5 | Data & Navigation (13) | ⬜ pending | 8h | [phase-05](./phase-05-data-nav.md) |
| 6 | Advanced/heavy — go/no-go (5) | ⬜ pending | 8h | [phase-06](./phase-06-advanced-heavy.md) |
| 7 | Gallery, harness, final verify | ⬜ pending | 4h | [phase-07](./phase-07-gallery-verify.md) |

## Dependencies
Phase 1 **BLOCKS** 2-6 (token roles + Button variant-alias API + radius scale must exist first; every later phase's components read these). Phases 2-5 are independent of each other post-P1 (touch disjoint files) — can run in parallel if multiple executors. Phase 6 independent but benefits from P2-P5 patterns (Popover/Command reuse). Phase 7 depends on all of 2-6 landing (gallery pages reference every new/aligned component).

## Cross-Cutting Rules (apply to every phase)
- shadcn variant/size/token **names** are a generic design API — fine to adopt. Color **values** still come only from `palette.rs`/`semantic.rs`; never hardcode hex/hsla in a component.
- Non-breaking: component changes are additive (new builder methods/enum variants) or aliases. If a rename is unavoidable, update every call site + gallery + harness **in the same phase** — no dangling breakage.
- Do not touch `crates/app` or root `default-members` (still `["crates/app"]`).
- Every component (new or aligned) gets a page/section in `examples/ui_gallery`; interactive ones get a `#[gpui::test]` + `TestAppContext` case (pattern: `examples/ui_gallery/tests/visual_harness.rs`, `crates/ui/src/components/context_menu.rs`). Overlays use existing deferred+anchored+occlude pattern (`popover.rs`/`modal.rs`) — don't invent a new overlay primitive.
- Reuse existing composites: Modal→Dialog/AlertDialog/Sheet, Popover→HoverCard, DropdownMenu→Menubar base, Combobox→Command base, notification.rs→Sonner base, data_table.rs→Table.
- Heavy/infeasible components: document the concrete limitation (missing crate dependency, unverified GPUI capability) — never silently claim done.
- Gate: `make check`, `cargo fmt --all --check`, `cargo test -p ui`, `cargo test -p ui_gallery` must stay green at the end of every phase.

## Key Codebase Facts (from research, don't re-derive)
- `crates/ui/src/components/button/button_like.rs`: `ButtonStyle` (`Filled/Tinted(TintColor)/Outlined/OutlinedGhost/OutlinedCustom/Subtle/Transparent`), `TintColor` (`Accent/Error/Warning/Success`), `ButtonSize` (`Large/Medium/Default/Compact/None`). `button.rs` wraps it with `.primary()/.danger()/.soft()` convenience + private `fn style()` (line 360). **102 `ButtonStyle::` + 18 `.primary()` + 6 `.danger()` + 4 `.soft()` call sites** — renaming is ~130-site churn, ruled out; additive alias API only (see phase-01).
- `crates/ui/src/styles/palette.rs`: mode-agnostic ramps `neutral/primary/success/warning/danger/info` (Tailwind hex, 50-950). `crates/ui/src/styles/semantic.rs`: theme-aware neutral roles reading `cx.theme().colors()` (`crates/theme/src/styles/colors.rs`) — `background/surface/elevated_surface/border/border_muted/border_focused/text/text_muted/text_placeholder/hover_bg/active_bg/icon/icon_muted`. No `secondary`/`muted_bg`/`accent`(standalone)/`ring`/`radius` roles exist yet.
- No centralized radius scale: components call gpui's own `.rounded_sm()/_md()/_lg()/_full()` ad hoc per instance (verified via grep, 30+ call sites). `radius.rs` should be a **thin mapping/doc module** (shadcn `--radius` step → which existing `rounded_*` call to use), not a new corner-radius engine — gpui's utilities already are the Tailwind-style scale shadcn expects.
- shadcn now ships OKLCH color values; codebase keeps hex→Hsla via `palette.rs`. Per prior decision: **keep hex/Hsla, treat shadcn OKLCH as value-source reference only** — no OKLCH rework.
- Gallery: `examples/ui_gallery/src/pages/{elements,forms,feedback,overlays,navigation,data,layout,examples}.rs`. Test harness: `examples/ui_gallery/tests/visual_harness.rs` (`#[gpui::test]` + `TestAppContext`, `#[path]`-includes `gallery_app.rs`/`pages/mod.rs` since `ui_gallery` is bin-only).
- Component pattern reference: `crates/ui/src/components/badge.rs` (enum variant/color builder + `RenderOnce`).

## Resolved Decisions (user sign-off 2026-07-04 — best-practice)
1. ✅ **AI-chat components OUT** (Attachment/Bubble/Message/Marker/Message Scroller). Niche AI-chat UI, not core shadcn/ui kit — YAGNI. Not built; revisit only if the app becomes a chat app.
2. ✅ **Chart = hand-rolled minimal primitives via GPUI `canvas()`** (bar/line/area/pie), NO external plotting crate (most Rust plotting crates render to image buffers, not GPUI elements; off-thread render is disproportionate complexity). Scope = "basic charts", not full Recharts parity. Stays P6/optional; document the scope boundary, don't fake advanced chart types.
3. ✅ **Button = additive alias; shadcn names are the recommended PUBLIC vocabulary.** New code uses `.variant(ButtonVariant::{Default,Destructive,Outline,Secondary,Ghost,Link})` + `.size({Sm,Default,Lg,Icon})`; old `.primary()/.danger()/.soft()` + `ButtonStyle::*` keep working (soft-deprecated in doc comments, NOT removed). Zero rename of the ~130 call sites — churn with no functional benefit.
4. ✅ **Keep hex→Hsla** (`palette.rs` unchanged); shadcn OKLCH is a value-source reference only. No OKLCH rework.
5. ✅ **Re-verify shadcn specifics against real source** (Select/Checkbox/Slider variant+size lists, `--radius` calc formula) — MANDATORY first step of Phase 1 (tokens/radius) and Phase 3 (form components), via the shadcn component registry / docs, before finalizing each API shape. Not a blocker, a required verify step (already folded into those phases).

## Open Questions
- None blocking. Phase-06 Chart scope (which basic chart types to ship first) is refined during that phase's spike, within the hand-rolled-`canvas()` decision above.
