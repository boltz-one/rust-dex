# Phase 05: app → boltz-app + Bootstrap Upgrade

## Context links
- Plan: [plan.md](./plan.md)
- Prior phases: [phase-01](./phase-01-font-kit-safety-fix.md) through [phase-04](./phase-04-gpui-platform-and-ui-rename.md) (all must land first — `app` depends on `gpui_platform`, `theme`, `ui`)
- Research: `research/researcher-02-app-bootstrap.md` (recommends Option A / minimal; this phase overrides that recommendation — see ADR)

## Overview
- Date: 2026-07-04
- Description: Rename `app` → `boltz-app` (package AND binary — see ADR decision 1), then implement a `cargo-generate`-compatible standalone template under `template/` so a new user can `cargo generate --git <repo> --subfolder template && cargo run` instead of forking the entire 35-crate monorepo — see ADR decision 2 for why this is now possible and worth doing.
- Priority: P1 (rename half is required; template half is the separable, cuttable part — see Risk Assessment)
- Status: not-started

## Key Insights
- Current state: `crates/app/Cargo.toml` — `[package] name = "app"`, `default-run = "app"`, `[[bin]] name = "app"` (lines 4, 7, 13). Binary-only, no `[lib]`.
- `Makefile:3` — `PACKAGE := app`, used by `cargo run -p $(PACKAGE)` (line 8) and `cargo check -p $(PACKAGE)` (line 11). If only `[package] name` is renamed and `Makefile` isn't updated, `make dev`/`make check` breaks immediately (`error: package ID specification 'app' did not match any packages`).
- `crates/app/src/main.rs` (103 lines, read in full): hardcodes `const APP_ID: &str = "com.example.app"` (line 10) and `window.set_window_title("App")` (line 86) — both are exactly the kind of per-project values a bootstrap template needs to parameterize.
- `README.md` and `docs/project-overview-pdr.md`/`docs/code-standards.md` currently describe the ONLY bootstrap flow as "fork/clone the whole repo, edit `crates/app/src/main.rs` in place" (`README.md:31`, "Start developing: Open `crates/app/src/main.rs`"). Both docs also assert `publish = false` workspace-wide (`docs/project-overview-pdr.md:99`, `docs/code-standards.md:8`), which is now stale (workspace `publish = true` since a recent unrelated commit) — this phase should correct those two lines since it directly changes what `app`/`boltz-app` means for publishing.
- crates.io name `boltz-app` confirmed available (researcher-01, HTTP 404 check).

## Requirements
1. Rename `[package] name` "app" → "boltz-app" in `crates/app/Cargo.toml`.
2. Rename `[[bin]] name` and `default-run` "app" → "boltz-app" (ADR decision 1).
2b. Add `license = "GPL-3.0-or-later"` to `crates/app/Cargo.toml` — `app` is one of only 3 crates in this plan missing `license` (with `ui`, `font_kit`; both fixed in their own phases), and it's a hard blocker for a real crates.io publish, not a soft warning. Matches the UI-application layer convention (`ui`, `theme`, `component`, `icons`, `menu`, `syntax_theme`, `ui_macros` — all `GPL-3.0-or-later`), which `app` sits directly on top of; the `Apache-2.0` convention belongs to the platform/infra layer (`gpui_platform` family, `collections`, `util`) that `app` only consumes transitively.
3. Update `Makefile:3` `PACKAGE := app` → `PACKAGE := boltz-app`.
4. Update root `Cargo.toml` `app` workspace-dependency entry (line 178) — note: nothing currently depends on `app` via workspace deps (it's a leaf/root of the dependency graph, the binary crate itself), so this entry mainly exists for consistency/potential future lib extraction; still add `version`/`package` for pattern consistency and so `cargo metadata` reports it correctly.
5. Add a standalone `template/` directory implementing Option B (ADR decision 2) — a `cargo-generate`-compatible copy of the app skeleton that depends on the now-`boltz-*`-published crates via crates.io versions, not path deps.
6. Correct the two stale `publish = false` doc lines identified above.
7. Do NOT change `[profile.release.package] app = { codegen-units = 16 }` in root `Cargo.toml:272` silently — this profile override key must ALSO become `boltz-app` (profile package overrides key on the package name, not the workspace-dependency key) — this is a real, easy-to-miss edit distinct from the `[workspace.dependencies]` entry.

## Architecture
- `boltz-app` remains the in-repo, path-dependency-based binary — unchanged role, just renamed (package + binary identifiers).
- NEW: `template/` is a self-contained, separate Cargo project (its own `[workspace]` root — see Implementation Steps step 5c for why this matters) living inside this repo's git tree but outside this repo's Cargo workspace. It is never built by `cargo check --workspace` from the repo root and is not part of `scripts/publish-crates.sh` (it has nothing to publish — `cargo-generate` consumes it via git, not crates.io).

```
rust-dex/                          (existing monorepo, unchanged)
├── crates/app/                    (renamed to boltz-app, otherwise unchanged)
├── ...
└── template/                      (NEW — standalone cargo-generate template)
    ├── cargo-generate.toml
    ├── Cargo.toml                 ([workspace] + [package], depends on boltz-* via crates.io versions)
    ├── .gitignore
    ├── README.md                  (3-line quick start)
    └── src/
        └── main.rs                (copy of crates/app/src/main.rs, APP_ID + window title templated)
```

## ADR Rationale (required — high-risk lane, this is the judgment-call phase)

### Decision 1: rename BOTH `[package] name` and `[[bin]] name`/`default-run` to `boltz-app`

**Context**: researcher-02 only proposed renaming `[package] name`, leaving `[[bin]] name = "app"` untouched.

**Why that's wrong**: `cargo install boltz-app` installs whatever binary `[[bin]] name` specifies into the user's `~/.cargo/bin/`. If it stays `"app"`, the installed binary is literally named `app` — a maximally generic, collision-prone name in a global `$PATH` (many tools/scripts are named `app`). There's no ergonomic upside to keeping a short/different bin name here (unlike e.g. `ripgrep`→`rg`, which is a deliberate typing-ergonomics choice for a CLI tool used constantly) — GPUI apps aren't invoked by hand dozens of times a day, so binary brevity isn't a real requirement.

**Decision**: rename `[[bin]] name` and `default-run` to `"boltz-app"` too, matching the package name. Requires `Makefile:3` `PACKAGE := boltz-app` in lockstep (Makefile uses `-p $(PACKAGE)`, which is the package-ID flag — this alone would still work with the package rename even if bin name didn't change, but since both change together it's one coordinated edit).

### Decision 2: implement Option B (rename + standalone `template/`) over researcher-02's Option A (rename only, defer tooling)

**Context**: user's explicit ask (Vietnamese, task brief): "app should become package `boltz-app`, a place to bootstrap a NEW app faster/simpler than today." researcher-02 recommended renaming only and deferring all scaffolding to "future, optional," reasoning that the git-clone-the-whole-monorepo flow "already works" and a template is extra untested surface area.

**Why researcher-02's recommendation under-delivers**: cloning a 35-crate monorepo (most of it GPUI internals a new-app author will never touch) to start one small app is not "faster or simpler" than what exists today — it IS what exists today. Renaming `app`→`boltz-app` alone changes zero bytes of the actual bootstrap experience; a user still forks the whole repo. This doesn't answer the user's ask, it just does the mechanical half of this plan's overall theme (renaming) and calls it done.

**Why Option B is now specifically possible (and wasn't before this plan)**: `cargo-generate --git <repo> --subfolder template` scaffolds ONLY the `template/` subfolder into a fresh directory, then the generated project's own `Cargo.toml` must resolve its dependencies (`gpui`, `gpui_platform`, `theme`, `ui`) somehow. Before this rename effort, those crates only existed as **path** dependencies inside this monorepo — a standalone generated project has no such paths, so a template would have been unbuildable (or would have to vendor/re-clone the monorepo anyway, defeating the purpose). This entire 4-phase rename (phases 01-04) is what makes those crates resolvable by **version** from crates.io — `gpui = { version = "0.2.2", package = "boltz-gpui" }` — which is exactly the trick already used inside this repo's own root `Cargo.toml` and is copy-pasteable into a totally separate, standalone `Cargo.toml`. Put simply: Option B is the payoff this whole rename effort was for; doing the rename and NOT building the one thing it unlocks is leaving the actual point on the table.

**Cost/risk being accepted**: new directory to maintain, `cargo-generate.toml` authoring (new to this repo), a second `Cargo.toml`/`main.rs` to keep loosely in sync with `crates/app`'s if that file evolves, and template-placeholder testing (`cargo generate` needs an actual dry run to confirm it produces a buildable project) — this is real, ongoing surface area, not free. Mitigated by keeping the template as close to a literal copy of `crates/app/src/main.rs` as possible (only 2 lines templated: `APP_ID`, window title) rather than building original scaffolding logic.

**Why this is still cleanly separable / cuttable**: `template/` has zero build/dependency edge into the rest of the workspace (it's not a workspace member, nothing in `crates/*` depends on it). If the user wants to descope, phase-05 can ship as rename-only (Requirements 1-4, 6-7) and the template (Requirements 5) can be dropped or deferred to a follow-up plan without touching anything already merged. This is resolved in plan.md's Decisions section (item 4): Option B confirmed, but the separability is preserved as a safety net.

### Decision 3: template consumed via `cargo generate --subfolder`, not a separate published crate or a dotfile-hidden `.template/`

Two-doc-comparison: cargo-generate supports pointing at a subfolder of an existing git repo (`--subfolder <path>`) so the template does NOT need to be its own repo. A plain `template/` (no leading dot) is used rather than `.template/` because the directory is meant to be explicitly discoverable and documented (linked from `README.md`), not hidden — hiding it would work technically but adds no value and makes it harder for a human browsing the repo to find.

## Related code files
- `crates/app/Cargo.toml:4` (`name`), `:7` (`default-run`), `:13` (`[[bin]] name`)
- `Makefile:3` (`PACKAGE := app`)
- `Cargo.toml:178` (`app` workspace-dependency entry), `:272` (`[profile.release.package] app = ...`)
- `README.md:31` ("Start developing: Open `crates/app/src/main.rs`" — update to mention `template/` as the new fast path, keep the existing line as the "deep customization" path)
- `docs/project-overview-pdr.md:99` (stale `publish = false`)
- `docs/code-standards.md:8` (stale `publish = false`)
- NEW: `template/Cargo.toml`, `template/cargo-generate.toml`, `template/src/main.rs`, `template/README.md`, `template/.gitignore`

## Implementation Steps

1. `crates/app/Cargo.toml`:
   ```diff
    [package]
    description = "A clean GPUI desktop application base."
    edition.workspace = true
   -name = "app"
   +name = "boltz-app"
    version = "0.1.0"
    publish.workspace = true
   +license = "GPL-3.0-or-later"
   -default-run = "app"
   +default-run = "boltz-app"

    [lints]
    workspace = true

    [[bin]]
   -name = "app"
   +name = "boltz-app"
    path = "src/main.rs"
   ```

2. `Makefile`:
   ```diff
   -PACKAGE := app
   +PACKAGE := boltz-app
   ```

3. Root `Cargo.toml`:
   ```diff
   -app = { path = "crates/app" }
   +app = { path = "crates/app", version = "0.1.0", package = "boltz-app" }
   ```
   and:
   ```diff
    [profile.release.package]
   -app = { codegen-units = 16 }
   +boltz-app = { codegen-units = 16 }
   ```

4. `docs/project-overview-pdr.md:99` and `docs/code-standards.md:8`: change `- **Publishing**: disabled (\`publish = false\` in workspace)` (and code-standards' equivalent line) to reflect `publish = true` at workspace level with the `boltz-*` naming convention, per the ALREADY-existing correct explanation elsewhere in `docs/code-standards.md:56,90` (`Published names: prefixed with boltz- in Cargo.toml`).

5. Create `template/` (new standalone project, not a workspace member):

   a. `template/Cargo.toml`:
   ```toml
   [workspace]

   [package]
   name = "{{project-name}}"
   version = "0.1.0"
   edition = "2024"
   publish = false

   [dependencies]
   gpui = { version = "0.2.2", package = "boltz-gpui", default-features = false }
   gpui_platform = { version = "0.1.0", package = "boltz-gpui-platform" }
   theme = { version = "0.1.0", package = "boltz-theme" }
   ui = { version = "0.1.0", package = "boltz-ui" }
   ```
   The bare `[workspace]` table with no `members` is required — it marks this manifest as its OWN workspace root, decoupling it from the parent monorepo's workspace (without it, Cargo would try to walk up the directory tree, find the repo-root `Cargo.toml`, and either error that this path isn't a listed member or attempt unwanted inheritance). Version numbers here must be bumped to match whatever the crates' actual first-published versions end up being (0.1.0 assumed per this plan; 0.2.2 for gpui, already published).

   b. `template/cargo-generate.toml`:
   ```toml
   [template]
   cargo_generate_version = ">=0.20.0"
   ```
   (No custom `[placeholders]` needed — `{{project-name}}` and `{{authors}}` are cargo-generate built-ins.)

   c. `template/src/main.rs`: copy `crates/app/src/main.rs` verbatim, then template exactly 2 values:
   ```diff
   -const APP_ID: &str = "com.example.app";
   +const APP_ID: &str = "com.example.{{project-name}}";
   ...
   -                    window.set_window_title("App");
   +                    window.set_window_title("{{project-name}}");
   ```
   No other changes — every `use gpui::...`/`use theme::...`/`use ui::...` line stays IDENTICAL to `crates/app/src/main.rs` because the template's `Cargo.toml` dependency keys (`gpui`, `gpui_platform`, `theme`, `ui`) match the monorepo's own keys exactly (see Decision 2's ADR rationale) — this is the concrete payoff of the alias trick.

   d. `template/README.md` (new, ~10 lines): `cargo generate --git https://github.com/boltz-one/rust-dex --subfolder template --name my-app && cd my-app && cargo run` plus a one-line pointer back to the monorepo's `docs/` for deeper customization.

   e. `template/.gitignore`: `/target`

6. Update root `README.md:31` area to add a "Fast path" callout above the existing "Start developing" line, pointing at `template/README.md`'s `cargo generate` command, while keeping the existing fork/clone instructions as the "deep customization / contribute to boltz itself" path.

## Todo list
- [ ] Rename `[package] name`, `default-run`, `[[bin]] name` in `crates/app/Cargo.toml`
- [ ] Add `license = "GPL-3.0-or-later"` to `crates/app/Cargo.toml`
- [ ] Update `Makefile` `PACKAGE :=`
- [ ] Update root `Cargo.toml` `app` workspace-dependency entry AND `[profile.release.package]` key
- [ ] Fix stale `publish = false` claims in `docs/project-overview-pdr.md` and `docs/code-standards.md`
- [ ] Create `template/Cargo.toml` with own `[workspace]` root + version-pinned `boltz-*` deps
- [ ] Create `template/cargo-generate.toml`
- [ ] Create `template/src/main.rs` (copy + 2 templated values)
- [ ] Create `template/README.md`, `template/.gitignore`
- [ ] Add a "Fast path" callout to root `README.md`
- [ ] `cargo check -p boltz-app` passes (monorepo side)
- [ ] `make dev` / `make check` still work with new `PACKAGE` value
- [ ] Manual `cargo generate --path ./template --name test-boltz-app` dry run in a scratch dir (uses `--path` not `--git` for local testing before the real crates.io publish makes `--git` resolvable) — confirm generated `Cargo.toml`/`main.rs` have `{{project-name}}` substituted correctly. NOTE: `cargo run` in the generated scratch project will NOT succeed until `boltz-gpui-platform`/`boltz-theme`/`boltz-ui` are actually live on crates.io (this plan only reaches `DRY_RUN` in phase-06) — so this dry run validates templating substitution only, not a full build; document that limitation in the PR/commit description.

## Success Criteria
- `cargo metadata` (root workspace) reports `boltz-app` correctly; `template/` does not appear in root workspace metadata at all (confirms it's properly decoupled).
- `make dev` opens the app exactly as before (behavior-identical, only names changed).
- `cargo generate --path ./template --name demo` (local path mode) produces a `demo/` directory with `Cargo.toml`/`src/main.rs` containing `demo` substituted for every `{{project-name}}` occurrence, and zero literal `{{project-name}}` left over.

## Risk Assessment
- **Rename half (Requirements 1-4, 6-7)**: low risk, same category as phases 02-04.
- **Template half (Requirement 5)**: medium risk — new untested mechanism, and its end-to-end "does it actually build" promise can't be FULLY verified until the dependent crates are truly live on crates.io (post-dry-run, post-real-publish, which is out of this plan's scope per plan.md's Decisions section item 5). Explicitly separable: if time-boxed out, ship Requirements 1-4/6-7 now and treat Requirement 5 as its own follow-up plan.
- Missing `description` field on `ui`'s Cargo.toml (noted in phase-04) means the template's own dependency on `boltz-ui` is unaffected (description doesn't gate resolution), but is worth folding into the same pre-real-publish `description` hygiene pass deferred in plan.md's Decisions section (item 3).

## Security Considerations
- N/A for application logic. The `template/` directory becomes a public, git-clonable entry point once this repo is public — ensure `template/README.md` doesn't contain placeholder secrets or example tokens (it won't, per the design above — only a `cargo generate` command and a `cargo run`).
- Registry-identity note (shared with phase-01): `boltz-app` on crates.io is permanent once published for real; get the bin/package naming decision (ADR decision 1) confirmed before a real publish, since renaming a published binary crate later means users' installed `boltz-app` binary name can't retroactively change.

## Next steps
Proceed to phase-06 (verify + dry-run) once phase-05's rename half is confirmed working; the template half's local `cargo generate --path` dry run can run in parallel with phase-06 since it has no dependency on the publish pipeline.
