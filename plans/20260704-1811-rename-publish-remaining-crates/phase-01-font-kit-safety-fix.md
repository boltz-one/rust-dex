# Phase 01: font_kit Safety Fix

## Context links
- Plan: [plan.md](./plan.md)
- Research: `research/researcher-01-dependency-graph.md` (§4 crates.io availability), `research/researcher-02-app-bootstrap.md` (unrelated, for cross-reference only)
- Files: `crates/font_kit/Cargo.toml`, root `Cargo.toml` (lines 44-72, 178), `scripts/publish-crates.sh`

## Overview
- Date: 2026-07-04
- Description: Fix two independent, live publish-blocking issues in `font_kit` before touching any other crate: (a) its `[package] name = "font_kit"` collides with the pre-existing `font-kit`/`font_kit` crate on crates.io (the original servo/pcwalton crate this was vendored from — crates.io treats `-`/`_` as equivalent for uniqueness), and (b) a dangling `readme = "README.md"` reference plus a dropped `license` field, both discovered during this planner's re-verification, which will hard-fail `cargo package`/`cargo publish --dry-run` regardless of the name fix.
- Priority: P0 (blocks everything downstream — must land first)
- Status: not-started

## Key Insights
- `font_kit` is the ONLY crate among the 30 in this workspace with a `readme =` field pointing at a file that doesn't exist on disk (verified: `ls crates/font_kit/` shows only `build.rs`, `Cargo.toml`, `src/` — no `README.md`).
- Git archaeology (`git log --oneline -- crates/font_kit/Cargo.toml`): commit `3146812` ("chore(crates): drop per-crate license fields and vendored LICENSE/README files") deleted `crates/font_kit/README.md`, `LICENSE-APACHE`, `LICENSE-MIT`, and the `license = "MIT OR Apache-2.0"` line, but left the `readme = "README.md"` pointer in place — an oversight, not a deliberate decision. Commit `c8eb940` ("fix(font_kit): normalize crate for publishing") then flipped `publish = false` → `publish = true`, activating the footgun.
- Cargo hard-errors locally when a `readme` path doesn't exist (this happens during `cargo package`, which `cargo publish --dry-run` invokes) — this is NOT a soft warning, unlike missing `description`/`license` (which only warn, and only when ALL of description/license/homepage/documentation/repository are absent — here `repository.workspace = true` is inherited by every crate in this workspace, so that warning path never triggers). This means phase-06's `DRY_RUN=1 ./scripts/publish-crates.sh` will abort on `font_kit` specifically because of the readme, not (only) because of the name collision.
- `license` missing is a softer local issue but a hard blocker for the REAL (non-dry-run) crates.io publish — crates.io's registry-side validation requires `license` or `license-file`. Must be fixed now while we're already in this file, even though it won't surface until someone runs a real publish.
- `version = "0.14.1"` is unaffected by any of this — no bump needed (this is a rename+repair, not a functional release).

## Requirements
1. Rename `[package] name` from `"font_kit"` to `"boltz-font-kit"`.
2. Remove the dangling `readme = "README.md"` line (do not resurrect the deleted file — see ADR).
3. Restore `license = "MIT OR Apache-2.0"`.
4. Update root `Cargo.toml`'s `font_kit` workspace-dependency entry: add `package = "boltz-font-kit"` and `version = "0.14.1"`.
5. Append `boltz-font-kit` as the first entry in `scripts/publish-crates.sh`'s `PACKAGES` array (it has zero internal deps, so it's a valid first entry; keep the existing 16 entries below it or above it — order relative to the existing 16 doesn't matter since they have no dependency relationship, but it must come before every other crate added in phases 2-5).
6. Do NOT touch `[lib] name = "font_kit"` (line 11) or `path = "src/lib.rs"`-equivalent — the Rust-level crate identifier stays `font_kit` because every consumer (`gpui_macos`) resolves it via its own dependency-table key `font_kit`, not via `boltz-font-kit`.
7. Do NOT touch any `use font_kit::...` statement in `crates/gpui_macos/src/**` — none are needed.

## Architecture
No architectural change — this is a metadata-only fix plus the mechanical rename. `font_kit`'s only internal consumer is `gpui_macos` (macOS target-gated dependency, `crates/gpui_macos/Cargo.toml:32`, `font_kit.workspace = true`), unaffected by this phase since the dependency key stays `font_kit`.

## ADR Rationale (required — high-risk lane)

**Context**: Fixing the readme and license fields wasn't in the original task scope (which named only the name-collision) but was discovered during mandatory re-verification of the actual files. Two decisions needed: (a) restore the deleted README.md file, or remove the dangling pointer; (b) what license string to use.

**Decision (a) — remove the `readme =` pointer, don't restore the file**: Commit `3146812`'s explicit intent was "drop per-crate ... vendored LICENSE/README files" because "publishing metadata is managed elsewhere." Resurrecting a vendored README that documents a stale fork history (Zed-specific provenance notes, per the diff) would contradict that decision and reintroduce content nobody has committed to maintaining. Removing the field is the minimal fix consistent with the prior commit's stated intent, and unblocks the dry-run.

**Why not over B (restore full README)**: Would require deciding what the README should say (provenance? license notice? usage?) — a content decision out of scope for a mechanical safety fix, and one the prior commit already explicitly opted out of.

**Decision (b) — restore `license = "MIT OR Apache-2.0"` exactly as it was before deletion**: This is not a style choice. `font_kit` is vendored/forked from `servo/font-kit`, a third-party crate originally dual-licensed MIT/Apache-2.0. Removing the license field doesn't relicense the underlying code — it just makes the manifest non-compliant and the crate rejected by crates.io on real publish. Restoring the original upstream license string is the only legally accurate option; inventing a different license (e.g. matching this repo's own `GPL-3.0-or-later` convention used by sibling crates) would misrepresent a fork's actual licensing and is not this planner's call to make unilaterally — resolved in plan.md's Decisions section (item 1) as the default: simply restoring what was correct before the deletion.

**Why Phase 1 and not folded into a later "hygiene" phase**: The readme bug hard-fails the phase-06 dry-run for the ENTIRE `PACKAGES` array (script uses `set -euo pipefail` and `die`s on unexpected failures), so it must be fixed before phase-06 regardless of sequencing; doing it in phase-01 alongside the already-mandated font_kit touch avoids opening the file twice.

## Related code files
- `crates/font_kit/Cargo.toml:2` — `name = "font_kit"`
- `crates/font_kit/Cargo.toml:3` — `version = "0.14.1"` (unchanged)
- `crates/font_kit/Cargo.toml:4` — `description = "A cross-platform font loading library"` (unchanged, already fine)
- `crates/font_kit/Cargo.toml:5` — `readme = "README.md"` (delete this line)
- `crates/font_kit/Cargo.toml:8` — `publish = true` (unchanged)
- `Cargo.toml:47` — `font_kit = { path = "crates/font_kit" }`
- `scripts/publish-crates.sh:35-52` — `PACKAGES` array

## Implementation Steps

1. Edit `crates/font_kit/Cargo.toml`:
   ```diff
    [package]
   -name = "font_kit"
   +name = "boltz-font-kit"
    version = "0.14.1"
    description = "A cross-platform font loading library"
   -readme = "README.md"
   +license = "MIT OR Apache-2.0"
    edition = "2018"
    rust-version = "1.77"
    publish = true
   ```
   (Net effect: line 2 renamed, line 5 changed from `readme` to `license`.)

2. Edit root `Cargo.toml` line 47:
   ```diff
   -font_kit = { path = "crates/font_kit" }
   +font_kit = { path = "crates/font_kit", version = "0.14.1", package = "boltz-font-kit" }
   ```

3. Edit `scripts/publish-crates.sh`: insert `boltz-font-kit` into the `PACKAGES` array (line 35-52). Recommended: add as a new first line (line 36) since it has no internal deps and semantically kicks off "batch 2" of crates:
   ```diff
    PACKAGES=(
   +  boltz-font-kit
      boltz-collections
      boltz-derive-refineable
   ...
   ```

4. Verify: `cargo check -p boltz-font-kit` (compiles; package resolution only, no behavior change expected) and `cargo metadata --format-version 1 --no-deps | python3 -c "import sys,json;print([p['name'] for p in json.load(sys.stdin)['packages'] if 'font' in p['name']])"` to confirm the package now reports as `boltz-font-kit`.

## Todo list
- [ ] Rename `[package] name` in `crates/font_kit/Cargo.toml`
- [ ] Remove dangling `readme =` line
- [ ] Add `license = "MIT OR Apache-2.0"`
- [ ] Add `version` + `package` to root `Cargo.toml`'s `font_kit` entry
- [ ] Add `boltz-font-kit` to `scripts/publish-crates.sh` `PACKAGES` array
- [ ] `cargo check -p boltz-font-kit` passes
- [ ] `cargo check -p gpui_macos` passes (confirms downstream consumer unaffected)

## Success Criteria
- `cargo metadata` reports the crate at `crates/font_kit` under name `boltz-font-kit`.
- `cargo check --workspace` (macOS target) succeeds with no changes to any `use font_kit::` call site.
- `cargo package -p boltz-font-kit --list` (or `--dry-run` via the publish script) no longer errors on a missing readme file.

## Risk Assessment
- **Low technical risk**: metadata-only change, zero source-code edits.
- **Medium sequencing risk**: every other phase in this plan depends on this landing first (gpui_macos, and transitively gpui_platform/app, reference `font_kit.workspace = true`) — if phase-01 is skipped or partially applied, phase-06's dry-run will fail confusingly on an unrelated-looking readme error.
- **License correctness risk**: if "MIT OR Apache-2.0" turns out to be wrong (e.g. the vendored fork diverged enough to need a different notice), this needs legal/maintainer sign-off — resolved-but-reversible in plan.md's Decisions section (item 1), not a purely mechanical call.

## Security Considerations
- N/A for code security. Registry-identity note: once `boltz-font-kit` is published to crates.io it cannot be unpublished except within a 72-hour yank window, and the name is claimed permanently even if yanked — get the license string right before a real (non-dry-run) publish.

## Next steps
Proceed to phase-02 only after this phase's Todo list is fully checked and `cargo check --workspace` passes.
