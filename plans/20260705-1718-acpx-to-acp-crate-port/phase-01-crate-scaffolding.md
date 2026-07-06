# Phase 1: Crate Scaffolding + Workspace Registration + Dependency Choices

## Context links

- Plan: [plan.md](./plan.md)
- Research: [researcher-01-rust-acp-ecosystem.md](./research/researcher-01-rust-acp-ecosystem.md), [researcher-02-acpx-architecture.md](./research/researcher-02-acpx-architecture.md)
- Next phase: [Phase 2](./phase-02-protocol-transport-lifecycle.md)

## Overview

- **Date:** 2026-07-05
- **Description:** Stand up the `crates/acp` crate skeleton (published as `boltz-acpx`), register it in the workspace, pin its dependency set, and land the foundational error/type/control modules every later phase imports. No protocol logic yet.
- **Priority:** P1 (blocks all other phases)
- **Implementation status:** Done — `cargo check -p boltz-acpx`, `cargo test -p boltz-acpx` (7 passed), `make check-all`, `cargo fmt --all -- --check` all green. Lib entry file confirmed as `src/acp.rs` (matches `http_client`/`scheduler`/`theme` convention). Pinned `agent-client-protocol = "=1.0.1"`, `agent-client-protocol-schema = "=1.2.0"` in workspace deps (not yet used by crate code — Phase 2 will add them to `crates/acp/Cargo.toml`'s `[dependencies]` when first consumed, avoiding an unused-dependency warning window).
- **Review status:** Not reviewed

## Key Insights

- `crates/acp/Cargo.toml` exists but is 0 bytes — bare scaffold, no `[package]` section. `crates/acp` is **not** in root `Cargo.toml` `[workspace.members]`.
- Naming convention (verified against `crates/scheduler/Cargo.toml`, `crates/http_client/Cargo.toml`): crate dir `acp`, package name `boltz-acpx`, `edition.workspace = true`, `publish = true`, `license = "Apache-2.0"`, `repository.workspace = true`, `[lints] workspace = true`.
- `agent-client-protocol` and `agent-client-protocol-schema` (crates.io, github.com/agentclientprotocol/rust-sdk) are **not yet** in `[workspace.dependencies]` — must be added here since acpx's protocol types get reused via ADR-1 (Phase 2), and the schema crate is needed even before Phase 2 lands so `types.rs`/`error.rs` can reference its enums where they overlap (e.g. `ToolKind`, `StopReason`).
- Workspace already carries `smol = "2.0"`, `futures = "0.3.32"`, `futures-lite = "1.13"`, `parking_lot`, `thiserror = "2.0.12"`, `anyhow`, `serde`/`serde_json`, `chrono`, `uuid`, `dirs`, `tempfile` — all reusable, no new workspace deps needed for these.
- `crates/util` (`boltz-util`) already vendors a cross-platform subprocess abstraction (`src/command.rs`, `src/command/darwin.rs`, `src/process.rs`) built on `smol::process`, including the macOS `posix_spawn` quirk isolated behind a facade. `acp` should depend on `util` from Phase 2 onward rather than re-implementing spawn/kill — flagged here so the Cargo.toml dependency list is correct from the start.
- File-size convention: <200 lines/file. acpx's `types.ts` (470 lines) and `errors.ts` (220 lines) do not fit in one Rust file each — split by concern (see Related code files).

## Requirements

1. `crates/acp/Cargo.toml` fully populated per workspace convention, added to root `[workspace.members]` and `[workspace.dependencies]`.
2. `agent-client-protocol` + `agent-client-protocol-schema` added to `[workspace.dependencies]` at the version used by Zed's current release (verify latest on crates.io at implementation time — do not hardcode a stale version into the plan).
3. Crate compiles standalone (`cargo check -p boltz-acpx`) with only foundational modules (no protocol logic): `lib.rs`, `error.rs`, `types.rs`, `control.rs`, `platform/` (empty scaffolds with doc comments describing what Phase 2/3 will add).
4. `[features] test-support = []` feature flag defined (matches `http_client`/`scheduler` pattern) for later test-only fake-agent binary wiring (Phase 2+).
5. `make check-all` still passes after the crate is added (verify no dependency version conflicts).

## Architecture

```
crates/acp/
├── Cargo.toml
└── src/
    ├── lib.rs            # crate root, module declarations, re-exports
    ├── error.rs          # AcpError enum (thiserror) — ports errors.ts + jsonrpc-error.ts code table
    ├── types.rs           # PermissionMode, NonInteractivePermissionPolicy, SessionResumePolicy,
    │                       # AuthPolicy — the CLI-agnostic subset of acpx's types.ts (drop
    │                       # OUTPUT_*, EXIT_CODES, OutputFormatter — those are cli/ only, out of scope)
    ├── control.rs         # timeout/cancellation helpers — ports async-control.ts onto
    │                       # smol::Timer + futures::future::{select, Either}
    └── platform/
        ├── mod.rs         # re-exports; single place other modules import OS-branching helpers from
        └── liveness.rs    # is_process_alive(pid) — ports process-liveness.ts; #[cfg(unix)] kill(pid,0)
                            # vs #[cfg(windows)] OpenProcess, isolated here per ADR below
```

## ADR Rationale

### ADR-0: Where do OS `#[cfg]` gates for this crate live?

- **Context:** `docs/code-standards.md` restricts `#[cfg(target_os)]` gates to `gpui_platform`, `gpui_macos`/`gpui_linux`/`gpui_windows`, and `font_kit` — never in app-layer or platform-agnostic crates. `acp` is neither an app crate nor one of the named platform crates, yet it unavoidably needs a few OS branches: process liveness check (`kill(pid, 0)` on POSIX vs `OpenProcess`/`GetExitCodeProcess` on Windows) and, later, terminal process-group kill semantics (Phase 3).
- **Decision:** Concentrate every OS branch for this crate inside `crates/acp/src/platform/`, mirroring the same isolation principle the named platform crates follow, rather than scattering `#[cfg]` through `client/`, `terminal/`, etc. `platform/` re-exports OS-agnostic function signatures; callers never see `#[cfg]`.
- **Why this over alternatives:** (a) Splitting `acp` into `acp` + `acp_macos`/`acp_linux`/`acp_windows` crates (mirroring gpui's pattern exactly) is over-engineering for ~2 small functions — YAGNI. (b) Scattering `#[cfg]` inline at each call site violates the codebase's stated isolation intent even if it isn't the literal named exception list. (c) A single `platform/` module gives one auditable location, consistent with the *spirit* of the rule (isolate, don't scatter) without adding crate-graph overhead.

## Related code files

- `crates/acp/Cargo.toml` (currently 0 bytes) — to be filled in this phase.
- `Cargo.toml` (root) — add `"crates/acp"` to `[workspace.members]`, add `acp = { path = "crates/acp", version = "0.1.0", package = "boltz-acpx" }` and `agent-client-protocol`/`agent-client-protocol-schema` to `[workspace.dependencies]`.
- Reference for Cargo.toml shape: `crates/http_client/Cargo.toml`, `crates/scheduler/Cargo.toml`.
- Reference for platform isolation pattern: `crates/gpui_platform/src/gpui_platform.rs`, `crates/util/src/command/darwin.rs`.
- Source to port: `others/acpx/src/types.ts` (470 lines, partial — CLI-only types excluded), `others/acpx/src/errors.ts` (220 lines), `others/acpx/src/async-control.ts` (81 lines), `others/acpx/src/process-liveness.ts` (12 lines).

## Implementation Steps

1. Read `others/acpx/src/errors.ts` in full; enumerate every custom error class (e.g. `PermissionDeniedError`, `PermissionPromptUnavailableError`, `SessionNotFoundError`, `SessionResolutionError`, `SessionResumeRequiredError`, `SessionModeReplayError`, `SessionModelReplayError`, `SessionConfigOptionReplayError`) and the `OUTPUT_ERROR_CODES`/JSON-RPC error-code table from `others/acpx/src/acp/jsonrpc-error.ts`. Design one `thiserror` enum `AcpError` covering all of them with `#[error("...")]` messages and a `code()` method returning the JSON-RPC error code where applicable.
2. Write `crates/acp/src/error.rs` with `AcpError` + `pub type Result<T> = std::result::Result<T, AcpError>`.
3. Write `crates/acp/src/types.rs`: port `PermissionMode` (`deny-all`/`approve-reads`/`approve-all` as a Rust enum with an `Ord`-derived rank, replacing `PERMISSION_MODE_RANK`), `NonInteractivePermissionPolicy`, `SessionResumePolicy`, `AuthPolicy`. Use `serde(rename_all = "kebab-case")` to preserve wire/config compatibility with acpx's string literals if config files are ever shared.
4. Write `crates/acp/src/control.rs`: port `withTimeout`/`InterruptedError`/`TimeoutError` from `async-control.ts` using `futures::future::{select, Either}` + `smol::Timer::after(duration)`. Provide a `pub async fn with_timeout<F: Future>(fut: F, timeout: Duration) -> Result<F::Output>` returning `AcpError::Timeout` on expiry.
5. Write `crates/acp/src/platform/liveness.rs`: `pub fn is_process_alive(pid: u32) -> bool` — `#[cfg(unix)]` via `libc::kill(pid, 0)` (note: `libc` already a workspace dep), `#[cfg(windows)]` via `windows::Win32::System::Threading::OpenProcess`/`GetExitCodeProcess` (the `windows` workspace dependency already has `Win32_System_Threading` feature enabled — confirm at implementation time, add if missing).
6. Write `crates/acp/src/lib.rs` declaring `pub mod error; pub mod types; pub mod control; mod platform;` with a crate-level `//!` doc comment describing scope (link back to this plan).
7. Write `crates/acp/Cargo.toml`:
   ```toml
   [package]
   name = "boltz-acpx"
   version = "0.1.0"
   edition.workspace = true
   publish = true
   license = "Apache-2.0"
   description = "Agent Client Protocol (ACP) client/runtime for GPUI"
   repository.workspace = true
   [lints]
   workspace = true

   [lib]
   path = "src/acp.rs"   # OR src/lib.rs — confirm against latest convention in a sibling crate at impl time

   [features]
   test-support = []

   [dependencies]
   anyhow.workspace = true
   thiserror.workspace = true
   futures.workspace = true
   parking_lot.workspace = true
   serde.workspace = true
   serde_json.workspace = true
   chrono.workspace = true
   uuid.workspace = true
   log.workspace = true

   [target.'cfg(not(target_family = "wasm"))'.dependencies]
   smol.workspace = true
   ```
   (NOTE: check whether sibling crates name their lib entry file `src/<crate>.rs` — e.g. `http_client` uses `src/http_client.rs`, `scheduler` uses `src/scheduler.rs` — this workspace's convention deviates from the `src/lib.rs` default. Follow that convention: `src/acp.rs`, not `src/lib.rs`. Update all file paths in this plan's "Architecture" tree accordingly at implementation time — `lib.rs` mentioned above is a placeholder name only.)
8. Add `agent-client-protocol` / `agent-client-protocol-schema` to root `Cargo.toml` `[workspace.dependencies]` at latest crates.io version, and `"crates/acp"` to `[workspace.members]` + `acp = { path = "crates/acp", ... package = "boltz-acpx" }` to `[workspace.dependencies]`.
9. Run `cargo check -p boltz-acpx` and `make check-all`; fix any workspace dependency-resolution conflicts.
10. `cargo fmt --all -- --check`.

## Todo list

- [ ] Confirm the lib entry-file naming convention (`src/acp.rs` vs `src/lib.rs`) against 2+ sibling crates before writing `Cargo.toml`'s `[lib] path`.
- [ ] Populate `crates/acp/Cargo.toml`.
- [ ] Add `crates/acp` to root `Cargo.toml` workspace members + dependencies.
- [ ] Add `agent-client-protocol` + `agent-client-protocol-schema` to root `Cargo.toml` workspace dependencies (pin exact version used).
- [ ] Write `error.rs`, `types.rs`, `control.rs`, `platform/mod.rs`, `platform/liveness.rs`.
- [ ] Write crate-root doc comment in the lib entry file.
- [ ] `cargo check -p boltz-acpx` passes.
- [ ] `make check-all` passes (no cross-crate breakage).
- [ ] `cargo fmt --all -- --check` passes.

## Success Criteria

- `cargo check -p boltz-acpx` succeeds with zero warnings from the new code.
- `make check-all` succeeds (workspace-wide, confirms no dependency conflicts introduced by adding `agent-client-protocol`).
- Every file in `crates/acp/src/` is under 200 lines.
- No `#[cfg(target_os)]` outside `crates/acp/src/platform/`.

## Risk Assessment

- **Dependency version drift:** `agent-client-protocol` is externally maintained; pinning too loosely risks a breaking upstream change silently landing on `cargo update`. Mitigate: pin an exact version (`=x.y.z`) initially, bump deliberately.
- **Lib entry-file naming mistake:** if Phase 1 guesses wrong on `src/lib.rs` vs `src/acp.rs`, every subsequent phase's file paths in this plan are off by a rename. Mitigate: verify against 2+ sibling crates (`http_client`, `scheduler`, `theme`) before finalizing, do this first in Phase 1.
- **windows feature gaps:** `is_process_alive`'s Windows branch may need a `windows` crate feature not yet enabled in root `Cargo.toml`'s big feature list. Mitigate: check `Win32_System_Threading` is present (it is, per current root `Cargo.toml`) before assuming any other Win32 module is available.

## Security Considerations

- None yet — no untrusted input parsed in this phase (types/error/control scaffolding only). Establishes the `AcpError` shape that later phases (permissions, filesystem) rely on to avoid leaking internal error detail (e.g. raw OS errors) to the agent subprocess or UI without review — flag for Phase 3 review, not actionable here.

## Next steps

- Proceed to [Phase 2](./phase-02-protocol-transport-lifecycle.md) once `cargo check -p boltz-acpx` is green.
- Open question carried forward: **error-handling strategy** (`thiserror` enum vs `anyhow`) — this phase assumes `thiserror` for the public `AcpError` type per the plan's proposed default; get explicit user confirmation before Phase 2 builds on it, since reversing it later touches every phase.
