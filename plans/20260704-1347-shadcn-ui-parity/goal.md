# Goal: shadcn/ui Parity Port for GPUI UI Kit

## Mission
Align the existing `crates/ui` GPUI kit to shadcn/ui's full catalog (~53 in-scope components) — adopt shadcn variant/size/token API + styling where components overlap (additively, zero call-site churn), add what's missing, into the ONE existing kit. Done = `make check` + `cargo fmt --all --check` green, `cargo test -p ui` + `cargo test -p ui_gallery` pass, every in-scope component showcased in the gallery.

## Context & Key Files
- Full plan: `plans/20260704-1347-shadcn-ui-parity/plan.md`
- Phases (P1 BLOCKS 2-6): `phase-01-token-foundation.md` → `phase-02-core-elements.md`, `phase-03-forms.md`, `phase-04-overlays.md`, `phase-05-data-nav.md`, `phase-06-advanced-heavy.md` → `phase-07-gallery-verify.md`
- Research: `research/researcher-01-shadcn-catalog.md`, `research/researcher-02-token-gap.md`
- Code: `crates/ui/src/styles/{palette,semantic}.rs`, `crates/ui/src/components/` (pattern ref `badge.rs`), `examples/ui_gallery/`, harness `examples/ui_gallery/tests/visual_harness.rs`

## Requirements
**Must do (P1 first — it BLOCKS all):**
- P1: add missing shadcn semantic roles (secondary, muted-bg, accent-standalone, ring, card/popover aliases, destructive=palette::danger) + a thin `radius.rs` mapping module + additive `ButtonVariant`/`.variant()`/`.size(Sm/Default/Lg/Icon)` API. Re-verify Select/Checkbox/Slider variants + `--radius` formula against real shadcn source first.
- Button: additive alias only — new shadcn-named API is the recommended vocabulary; keep `.primary()/.danger()/.soft()` + `ButtonStyle::*` working (soft-deprecated in docs). NO renaming the ~130 call sites.
- Colors only from `palette::*`/`semantic::*` (no hardcoded hex/hsla); shadcn variant/token NAMES are fine (generic design API), OKLCH is value-source reference only (keep hex→Hsla).
- Every component (new or aligned) gets a gallery page/section; interactive ones get a `#[gpui::test]`+`TestAppContext` case. Overlays reuse the existing deferred+anchored+occlude pattern. Reuse composites (Modal→Dialog/Sheet, Popover→HoverCard, DropdownMenu→Menubar, Combobox→Command, notification→Sonner, data_table→Table).
- P6 Chart: hand-roll basic types (Bar/Line/Area/Pie) via GPUI `canvas()`, NO external crate; defer+document advanced types.

**Must not:**
- Rename Button variants / rewrite existing call sites; break any `crates/ui` public API (additive/alias only, else update all callers+gallery+harness same phase).
- Touch `crates/app` or root `default-members`.
- Build AI-chat components (Attachment/Bubble/Message/Marker — OUT). Add a plotting crate for Chart. Fake heavy components — document real limits.

## Success Criteria
- `make check` exits 0; `cargo fmt --all --check` exits 0.
- `cargo test -p ui` passes (existing regressions incl. data_table green — proves non-breaking).
- `cargo test -p ui_gallery` passes (harness `#[gpui::test]` cases for new interactive components).
- `cargo build -p ui_gallery` links; every in-scope component reachable in a gallery page.
- New semantic roles + `radius.rs` + `ButtonVariant` API exist; old Button builders still compile.
- Heavy/deferred items (Chart advanced types, etc.) documented in-code, not silently dropped.

## Out of Scope
- AI-chat components; native `<select>`; standalone Toast (superseded by Sonner).
- OKLCH rework; a new component crate (one kit only).
- `crates/app` changes.

## Verification
```bash
make check && cargo fmt --all --check
cargo test -p ui
cargo test -p ui_gallery
cargo build -p ui_gallery   # optional: cargo run -p ui_gallery for manual visual check
```
