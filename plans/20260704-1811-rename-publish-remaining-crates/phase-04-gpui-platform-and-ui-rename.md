# Phase 04: gpui_platform + ui Rename

## Context links
- Plan: [plan.md](./plan.md)
- Prior phases: [phase-01](./phase-01-font-kit-safety-fix.md), [phase-02](./phase-02-leaf-crates-rename.md), [phase-03](./phase-03-mid-tier-rename.md) (all must land first)
- Research: `research/researcher-01-dependency-graph.md` §1-3

## Overview
- Date: 2026-07-04
- Description: Rename `gpui_platform` (depends on `gpui_macos`/`gpui_linux`/`gpui_windows`, target-gated) and `ui` (depends on `component`, `icons`, `menu`, `theme`, `ui_macros`). These are the last two crates before `app` itself (phase-05) and have the widest fan-in of any crate in this batch.
- Priority: P1
- Status: not-started

## Key Insights
- `gpui_platform`'s dependencies on `gpui_macos`/`gpui_linux`/`gpui_windows` are all target-`cfg`-gated (`crates/gpui_platform/Cargo.toml:24-32`) — e.g. macOS build only compiles the `gpui_macos` edge. This means a full cross-platform `cargo check` isn't possible from a single OS; verification is inherently partial per-machine (documented as a known CI gap, not a plan defect — same caveat as phase-02/03's platform-gated crates).
- `ui` has the widest internal fan-in of the whole 14-crate batch: `component`, `icons`, `menu`, `theme`, `ui_macros` (`crates/ui/Cargo.toml:16,20,22,27-28`) plus already-renamed `gpui_macros`, `gpui_util` (lines 19, 29). All of these must already be renamed (phases 02-03 for the 4 new ones; `gpui_macros`/`gpui_util` were already done in the original 16-crate batch).
- `ui/Cargo.toml` has **no `description` field** (confirmed by direct read — every other field present is `edition.workspace`, `publish.workspace`, `[lib] name = "ui"` explicit at line 11). This is a pre-existing gap across 12 of the 14 crates in this whole plan (see plan.md resolved-decision log) — `description` is NOT fixed in this phase (soft warning only, content-authoring judgment call for 12 crates, deferred as its own hygiene pass). `license`, however, IS a hard blocker for a real crates.io publish (server-side rejection, not just a local warning) and `ui` is one of only 3 crates in this plan missing it — since we're already editing this exact file, fixing it now avoids a second future touch. Best-practice pick: `license = "GPL-3.0-or-later"`, matching every sibling UI-layer crate this one directly depends on (`component`, `icons`, `menu`, `theme`, `syntax_theme`, `ui_macros` — all already `GPL-3.0-or-later`); the platform/infra layer (`gpui_platform` family, `collections`, `util`) uses `Apache-2.0` instead, so this is a deliberate layer-consistent choice, not a guess.
- `ui/Cargo.toml`'s `[lib] name = "ui"` (line 11) is an EXPLICIT override, same category as `font_kit`'s `[lib] name = "font_kit"` — leave untouched per the established pattern; it only affects the compiled artifact/doc identity, not consumer `use` resolution.
- `app` (phase-05) will depend on both `gpui_platform` and `ui` — this phase must fully land before phase-05 starts.

## Requirements
| Crate dir | New `[package] name` | Root Cargo.toml line | Crate Cargo.toml `name=` line |
|---|---|---|---|
| `crates/gpui_platform` | `boltz-gpui-platform` | 53 | 2 |
| `crates/ui` | `boltz-ui` | 66 | 2 (keep `[lib] name = "ui"` at line 11 unchanged) |

3. Add `license = "GPL-3.0-or-later"` to `crates/ui/Cargo.toml` (currently missing, hard blocker for real publish — see Key Insights).

## Architecture
No architectural change. `gpui_platform` is a target-selection facade (per `docs/system-architecture.md`), `ui` is the component library — both keep their exact current module structure.

## ADR Rationale
Follows established pattern, no new decision.

## Related code files
- `crates/gpui_platform/Cargo.toml:2`
- `crates/ui/Cargo.toml:2` (and note `:11` `[lib] name = "ui"` — do not touch)
- `Cargo.toml:53` (gpui_platform), `:66` (ui)
- `scripts/publish-crates.sh` `PACKAGES` array (post phase-03 state)

## Implementation Steps
1. `crates/gpui_platform/Cargo.toml` line 2: `name = "gpui_platform"` → `name = "boltz-gpui-platform"`
2. `crates/ui/Cargo.toml`:
   ```diff
    [package]
   -name = "ui"
   +name = "boltz-ui"
    version = "0.1.0"
    edition.workspace = true
    publish.workspace = true
   +license = "GPL-3.0-or-later"
   ```
   (leave `[lib] name = "ui"` at line 11 as-is)
3. Root `Cargo.toml`:
   ```diff
   -gpui_platform = { path = "crates/gpui_platform", default-features = false }
   +gpui_platform = { path = "crates/gpui_platform", default-features = false, version = "0.1.0", package = "boltz-gpui-platform" }
   -ui = { path = "crates/ui" }
   +ui = { path = "crates/ui", version = "0.1.0", package = "boltz-ui" }
   ```
4. `scripts/publish-crates.sh`: append `boltz-gpui-platform`, then `boltz-ui` (in that order — `ui` doesn't depend on `gpui_platform` so order between these two doesn't strictly matter, but both must come after phase-01/02/03's entries).
5. Verify: `cargo check -p boltz-gpui-platform -p boltz-ui`.

## Todo list
- [ ] Rename `[package] name` in both crate manifests (leave `ui`'s `[lib] name = "ui"` at line 11 untouched)
- [ ] Add `license = "GPL-3.0-or-later"` to `crates/ui/Cargo.toml`
- [ ] Add `version` + `package` to both root `Cargo.toml` entries (preserve `default-features = false` on gpui_platform)
- [ ] Append `boltz-gpui-platform`, `boltz-ui` to `scripts/publish-crates.sh`
- [ ] `cargo check -p boltz-gpui-platform -p boltz-ui` passes
- [ ] `cargo check -p ui_gallery` (the `examples/ui_gallery` binary, `publish = false`) still compiles — it depends on `ui`, `gpui_platform`, `component`, `icons`, `theme` all via unaffected dependency keys; this is the best available smoke test that fan-in consumers are unaffected before touching `app` in phase-05

## Success Criteria
- `cargo metadata` lists both under new names with correct resolved dependency graph.
- `cargo check --workspace --all-targets` compiles, including `examples/ui_gallery`.

## Risk Assessment
- Medium: `ui`'s wide fan-in (5 renamed-in-this-batch deps + 2 already-renamed) makes it the highest-surface-area single edit in the plan for catching a missed upstream `package =` addition — if any of phase-02/03's crates were only half-updated (e.g. crate manifest renamed but root `Cargo.toml` entry missed), this is where `cargo check -p boltz-ui` will surface it with a `no matching package named boltz-X` error.
- `examples/ui_gallery` is workspace member but `publish = false` — confirm it's NOT accidentally added to `scripts/publish-crates.sh` (it never should be; not part of the 30 published crates).

## Security Considerations
N/A — metadata-only change.

## Next steps
Proceed to phase-05 (`app` → `boltz-app` + bootstrap ADR) only after this phase's crates and `ui_gallery` check clean.
