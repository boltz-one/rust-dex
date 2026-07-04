# Goal: Rename remaining 14 crates to boltz-* for crates.io publishing

## Mission
Rename 14 remaining workspace crates to `boltz-*` crates.io names (continuing the pattern already used for 16 crates), fix a live `font_kit` publish-blocking bug, and upgrade `app` into `boltz-app` with a standalone `cargo-generate` bootstrap template.

## Context & Key Files
- Full plan: `plans/20260704-1811-rename-publish-remaining-crates/plan.md`
- Phases, execute in order: `phase-01-font-kit-safety-fix.md` → `phase-02-leaf-crates-rename.md` → `phase-03-mid-tier-rename.md` → `phase-04-gpui-platform-and-ui-rename.md` → `phase-05-app-bootstrap-upgrade.md` → `phase-06-verify-and-dry-run.md` (same dir)
- Root `Cargo.toml` (`[workspace.dependencies]`), `scripts/publish-crates.sh` (`PACKAGES` array), `crates/<name>/Cargo.toml` per crate

## Requirements
**Must do:**
- Topo order: font_kit → syntax_theme → icons → menu → ui_macros → gpui_wgpu → gpui_windows → theme → gpui_linux → gpui_macos → component → gpui_platform → ui → app
- Per crate X: `[package] name` → `boltz-X` in `crates/X/Cargo.toml`; root `Cargo.toml` workspace-dependency entry for X gains `package = "boltz-X"` + `version`; append `boltz-X` to `PACKAGES` in `scripts/publish-crates.sh`
- Phase-01 first: fix `font_kit` name collision (`boltz-font-kit`), remove dangling `readme =` line, restore `license = "MIT OR Apache-2.0"`
- `ui` and `app` also get `license = "GPL-3.0-or-later"` (currently missing, blocks real publish)
- `app`: rename `[package] name`, `[[bin]] name`, `default-run` all to `boltz-app`; update `Makefile:3` `PACKAGE :=`; update `[profile.release.package]` key in root `Cargo.toml`; fix stale `publish = false` claims in `docs/project-overview-pdr.md` and `docs/code-standards.md`
- Add standalone `template/` dir (own `[workspace]` root, `cargo-generate.toml`, `Cargo.toml` depending on `boltz-*` via crates.io versions, `src/main.rs` copied from `crates/app/src/main.rs` with `APP_ID`/window title templated) — full rationale in phase-05 ADR

**Must not:**
- Change any `[lib]` name/path or Rust `use X::...` call sites — only `[package] name` changes, consumers keep their dependency-table key
- Run a real (non-dry-run) publish or use `CARGO_REGISTRY_TOKEN` locally — real publish is CI-only via `.github/workflows/publish.yml`, out of scope here
- Add `template/` as a workspace member or to `scripts/publish-crates.sh`

## Success Criteria
- `cargo check --workspace --all-targets --features gpui_platform/runtime_shaders` exits 0
- `cargo metadata --format-version 1 --no-deps` lists exactly 30 `boltz-*` packages (16 existing + 14 new), zero with `template` in `manifest_path`
- `DRY_RUN=1 ./scripts/publish-crates.sh` exits 0 across all 30 packages

## Out of Scope
- Real crates.io publish (needs `CARGO_REGISTRY_TOKEN`; CI-only follow-up)
- Adding `description` field to the 12 crates missing it (soft warning, deferred hygiene pass)
- Full build test of `template/`'s generated project (can't succeed until real publish lands)

## Verification
```bash
cargo check --workspace --all-targets --features gpui_platform/runtime_shaders
cargo metadata --format-version 1 --no-deps | python3 -c "
import sys, json
m = json.load(sys.stdin)
n = [p['name'] for p in m['packages'] if p['name'].startswith('boltz-')]
assert len(n) == 30, f'expected 30, got {len(n)}: {n}'
assert not any('template' in p['manifest_path'] for p in m['packages'])
print('OK', len(n))
"
DRY_RUN=1 ./scripts/publish-crates.sh
```
