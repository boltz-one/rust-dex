# Goal: Port acpx (TS) → `crates/acp` (boltz-acpx)

## Mission
Port the Core+persistence+queueing slice of `others/acpx` (TS ACP CLI) into a new Rust crate `crates/acp`, embeddable by this GPUI app, following the 6-phase plan below. Done = all 6 phases implemented, each phase's Success Criteria passes, `make check-all` green.

## Context & Key Files
- Plan: `plans/20260705-1718-acpx-to-acp-crate-port/plan.md`
- Phases (implement in this order, Phase 5 may run parallel to 2-3): `phase-01-crate-scaffolding.md`, `phase-02-protocol-transport-lifecycle.md`, `phase-03-permissions-filesystem-terminal.md`, `phase-04-runtime-engine-public-contract.md`, `phase-05-session-persistence.md`, `phase-06-prompt-queueing-cancellation.md`
- Source to port: `others/acpx/src/{acp,runtime,session,types.ts,errors.ts,permissions*.ts,filesystem.ts,agent-registry.ts,mcp-servers.ts}`
- Target: `crates/acp/Cargo.toml` (currently 0 bytes, not in root `Cargo.toml` workspace members)
- Research: `research/researcher-01-rust-acp-ecosystem.md`, `research/researcher-02-acpx-architecture.md`

## Requirements
**Must do:**
- Register `crates/acp` (package `boltz-acpx`) in root `[workspace.members]`; follow `boltz-*` Cargo.toml conventions (see `crates/scheduler`, `crates/http_client`).
- ADR-1: reuse `agent-client-protocol`/`agent-client-protocol-schema` crates; hand-roll only `session/set_mode`, `session/set_config_option`, `terminal/release`.
- ADR-2/3: async substrate = `smol`, subprocess via `boltz-util::command`/`process` (already smol-based). No tokio, no new executor.
- ADR-4: per-session single-flight prompt queue (not global).
- ADR-5: persistence via serde `#[serde(default)]` + `#[serde(flatten)]` + enum schema-version tag.
- ADR-6: permission-request API is async/channel-based, never blocks the GPUI event loop.
- All new source files < 200 lines; real subprocess I/O in tests, no mocks.

**Must not:**
- Do not port `cli*`, `cli/` (incl. `cli/queue/` IPC daemon), `flows*`, `conformance/`, `examples/`, `scripts/`.
- Do not introduce tokio or any executor beyond `smol`.

## Success Criteria
- `cargo check -p boltz-acpx` and `make check-all` pass, zero warnings from new code.
- Phase 2: real subprocess integration test proves spawn→initialize→shutdown and SIGTERM→SIGKILL escalation.
- Phase 3: permission decision-tree tests cover all mode×policy combos; fs sandbox rejects path traversal; terminal kill verified via `is_process_alive`.
- Phase 4: simulated host consumer drives `ensure_session`→`start_turn`→event stream→result against the Phase-2 fake agent; reconnect-after-crash test passes.
- Phase 5: unknown-field round-trip, missing-optional-field default, suffix-id ambiguity, and 200-message conversation-trim tests all pass.
- Phase 6: two sessions' turns run concurrently; same-session turns serialize in order; queue-bound overflow returns a typed error, not a panic.

## Out of Scope
- CLI/commander surface, flows DSL + replay viewer, conformance harness, compare-command, config/status commands, cross-process IPC queue daemon.

## Open Decisions
7 unresolved questions in `plan.md` § "Unresolved Questions" (MCP scope, per-agent CLI variants, terminal streaming vs polling, legacy model-metadata window, error strategy, terminal manager scope, session state dir) — confirm with user before the affected phase; each phase file's "Next steps" states the fallback default if unanswered.

## Verification
```bash
cargo check -p boltz-acpx
cargo test -p boltz-acpx
cargo fmt --all -- --check
make check-all
```
