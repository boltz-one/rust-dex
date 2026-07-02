# Goal: Tailwind UI Component Gallery + UI Kit Extension

## Mission
Restyle rust-dex's `crates/ui` (~44 GPUI components) to match Tailwind UI (Application UI) visuals with a generic, dark/light-aware design system, add the missing components, and ship a navigable `examples/ui_gallery` binary showcasing all of them — verified by `make check-all` green + `cargo run -p ui_gallery` opening a working window.

## Context & Key Files
- Full plan: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/plan.md` (read Cross-Cutting Requirements first)
- Phases (do in order): `phase-01-design-tokens.md` → `phase-02-core-components.md` → `phase-03-form-controls.md` → `phase-04-composite-overlay.md` → `phase-05-navigation-gallery.md` (all under PLAN_PATH)
- Research specs: `research/researcher-01-tailwind-spec.md` (tokens/values), `research/researcher-02-gpui-codebase.md` (GPUI capabilities)
- Bootstrap template: `crates/app/src/main.rs`; component pattern: `crates/ui/src/components/button/button.rs`; theme: `crates/theme/src/styles/colors.rs`

## Requirements
**Must do:**
- Generic naming ONLY — no `tw`/`tailwind`/brand-color identifiers in code. Palette is ROLE-based: `palette::neutral/primary/success/warning/danger/info(shade)` (hex from Tailwind internally). Neutrals via `semantic::*` reading `cx.theme()`; accents via `palette`.
- Support BOTH dark + light (neutrals theme-driven so dark works free); focus ring = true gapped wrapper (`focus_ring()`); vendor Heroicons SVGs into `crates/icons` + `IconName`.
- Reuse existing components (most already exist — restyle, don't rewrite). Genuinely new: TextInput, Textarea, Select, RadioButton, Card, Navbar, Sidebar. TextInput uses the real vendored `EntityInputHandler` IME plumbing (no mock input).
- Gallery: new `examples/ui_gallery` crate added to `Cargo.toml` members; sidebar navigation reusing each component's `preview()`; light/dark toggle in navbar.
- Each phase ends with visual-verify (Playwright tailwindcss.com reference vs macOS offscreen screenshot) in BOTH modes.

**Must not:**
- Touch `crates/app` behavior or change `default-members = ["crates/app"]`.
- Hardcode neutral grays in components (use `semantic::*`). No mocks/fakes. Files >200 lines.

## Success Criteria
- `make check-all` (i.e. `cargo check --workspace --all-targets`) exits 0.
- `make check` + `cargo fmt --all -- --check` still pass; `crates/app` unchanged.
- `cargo run -p ui_gallery` opens a window; sidebar nav switches component-group pages.
- New modules exist: `crates/ui/src/styles/{palette,semantic,shadow,focus_ring}.rs`.
- New components exist + exported via `crates/ui/src/prelude.rs`: TextInput, Textarea, Select, RadioButton, Card, Navbar, Sidebar.
- `grep -rE 'tailwind_|tw_|(slate|blue|red|green|amber)_[0-9]' crates/ui/src` finds no code identifiers (only doc-comment references allowed).
- Every showcased component renders legibly in both light and dark (documented screenshot comparison).

## Out of Scope
- Changing/shipping `crates/app`.
- Full text editor features (multi-cursor, wrapping) — single-line/basic multi-line only.
- Windows/Linux screenshot verify (macOS offscreen only; accepted).
- Marketing UI / non-Application-UI Tailwind pages.

## Verification
```bash
make check-all && make check && cargo fmt --all -- --check
cargo run -p ui_gallery   # opens gallery window; click sidebar items
grep -rnE 'tailwind_|tw_|(slate|zinc|blue|indigo|red|green|amber)_[0-9]' crates/ui/src && echo "FAIL: brand ids" || echo "OK: generic"
```
