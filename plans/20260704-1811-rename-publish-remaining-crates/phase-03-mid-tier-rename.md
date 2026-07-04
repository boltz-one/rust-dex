# Phase 03: Mid-Tier Crates Rename

## Context links
- Plan: [plan.md](./plan.md)
- Prior phases: [phase-01](./phase-01-font-kit-safety-fix.md), [phase-02](./phase-02-leaf-crates-rename.md) (must land first — this phase's crates depend on `boltz-syntax-theme`, `boltz-gpui-wgpu`, `boltz-font-kit`)
- Research: `research/researcher-01-dependency-graph.md` §1-3

## Overview
- Date: 2026-07-04
- Description: Rename `theme` (→ `syntax_theme`), `gpui_linux` (→ `gpui_wgpu`), `gpui_macos` (→ `font_kit`), `component` (→ `theme`). Internal-order matters WITHIN this phase: `theme` and `gpui_linux`/`gpui_macos` have no dependency on each other and can go in any order, but `component` depends on `theme`, so rename `theme` before `component` if doing them sequentially in one sitting (both edits land in the same commit either way, but sequencing avoids a transient broken intermediate state if verifying incrementally).
- Priority: P1
- Status: not-started

## Key Insights
- `theme` depends on `syntax_theme` (`crates/theme/Cargo.toml:23`, `syntax_theme.workspace = true`) — requires phase-02 landed.
- `gpui_linux` depends on `gpui_wgpu` (`crates/gpui_linux/Cargo.toml:54`, optional, wayland/x11-feature-gated) — requires phase-02 landed.
- `gpui_macos` depends on `font_kit` (`crates/gpui_macos/Cargo.toml:32`, macOS-target-gated) — requires phase-01 landed.
- `component` depends on `theme` (`crates/component/Cargo.toml:20`, `theme.workspace = true`) — requires this phase's own `theme` rename to land first (or land together, since both are in this same phase file).
- `theme`'s `[features] test-support = ["gpui/test-support", "syntax_theme/test-support"]` (line 13) references `syntax_theme`'s OWN feature flag by its dependency-key name `syntax_theme` — this is unaffected by `syntax_theme`'s package rename (feature-flag cross-references use the dependency table key, same mechanism as `use` statements).
- `gpui_linux`'s `[features] wayland = [..., "gpui_wgpu", ...]` (line 19) similarly references `gpui_wgpu` by dependency key — unaffected.
- All 4 crates already have `version = "0.1.0"` and correct `license` fields (`GPL-3.0-or-later` for theme/component, `Apache-2.0` for gpui_linux/gpui_macos) — no metadata gaps beyond what phase-01 already fixed for font_kit's side of the `gpui_macos` dependency.
- `gpui_macos` has platform-gated `[target.'cfg(target_os = "macos")'.build-dependencies]` including `gpui.workspace = true` (line 60) — unrelated to this rename, no action needed.

## Requirements
| Crate dir | New `[package] name` | Root Cargo.toml line | Crate Cargo.toml `name=` line |
|---|---|---|---|
| `crates/theme` | `boltz-theme` | 65 | 2 |
| `crates/gpui_linux` | `boltz-gpui-linux` | 50 | 2 |
| `crates/gpui_macos` | `boltz-gpui-macos` | 51 | 2 |
| `crates/component` | `boltz-component` | 45 | 2 |

## Architecture
No architectural change. Dependency edges (`theme→syntax_theme`, `gpui_linux→gpui_wgpu`, `gpui_macos→font_kit`, `component→theme`) are all expressed via `X.workspace = true` using dependency-table keys that never change — only the resolved package identity changes.

## ADR Rationale
Follows established pattern, no new decision. The only sequencing nuance (rename `theme` before `component` within this phase) is a verification-safety recommendation, not an architectural choice.

## Related code files
- `crates/theme/Cargo.toml:2`
- `crates/gpui_linux/Cargo.toml:2`
- `crates/gpui_macos/Cargo.toml:2`
- `crates/component/Cargo.toml:2`
- `Cargo.toml:65` (theme), `:50` (gpui_linux), `:51` (gpui_macos), `:45` (component)
- `scripts/publish-crates.sh` `PACKAGES` array (post phase-02 state)

## Implementation Steps
1. `crates/theme/Cargo.toml` line 2: `name = "theme"` → `name = "boltz-theme"`
2. `crates/gpui_linux/Cargo.toml` line 2: `name = "gpui_linux"` → `name = "boltz-gpui-linux"`
3. `crates/gpui_macos/Cargo.toml` line 2: `name = "gpui_macos"` → `name = "boltz-gpui-macos"`
4. `crates/component/Cargo.toml` line 2: `name = "component"` → `name = "boltz-component"`
5. Root `Cargo.toml`:
   ```diff
   -theme = { path = "crates/theme" }
   +theme = { path = "crates/theme", version = "0.1.0", package = "boltz-theme" }
   -gpui_linux = { path = "crates/gpui_linux", default-features = false }
   +gpui_linux = { path = "crates/gpui_linux", default-features = false, version = "0.1.0", package = "boltz-gpui-linux" }
   -gpui_macos = { path = "crates/gpui_macos", default-features = false }
   +gpui_macos = { path = "crates/gpui_macos", default-features = false, version = "0.1.0", package = "boltz-gpui-macos" }
   -component = { path = "crates/component" }
   +component = { path = "crates/component", version = "0.1.0", package = "boltz-component" }
   ```
6. `scripts/publish-crates.sh`: append `boltz-theme`, `boltz-gpui-linux`, `boltz-gpui-macos`, `boltz-component` after phase-02's entries (in that relative order — `theme` first since `component` depends on it, though the script doesn't enforce ordering beyond what's needed for crates.io to resolve already-published deps at publish time).
7. Verify: `cargo check -p boltz-theme -p boltz-component` (cross-platform); `cargo check -p boltz-gpui-macos` (macOS only) or `cargo check -p boltz-gpui-linux` (Linux only) depending on dev machine OS.

## Todo list
- [ ] Rename `[package] name` in all 4 crate manifests
- [ ] Add `version` + `package` to all 4 root `Cargo.toml` entries (preserve existing `default-features = false` on gpui_linux/gpui_macos)
- [ ] Append all 4 new names to `scripts/publish-crates.sh` in dependency-safe order
- [ ] `cargo check -p boltz-theme -p boltz-component` passes
- [ ] `cargo check -p boltz-gpui-macos` (or `boltz-gpui-linux`, per available OS) passes
- [ ] Grep confirms zero `use theme::`/`use gpui_linux::`/`use gpui_macos::`/`use component::` call sites changed

## Success Criteria
- `cargo metadata` lists all 4 under new names, dependency graph intact (`boltz-theme` resolves `boltz-syntax-theme`; `boltz-gpui-macos` resolves `boltz-font-kit`; etc.)
- `cargo check --workspace` compiles.

## Risk Assessment
- Low-medium: this phase touches the first crates with *internal* dependency edges to other renamed crates in this batch, so it's the first point where a missed phase-01/02 step (forgotten `package =` addition) would surface as a resolution error (`no matching package named ...`) rather than a silent no-op — treat any such error as a signal to re-check the upstream phase, not a bug in this phase's own edits.

## Security Considerations
N/A — metadata-only change.

## Next steps
Proceed to phase-04 (`gpui_platform`, `ui`) only after this phase's crates check clean.
