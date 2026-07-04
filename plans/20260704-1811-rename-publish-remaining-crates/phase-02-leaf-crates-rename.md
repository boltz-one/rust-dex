# Phase 02: Leaf Crates Rename (Zero Internal Dependencies)

## Context links
- Plan: [plan.md](./plan.md)
- Prior phase: [phase-01-font-kit-safety-fix.md](./phase-01-font-kit-safety-fix.md) (must land first)
- Research: `research/researcher-01-dependency-graph.md` §1-3

## Overview
- Date: 2026-07-04
- Description: Rename the 6 crates with zero internal workspace dependencies: `syntax_theme`, `icons`, `menu`, `ui_macros`, `gpui_wgpu`, `gpui_windows`. Pure mechanical application of the established pattern — no design decisions.
- Priority: P1
- Status: not-started

## Key Insights
- All 6 confirmed zero-internal-deps by researcher-01 and by this planner's direct reads of each `crates/X/Cargo.toml` (2026-07-04) — their `[dependencies]` sections reference only external crates (`gpui`, `rust-embed`, `serde`, `strum`, `quote`, `syn`, `wgpu`, platform-specific windows/wasm deps). Safe to rename in any order relative to each other, and safe to publish first (after `boltz-font-kit`).
- All 6 already have `version = "0.1.0"`, `license` (GPL-3.0-or-later or Apache-2.0 per crate), and `edition.workspace = true` correctly set — no other metadata gaps to fix (unlike font_kit in phase-01).
- `ui_macros` has a `[dev-dependencies]` on `component` and `ui` (line 20-21) — this is fine, dev-dependencies don't affect the topo publish order used by `scripts/publish-crates.sh` (they're not needed to build the published artifact, only to run its own tests) and don't create a real cyclic dependency (`component`/`ui` depend on `theme`/`ui_macros` themselves, but not on `ui_macros`'s *published artifact* — cargo doesn't publish dev-dependencies as build requirements).
- `ui_macros` is a proc-macro crate (`proc-macro = true`, line 13) — no impact on the rename pattern.
- `gpui_wgpu`'s `[dependencies]` includes `gpui_util.workspace = true` (line 31) — `gpui_util` is ALREADY renamed (`boltz-gpui-util`, one of the original 16), no action needed here.
- `gpui_windows`' `[features] default = ["gpui/default"]` (line 15) references the `gpui` crate's own feature, not `gpui_windows`'s package name — unaffected by rename.

## Requirements
Apply the 3-step pattern from plan.md to each of the 6 crates. New names:
| Crate dir | New `[package] name` | Root Cargo.toml line | Crate Cargo.toml `name=` line |
|---|---|---|---|
| `crates/syntax_theme` | `boltz-syntax-theme` | 64 | 2 |
| `crates/icons` | `boltz-icons` | 58 | 2 |
| `crates/menu` | `boltz-menu` | 60 | 2 |
| `crates/ui_macros` | `boltz-ui-macros` | 67 | 2 |
| `crates/gpui_wgpu` | `boltz-gpui-wgpu` | 54 | 2 |
| `crates/gpui_windows` | `boltz-gpui-windows` | 55 | 2 |

## Architecture
No architectural change. Each crate's `[lib] path` stays as-is (`src/syntax_theme.rs`, `src/icons.rs`, `src/menu.rs`, `src/ui_macros.rs`, `src/gpui_wgpu.rs`, `src/gpui_windows.rs`); none of the 6 declares an explicit `[lib] name`, so nothing to preserve there beyond leaving the field absent (Cargo's default lib crate name is irrelevant to consumers, who resolve via their own dependency-table key — see plan.md's pattern note).

## ADR Rationale
Follows the established pattern with no new decision — same reasoning as the 16 already-renamed crates and phase-01. No ADR needed beyond plan.md's pattern rationale.

## Related code files
- `crates/syntax_theme/Cargo.toml:2`
- `crates/icons/Cargo.toml:2`
- `crates/menu/Cargo.toml:2`
- `crates/ui_macros/Cargo.toml:2`
- `crates/gpui_wgpu/Cargo.toml:2`
- `crates/gpui_windows/Cargo.toml:2`
- `Cargo.toml:64` (syntax_theme), `:58` (icons), `:60` (menu), `:67` (ui_macros), `:54` (gpui_wgpu), `:55` (gpui_windows)
- `scripts/publish-crates.sh:35-53` (`PACKAGES` array, after phase-01's insert)

## Implementation Steps
1. `crates/syntax_theme/Cargo.toml` line 2: `name = "syntax_theme"` → `name = "boltz-syntax-theme"`
2. `crates/icons/Cargo.toml` line 2: `name = "icons"` → `name = "boltz-icons"`
3. `crates/menu/Cargo.toml` line 2: `name = "menu"` → `name = "boltz-menu"`
4. `crates/ui_macros/Cargo.toml` line 2: `name = "ui_macros"` → `name = "boltz-ui-macros"`
5. `crates/gpui_wgpu/Cargo.toml` line 2: `name = "gpui_wgpu"` → `name = "boltz-gpui-wgpu"`
6. `crates/gpui_windows/Cargo.toml` line 2: `name = "gpui_windows"` → `name = "boltz-gpui-windows"`
7. Root `Cargo.toml`:
   ```diff
   -syntax_theme = { path = "crates/syntax_theme" }
   +syntax_theme = { path = "crates/syntax_theme", version = "0.1.0", package = "boltz-syntax-theme" }
   -icons = { path = "crates/icons" }
   +icons = { path = "crates/icons", version = "0.1.0", package = "boltz-icons" }
   -menu = { path = "crates/menu" }
   +menu = { path = "crates/menu", version = "0.1.0", package = "boltz-menu" }
   -ui_macros = { path = "crates/ui_macros" }
   +ui_macros = { path = "crates/ui_macros", version = "0.1.0", package = "boltz-ui-macros" }
   -gpui_wgpu = { path = "crates/gpui_wgpu" }
   +gpui_wgpu = { path = "crates/gpui_wgpu", version = "0.1.0", package = "boltz-gpui-wgpu" }
   -gpui_windows = { path = "crates/gpui_windows", default-features = false }
   +gpui_windows = { path = "crates/gpui_windows", default-features = false, version = "0.1.0", package = "boltz-gpui-windows" }
   ```
   (Note `gpui_windows` and later `gpui_linux`/`gpui_macos`/`gpui_platform` already carry `default-features = false` — preserve that key, just add `version`/`package` alongside it.)
8. `scripts/publish-crates.sh`: append all 6 to `PACKAGES` (order among themselves doesn't matter, they're mutually independent; must appear after `boltz-font-kit` from phase-01 for consistency with the plan's topo listing, though not strictly required since none depend on font_kit):
   ```diff
    PACKAGES=(
      boltz-font-kit
      boltz-collections
      ...
      boltz-gpui
   +  boltz-syntax-theme
   +  boltz-icons
   +  boltz-menu
   +  boltz-ui-macros
   +  boltz-gpui-wgpu
   +  boltz-gpui-windows
    )
   ```
9. Verify: `cargo check -p boltz-syntax-theme -p boltz-icons -p boltz-menu -p boltz-ui-macros -p boltz-gpui-wgpu -p boltz-gpui-windows` (Linux/Windows-gated crates may need target-specific runs or `--target` flags if cross-checking from macOS; at minimum run the check on the crates buildable on the current dev machine's OS).

## Todo list
- [ ] Rename `[package] name` in all 6 crate manifests
- [ ] Add `version` + `package` to all 6 root `Cargo.toml` entries
- [ ] Append all 6 new names to `scripts/publish-crates.sh`
- [ ] `cargo check -p <each new name>` passes for all buildable-on-current-OS crates
- [ ] No `use syntax_theme::`, `use icons::`, `use menu::`, `use ui_macros::`, `use gpui_wgpu::`, `use gpui_windows::` call sites changed (grep to confirm zero diffs outside the two files touched per crate)

## Success Criteria
- `cargo metadata --format-version 1 --no-deps` lists all 6 packages under their new `boltz-*` names.
- `cargo check --workspace` compiles with zero source-code changes outside `Cargo.toml` files and `scripts/publish-crates.sh`.

## Risk Assessment
- Low risk — no dependency edges into or out of these 6 crates within this phase's own set; each can be verified in isolation.
- `gpui_windows` and `gpui_wgpu` are platform/feature-gated (Windows target, wasm target) — full `cargo check` for those paths may require a non-macOS toolchain or `--target` cross-check; note this as a CI-environment gap, not a plan defect.

## Security Considerations
N/A — metadata-only change.

## Next steps
Proceed to phase-03 (mid-tier: `theme`, `gpui_linux`, `gpui_macos`, `component`) only after this phase's crates check clean.
