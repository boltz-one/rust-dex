# Goal: Tailwind Plus Application UI — Full Catalog Parity for GPUI UI Kit

## Mission
Audit `crates/ui` against the full Tailwind Plus **Application UI** catalog, fix every styling deviation, and build all missing core components — 42 deliverables (19 restyle/fix + 23 net-new), each showcased in `examples/ui_gallery`. Done = `make check` + `cargo fmt --all --check` green and `cargo run -p ui_gallery` opens with every component rendering.

## Context & Key Files
- Full plan: `plans/20260703-0001-tailwind-app-ui-complete/plan.md`
- Phases: `phase-01-gap-analysis-icons.md` (BLOCKS 2-7) → `phase-02-elements.md`, `phase-03-form-controls.md`, `phase-04-data-display.md`, `phase-05-overlays.md`, `phase-06-navigation.md`, `phase-07-layout-shells.md` (parallel) → `phase-09-gallery-consolidation.md`
- Research: `research/researcher-01-tailwind-appui-catalog.md`, `research/researcher-02-codebase-audit.md`
- Code: `crates/ui/src/components/`, `crates/ui/src/styles/` (tokens done), `crates/icons/src/icons.rs`, `examples/ui_gallery/src/gallery_app.rs`

## Requirements
**Must do:**
- Phase 1 FIRST: lock IN/OUT matrix + vendor only genuinely-missing Heroicons (`User`/`Home`/`Calendar`/`Minus`… — most already vendored) + register `IconName`.
- Generic naming, NO brand ids (`tw`/`tailwind`/`slate`/`blue`). Neutrals → `semantic::*` (theme-driven, dark+light); accents/status → `palette::{primary,success,warning,danger,info}`. Never hardcode a neutral gray.
- Reuse existing base before net-new (each phase has a Reuse Map). Focus ring = gapped wrapper (`styles/focus_ring.rs`).
- Every touched component: `#[derive(IntoElement, Documented, RegisterComponent)]` + builder + `impl Component { fn preview() }` + gallery showcase (Phase 9).
- Phase 5: grep each overlay tree for hex/hsla/rgb literals, replace with `semantic`/`palette`.

**Must not:**
- Touch `crates/app` or `default-members = ["crates/app"]`.
- Execute Phase 8 (Calendar/Command-palette/Color-picker/Carousel/Kanban/Virtualized — backlog, YAGNI) without explicit user go.
- Build a new overlay primitive for Combobox/Multi-Select — compose `Select`+`DropdownMenu`+`TextInput`, case-insensitive substring filter only.

## Success Criteria
- `make check` exits 0 (and `make check-all` if present).
- `cargo fmt --all --check` exits 0.
- `cargo run -p ui_gallery` opens a window; all new/restyled components render without panic, in light AND dark.
- All 42 in-plan components implemented per phase Success Criteria; each has a `preview()` + gallery entry.
- No brand-name identifiers in code (`grep -rEi 'tailwind|\bslate\b|\btw_' crates/ui/src` returns nothing in identifiers).

## Out of Scope
- Phase 8 advanced/deferred components (backlog only).
- Marketing / Ecommerce Tailwind blocks.
- Visual screenshot verification (user-confirmed: build + manual click-through only).
- Any change to `crates/app` shippable hello-world.

## Verification
```bash
make check && cargo fmt --all --check
make check-all 2>/dev/null || true
cargo run -p ui_gallery   # window opens, click through every page, light+dark
```
