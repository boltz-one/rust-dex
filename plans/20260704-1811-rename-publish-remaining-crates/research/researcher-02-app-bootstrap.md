# Research: Publish Strategy for `boltz-app` Bootstrap Template

## Executive Summary

**Recommendation**: Publish `boltz-app` as a binary-only crate for name reservation + CI symmetry with the 16 existing `boltz-*` crates on crates.io, BUT keep the actual bootstrap model **git-clone-based** (not installation-based). Minimize code changes; just rename metadata in `Cargo.toml` and add clear documentation that the template is consumed by cloning, not `cargo install`.

**Rationale**: The repo's stated vision (PDR + README) is explicitly "fork/clone this template; edit `crates/app/src/main.rs` in place." A published binary has limited standalone value (it's ~104 lines of demo code), and the Rust ecosystem already has two proven patterns for templates: (a) `cargo-generate` templates (NOT crates.io published) for scaffolding, and (b) published CLI tools for actual functionality. `boltz-app` fits neither cleanly. Publishing it reserves the name and keeps the publish pipeline uniform, but is not meant for end-users to call `cargo install boltz-app`.

---

## Current State (Evidence)

### Bootstrap Model
- **Repository model**: Explicit git-clone / fork-based [README.md:lines 14-36](file:///Users/nguyendk/Documents/projects/me/rust-dex/README.md) states "Start developing: Open `crates/app/src/main.rs` (70 lines, fully commented)" and "Typical Next Steps: 1. Add custom UI — Edit `crates/app/src/main.rs` to build out screens."
- **Quick Start**: `make dev` (runs locally in repo context), not `cargo install`.
- **Current package name**: `app` (line 4, `Cargo.toml`) — unrelated existing crate on crates.io.

### Publish Infrastructure
- **Workspace config**: `[workspace.package] publish = true` (Cargo.toml:39) — recent change; now symmetric across all 16 crates.
- **Naming pattern**: 16 crates already live on crates.io under `boltz-*` names (e.g., `boltz-gpui`, `boltz-collections`) per workspace deps (Cargo.toml:44–50).
- **Repository URL**: Already configured to `github.com/boltz-one/rust-dex` (Cargo.toml:41).

### App Crate Structure
- **Entry point**: Binary-only (`[[bin]]`, line 12 in `crates/app/Cargo.toml`), no lib.
- **Code size**: `src/main.rs` is ~104 lines; minimal `HelloWorldApp` with hardcoded demo (one "hello world" label in a centered window).
- **No library API**: No `lib.rs`; zero reusable scaffolding logic exposed for external consumption.

### Ecosystem Patterns (General Knowledge)
- **Pattern A (cargo-generate)**: Rust starter templates (e.g., `bevy_game_template`, `cargo_generate_template`) use `.template/` dirs + `cargo-generate.toml` metadata. Consumed via `cargo generate --git <repo>`, NOT crates.io published. Git-based delivery.
- **Pattern B (Published CLI)**: Tools like `cargo-generate`, `cargo-watch` are published to crates.io; users run `cargo install <tool>` and gain real functionality (not just scaffolding).
- **Pattern C (Fork/Clone)**: This repo — consumers clone/fork and edit in place. No installation step. Templates are discovered and consumed via git.

---

## Does Publishing `boltz-app` to crates.io Add Value?

**Short answer**: Limited direct user value; high value for name reservation + CI/publish pipeline consistency.

### Scenario: What happens if user does `cargo install boltz-app`?
1. User gets a runnable binary that opens a "hello world" window.
2. User cannot edit or customize the installed binary (it's compiled, opaque).
3. User would need to clone the repo anyway to start developing a NEW app (the actual goal).
4. Installing the binary does NOT scaffold a new project or provide a template to build on.

**Conclusion**: Publishing is not a scaffolding tool; it's a name reservation + symmetry mechanism.

### Why still publish?
1. **Name reservation**: Prevents name squatting; documents intent to the ecosystem.
2. **CI/publish symmetry**: All 16 workspace crates use uniform `publish = true`; no special cases simplify maintenance.
3. **Preview capability** (minor): Users curious about the template can `cargo install boltz-app && boltz-app` to see a working GPUI app before committing to clone the repo. Low friction discovery.
4. **Future-proofing**: If you later decide to add a `lib` or CLI subcommand for scaffolding, the crate is already "in the ecosystem."

---

## Implementation: What Concrete Changes?

### Minimal (Recommended)
1. **Rename in `crates/app/Cargo.toml`**:
   - Line 4: `name = "app"` → `name = "boltz-app"`
   - Line 3: `description = "A clean GPUI desktop application base."` → Consider: `"A minimal GPUI desktop app template. Fork/clone the repo to start building. See https://github.com/boltz-one/rust-dex#quick-start."`

2. **Update `Makefile`**:
   - Line 6: `PACKAGE := app` → `PACKAGE := boltz-app`

3. **Update `docs/code-standards.md` and `docs/project-overview-pdr.md`**:
   - Flag that these files say `publish = false` (stale). Correct to note: "Published on crates.io as `boltz-app` for name reservation; bootstrap consumed via git clone."
   - Clarify: "Publish = true is workspace-level; consuming the `boltz-app` binary directly is not the primary bootstrap flow. Fork the repo and edit `crates/app/src/main.rs` in place."

4. **No code logic changes**: Keep `src/main.rs` as-is (~104 lines). It's already a working example.

### DO NOT (unnecessary complexity):
- Split into separate `lib.rs` + `bin.rs` unless you add library scaffolding API (out of scope for now).
- Add a `scaffold()` / `init()` function or `boltz-app new <name>` subcommand (use cargo-generate if this is desired later).
- Add `.template/` directory or `cargo-generate.toml` (separate concern; recommend as future enhancement if users request it).

---

## Should a `cargo-generate` Template be Complementary?

**Answer**: Not required now; recommend as future optional enhancement.

### Rationale
- **Current model works**: Cloning the repo and editing in place is simple and proven. No scaffolding layer needed yet.
- **Future trigger**: If you observe users struggling with "how do I start a new app" beyond the main repo, add a `.template/` dir with:
  - `cargo-generate.toml` (placeholders: `{% raw %}{{project-name}}{% endraw %}`, `{% raw %}{{authors}}{% endraw %}`)
  - Template files with variable substitution (e.g., `src/main.rs` → `app_id` parameterized)
- **Separate concern**: cargo-generate delivery does NOT require crates.io publishing; it's purely git-based (`cargo generate --git https://github.com/boltz-one/rust-dex --name my_app`).
- **Recommendation**: Add to a future phase (e.g., "Post-publish refinement") if usage data suggests the need.

---

## Docs Flag: Current Stale References

**File**: `docs/project-overview-pdr.md`
- States (incorrectly): "Publishing: disabled (publish = false in workspace)"
- Correct to: "Published on crates.io under `boltz-*` crate names (workspace-level `publish = true`). `boltz-app` reserved for the template entry point; consumers fork/clone the repo for development."

**File**: `docs/code-standards.md`
- Review for similar publish state assertions; update to align.

---

## Decision Summary

| Aspect | Recommendation | Rationale |
|--------|---|---|
| Publish `boltz-app`? | YES (binary-only) | Name reservation + publish pipeline symmetry |
| Add lib or CLI logic? | NO (not now) | Bootstrap is git-clone-based; no packaged scaffolding needed yet |
| Split lib/bin? | NO | Keep single binary; no exposed library API |
| Add cargo-generate? | Future phase | Git-clone model works; recommend if users request scaffolding |
| Rename Makefile target? | YES | Update `PACKAGE := app` → `PACKAGE := boltz-app` |
| Update docs? | YES | Flag stale `publish = false` references in PDR + code-standards |

---

## Open Questions

1. **User intent**: Is the PRIMARY goal to "make it easy for others to fork and start building" (✓ current model), or to "provide one-command scaffolding" (requires cargo-generate or CLI subcommand)? — Assumed first based on README vision.
2. **Publishing cadence**: Will `boltz-app` version number track the rest of the workspace crates, or have its own semantic versioning? — Recommend tracking workspace (all v0.1.0, then v0.2.0, etc.) for simplicity.
3. **Future cargo-generate**: Should a `.template/` companion be added in a follow-up phase if early adopters request "cargo generate" convenience? — Flag as **optional future**, not blocking.
4. **Crate.io metadata**: Should the crate.toml include a link to the repo or a Makefile example in docs/README? — Recommend adding in crate README (auto-published to crates.io) pointing users to the git repo for the real bootstrap experience.

---

**Sources**:
- Repo: `Cargo.toml` lines 1–50 (workspace + publish config)
- Repo: `crates/app/Cargo.toml` (package metadata)
- Repo: `crates/app/src/main.rs` (104-line binary entry point)
- Repo: `README.md` lines 14–36 (explicit "fork/clone, edit in place" model)
- Repo: `Makefile` lines 1–20 (build targets)
- General knowledge: Rust ecosystem bootstrap patterns (cargo-generate, published CLI tools, fork-based templates)
