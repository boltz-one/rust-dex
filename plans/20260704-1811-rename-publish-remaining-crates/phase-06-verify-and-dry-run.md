# Phase 06: Verify + Dry-Run

## Context links
- Plan: [plan.md](./plan.md)
- Prior phases: [phase-01](./phase-01-font-kit-safety-fix.md) through [phase-05](./phase-05-app-bootstrap-upgrade.md) (all must land first)

## Overview
- Date: 2026-07-04
- Description: Full-workspace compile check, `cargo metadata` sanity check of all 30 `package =` aliases (16 pre-existing + 14 new), then a full `DRY_RUN=1` run of `scripts/publish-crates.sh` covering the complete `PACKAGES` array. Explicitly does NOT perform a real publish (requires `CARGO_REGISTRY_TOKEN`, irreversible) — that remains a manual follow-up for the user.
- Priority: P1 (final gate before this plan is considered complete)
- Status: not-started

## Key Insights
- `scripts/publish-crates.sh`'s dry-run mode (`DRY_RUN=1`) runs `cargo publish -p <name> --dry-run` per package, tolerating one specific expected failure pattern: `no matching package named \`boltz-...\`` (script line 124) — this happens when a crate's workspace dependencies aren't YET on crates.io (expected for every crate in this plan, since none of the 30 boltz-* crates are published yet as of this plan... actually 16 already ARE published per plan.md context, so only the 14 new ones plus anything depending on them mid-graph will hit this fallback path). The script downgrades that specific failure to a warning; any OTHER failure calls `die` and aborts the whole run.
- This means phase-01's readme bug (if not fixed) would `die` immediately on `boltz-font-kit` with an unrelated-looking file-not-found error — this phase's dry-run is effectively the regression test proving phase-01 was done correctly.
- `pkg_version()` (script line 59-62) shells out to `cargo metadata` + a Python one-liner matching on `p['name']` — this ONLY works correctly if every renamed crate's `[package] name` truly matches what's in `PACKAGES`; a typo in either location (e.g. `boltz-syntax_theme` instead of `boltz-syntax-theme`) causes `pkg_version` to silently return empty via the `next()` generator raising `StopIteration` → Python traceback → non-empty `$version` check fails → `die "cannot resolve version for $name"`. This is actually a strong catch-typos safety net already built into the existing script; lean on it rather than manually eyeballing 30 names.
- `already_published()` (script line 65-86) does a live HTTP call to `index.crates.io` for every package on every dry run — expect ~30 network round-trips, a few seconds each; this is normal, not a bug.

## Requirements
1. `cargo check --workspace --all-targets` succeeds (mirrors `Makefile`'s `check-all` target, `Makefile:15-16`, run with `RUN_FEATURES=gpui_platform/runtime_shaders`).
2. `cargo metadata --format-version 1 --no-deps` — programmatically verify all 30 expected `boltz-*` package names are present and each entry's `manifest_path` points at the expected `crates/X/Cargo.toml`.
3. `DRY_RUN=1 ./scripts/publish-crates.sh` completes with exit code 0 across the full 30-entry `PACKAGES` array.
4. Confirm `template/` (phase-05) is absent from the `cargo metadata` output above (proves it's correctly decoupled from the workspace, not an accidental 31st entry).

## Architecture
No new architecture — this is a pure verification phase. No source files are edited here (other than resolving anything phases 01-05 might have left broken, which should not happen if each phase's own Todo list was completed).

## ADR Rationale
No new decision — this phase runs the acceptance checks implied by the plan's design (all_prior phases + plan.md pattern). Short note only, per high-risk-lane requirement for non-decision phases.

## Related code files
- `scripts/publish-crates.sh` (run, not edited)
- `Makefile:15-16` (`check-all` target, run, not edited)
- Root `Cargo.toml` (read via `cargo metadata`, not edited)

## Implementation Steps
1. Run the full workspace check:
   ```sh
   RUSTUP_TOOLCHAIN=stable cargo check --workspace --all-targets --features gpui_platform/runtime_shaders
   ```
   Fix any compile error before proceeding — per the established pattern, none are expected (renames are metadata-only), so any failure here means a phase 01-05 step was missed or mistyped.

2. Sanity-check the full package list:
   ```sh
   cargo metadata --format-version 1 --no-deps | python3 -c "
   import sys, json
   m = json.load(sys.stdin)
   names = sorted(p['name'] for p in m['packages'] if p['name'].startswith('boltz-'))
   print(f'{len(names)} boltz-* packages:')
   for n in names: print(' ', n)
   "
   ```
   Expect exactly 30 (16 pre-existing + 14 from this plan). Cross-check the 14 new ones against plan.md's list verbatim: `boltz-font-kit, boltz-syntax-theme, boltz-icons, boltz-menu, boltz-ui-macros, boltz-gpui-wgpu, boltz-gpui-windows, boltz-theme, boltz-gpui-linux, boltz-gpui-macos, boltz-component, boltz-gpui-platform, boltz-ui, boltz-app`.

3. Confirm `template/` isolation:
   ```sh
   cargo metadata --format-version 1 --no-deps | python3 -c "
   import sys, json
   m = json.load(sys.stdin)
   assert not any('template' in p['manifest_path'] for p in m['packages']), 'template/ leaked into workspace metadata'
   print('OK: template/ not part of workspace')
   "
   ```

4. Run the full dry-run publish pipeline:
   ```sh
   DRY_RUN=1 ./scripts/publish-crates.sh
   ```
   Expect: `mode: DRY-RUN | no-verify=1 | 30 crates` header, then one `dry-run: validate package <name>@<version>` or `manifest OK; full check deferred` line per package, ending with `done`. Any `die` output indicates a real defect (missing `package =`, typo, or a re-emergence of phase-01's readme-class issue) — do not consider this plan complete until this run exits 0.

5. If time permits (optional, not blocking): from a scratch directory outside the repo, run `cargo generate --path <repo>/template --name smoke-test` (phase-05's own dry run) as a final cross-phase sanity check that phase-05's template and phase-01-04's renames are mutually consistent (the template's `Cargo.toml` version pins should match what `cargo metadata` reports in step 2).

## Todo list
- [ ] `cargo check --workspace --all-targets --features gpui_platform/runtime_shaders` passes
- [ ] `cargo metadata` reports exactly 30 `boltz-*` packages, matching plan.md's list
- [ ] `cargo metadata` shows zero packages with `template` in their `manifest_path`
- [ ] `DRY_RUN=1 ./scripts/publish-crates.sh` exits 0 across all 30 packages
- [ ] (Optional) `cargo generate --path ./template --name smoke-test` produces a consistent, buildable-manifest scratch project

## Success Criteria
- All 4 Requirements above pass with no manual workarounds or `--allow-dirty`/skip flags beyond what the script already documents.
- This is the last phase of the plan — once green, the 14-crate rename is functionally complete and the repo is ready for a REAL publish, pending the process step in plan.md's Decisions section (item 5: real publish runs via CI, not locally).

## Risk Assessment
- Low risk for the check/metadata steps (read-only, no mutation).
- The dry-run step makes ~30 live HTTPS calls to `index.crates.io` — flaky-network risk only; script already retries nothing per-call but each call has its own 20s timeout (`urllib.request.urlopen(..., timeout=20)`), so a transient network blip surfaces as a script failure, not a false "already published" result (the `except Exception: sys.exit(2)` path is NOT treated as "already published," only exit(0) is).
- **Explicitly out of scope**: a REAL (non-dry-run) publish. That requires `CARGO_REGISTRY_TOKEN` and is irreversible (crates.io names/versions cannot be un-published beyond a 72h yank window). This is a manual follow-up step to run via `.github/workflows/publish.yml` — see plan.md's Decisions section (item 5).

## Security Considerations
- Read-only network calls to the public crates.io sparse index — no credentials involved in dry-run mode (the script's own comment confirms `CARGO_REGISTRY_TOKEN` is only required for `DRY_RUN=0`).
- Once a real publish happens, all 14 new crate names become permanently public and squattable-if-abandoned — no action needed now, just noting for the eventual real-publish follow-up.

## Next steps
None within this plan — this is the terminal phase. Follow-ups (real publish, license hygiene pass on 12 crates lacking `description`, template end-to-end build test once crates are live) are listed in plan.md's Open Questions section for the user to schedule separately.
