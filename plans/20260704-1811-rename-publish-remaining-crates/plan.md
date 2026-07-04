---
title: "Rename remaining 14 crates to boltz-* for crates.io publishing"
description: "Apply the established boltz-* rename pattern to the last 14 workspace crates, fix a live font_kit name-collision footgun, and upgrade app into a real bootstrap template."
status: implemented (dry-run verified; real publish pending — CI only)
priority: P1
effort: 7h
branch: main
lane: high-risk
lane_reason: "≥3 files touched per phase, changes public crates.io registry identity (irreversible once published), phase-05 requires an architectural judgment call (ADR) on app's bootstrap model"
tags: [cargo, publishing, crates-io, rename, workspace, gpui]
created: 2026-07-04
---

# Rename Remaining 14 Crates for crates.io Publishing

## Context

16 crates already renamed to `boltz-*` and publish via `scripts/publish-crates.sh` (see `docs/codebase-summary.md`, root `Cargo.toml` `[workspace.dependencies]`). 14 crates remain. This plan finishes the rename, fixes a live publish-blocking bug in `font_kit`, and resolves the ADR for `app` → `boltz-app` per user's ask that it become a faster bootstrap starting point.

Research: `research/researcher-01-dependency-graph.md` (topo order, line numbers — re-verified by this planner, unchanged), `research/researcher-02-app-bootstrap.md` (bootstrap options — planner overrides its minimal recommendation, see phase-05).

## Established rename pattern (all phases 1-4 follow this)

Per crate `X` at `crates/X`:
1. `crates/X/Cargo.toml`: `[package] name = "X"` → `name = "boltz-X"` (kebab-case). Leave `[lib]`, `[[bin]]`, and all Rust `use X::...` untouched — Cargo resolves `use` by the *consumer's* dependency-table key, not the producer's package/lib name, so nothing downstream breaks.
2. Root `Cargo.toml` `[workspace.dependencies]`: entry keeps key `X`, gains `package = "boltz-X"` and `version = "<X's current [package] version>"` (none of the 14 currently have a `version` key — confirmed by reading root `Cargo.toml` lines 44-72, 178).
3. `scripts/publish-crates.sh`: append `boltz-X` to `PACKAGES` array, topo order, after the existing 16.

## Phases

| # | Phase | Crates | Status | File |
|---|-------|--------|--------|------|
| 1 | font_kit safety fix (do first — live footgun) | font_kit | not-started | [phase-01-font-kit-safety-fix.md](./phase-01-font-kit-safety-fix.md) |
| 2 | Leaf crates rename (zero internal deps) | syntax_theme, icons, menu, ui_macros, gpui_wgpu, gpui_windows | not-started | [phase-02-leaf-crates-rename.md](./phase-02-leaf-crates-rename.md) |
| 3 | Mid-tier rename | theme, gpui_linux, gpui_macos, component | not-started | [phase-03-mid-tier-rename.md](./phase-03-mid-tier-rename.md) |
| 4 | gpui_platform + ui rename | gpui_platform, ui | not-started | [phase-04-gpui-platform-and-ui-rename.md](./phase-04-gpui-platform-and-ui-rename.md) |
| 5 | app → boltz-app + bootstrap upgrade (ADR) | app | not-started | [phase-05-app-bootstrap-upgrade.md](./phase-05-app-bootstrap-upgrade.md) |
| 6 | Verify + dry-run | all 30 crates | not-started | [phase-06-verify-and-dry-run.md](./phase-06-verify-and-dry-run.md) |

Topo order (deps-first): `font_kit → syntax_theme → icons → menu → ui_macros → gpui_wgpu → gpui_windows → theme → gpui_linux → gpui_macos → component → gpui_platform → ui → app`

## Key decision made in this plan

**app bootstrap (phase 5)**: committing to Option B (rename `app`→`boltz-app` AND add a standalone `template/` directory consumable via `cargo generate --git <repo> --subfolder template`) over researcher-02's minimal rename-only recommendation. Rationale in phase-05's ADR section. Flagged as separable/cuttable if the user wants to descope.

## Discovered during re-verification (not in original research)

`crates/font_kit/Cargo.toml` has `readme = "README.md"` pointing to a file deleted in commit `3146812` — this hard-fails local `cargo package`/`cargo publish --dry-run` (Cargo errors if a declared readme path doesn't exist on disk), independent of the name-collision issue. Also its `license` field was dropped in the same commit with no replacement. Both must be fixed in phase-01 or the phase-06 dry-run aborts immediately. Full detail in phase-01.

## Decisions (resolved per best practice, superseding the earlier open-question list)

1. **font_kit license — RESOLVED**: restore `license = "MIT OR Apache-2.0"` (original upstream servo/font-kit license, dropped by commit `3146812`). Not a style choice — vendored code's license doesn't change because a manifest field was deleted; restoring the original is the only legally accurate option. See phase-01 ADR.
2. **font_kit version — RESOLVED**: keep `0.14.1` as the first published version of `boltz-font-kit`, no bump. It's a new, distinct crates.io package identity (never published before under any name), so there's no semver-continuity expectation to break.
3. **`license` on `ui`/`app` (hard publish blocker) vs `description` on 12/14 crates (soft warning) — RESOLVED, split**:
   - `license`: fixed inline, now in scope. `ui` and `app` each get `license = "GPL-3.0-or-later"`, matching the UI-application layer convention already used by every sibling crate they sit beside/on top of (`component`, `icons`, `menu`, `theme`, `syntax_theme`, `ui_macros`). Added to phase-04 (`ui`) and phase-05 (`app`) — cheap, mechanical, and these files are already being edited in this plan, so fixing a real publish-blocker now avoids a second future touch of the same lines.
   - `description`: still deferred, correctly. Missing on 12 crates is a soft Cargo warning only (doesn't block dry-run or even real publish), and writing 12 accurate one-line descriptions is a content-authoring task, not a mechanical rename — stays out of scope as its own future hygiene pass.
4. **app bootstrap Option A vs B — RESOLVED**: commit to Option B (rename + standalone `cargo-generate` `template/` dir, phase-05 ADR decision 2). Option A (rename-only) doesn't deliver the "faster/simpler bootstrap" the user explicitly asked for — cloning the full 35-crate monorepo is what already exists today, not an improvement. Option B is cleanly separable (zero workspace/build coupling) so it can still be cut later without touching anything else in this plan if it proves not worth maintaining.
5. **`CARGO_REGISTRY_TOKEN` custody — RESOLVED as a process rule, not a named owner**: best practice is that no individual holds/uses this token locally at all. Real (non-`DRY_RUN`) publishes should run exclusively through `.github/workflows/publish.yml` with the token stored as a GitHub Actions repository secret — this gives an audit trail (who triggered the workflow run) and avoids distributing a registry-wide credential to multiple laptops. This plan only reaches `DRY_RUN=1` (phase-06); the first real publish of these 14 crates should be a deliberate, reviewed CI run, not a local `cargo publish`.
