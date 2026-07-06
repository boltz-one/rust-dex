---
title: "Port acpx (TS) core + persistence + queueing into crates/acp"
description: "Port scoped ACP client/runtime/session-persistence logic from others/acpx into a new boltz-acpx Rust crate, embeddable in the GPUI desktop app."
status: done
priority: P1
effort: 53h
branch: main
lane: high-risk
tags: [acp, agent-client-protocol, rust-port, session-persistence, gpui]
created: 2026-07-05
---

# Plan: Port acpx → `crates/acp` (boltz-acpx)

## Scope

Port the **Core + session persistence + queueing** slice of `others/acpx` (TS) into a new
Rust crate `crates/acp` (published as `boltz-acpx`), embeddable by the GPUI desktop app.
Full scope/out-of-scope list, target-crate facts, and required ADR topics are in the
orchestrator brief this plan was generated from — repeated at the top of each phase file's
Context section. Out of scope: `cli*`, `cli/`, `flows*`, `conformance/`, `examples/`, `scripts/`.

**Research inputs (read in full before touching any phase file):**
- `research/researcher-01-rust-acp-ecosystem.md` — Rust ACP ecosystem, async/subprocess options.
- `research/researcher-02-acpx-architecture.md` — acpx TS architecture deep-dive.

**Locked-in override (do not re-litigate):** no tokio in this workspace. Async substrate is
`smol` (already a transitive workspace dep via `boltz-util`/`gpui_linux`), not
`async-task`+`async-executor` from scratch and not `boltz-scheduler` (that trait is
GPUI-`AppContext`-bound). See Phase 2 ADR-2/ADR-3.

## Phases

| # | Phase | Status | File-ownership (new dirs) | Depends on |
|---|-------|--------|----------------------------|------------|
| 1 | Crate scaffolding + workspace registration + deps | done | `crates/acp/{Cargo.toml,src/acp.rs,error.rs,types.rs,control.rs,platform/}` | — |
| 2 | Protocol/transport + client lifecycle + agent-command | done | `crates/acp/src/{client/,agent_command/,jsonrpc_gap.rs,mcp_servers.rs,auth_env.rs,error_normalization.rs,error_shapes.rs,session_control_errors.rs,agent_session_id.rs,version.rs}` | 1 |
| 3 | Permissions + filesystem + terminal-manager | done | `crates/acp/src/{permissions/,filesystem.rs,terminal/}` | 2 (needs `spawn_options`, `client` transport) |
| 4 | Runtime engine + public embeddable contract | done | `crates/acp/src/runtime/{engine/,public/}` (also extended `crates/acp/src/client/{mod,handshake}.rs` + `handlers.rs`, and added `crates/acp/src/session/persistence/file_session_store.rs`) | 2, 3 |
| 5 | Session persistence (versioned serde format) | done | `crates/acp/src/session/{*.rs,persistence/}` | 1 (types only — can run parallel to 2/3) |
| 6 | In-process prompt queueing + cancellation | done | `crates/acp/src/queue/`, `crates/acp/src/perf_metrics.rs` (stretch, deferred) | 2, 4 |

Phase 5 only needs Phase 1's `types.rs`/`error.rs` skeletons — it can be implemented in
parallel with Phases 2-3 by a different engineer/agent. Phase 6 needs Phase 4's turn
abstraction to exist first.

- [Phase 1: Crate Scaffolding](./phase-01-crate-scaffolding.md)
- [Phase 2: Protocol, Transport & Client Lifecycle](./phase-02-protocol-transport-lifecycle.md)
- [Phase 3: Permissions, Filesystem & Terminal](./phase-03-permissions-filesystem-terminal.md)
- [Phase 4: Runtime Engine & Public Contract](./phase-04-runtime-engine-public-contract.md)
- [Phase 5: Session Persistence](./phase-05-session-persistence.md)
- [Phase 6: Prompt Queueing & Cancellation](./phase-06-prompt-queueing-cancellation.md)

## Cross-Phase ADR Index

| # | Decision | Resolved in |
|---|---|---|
| ADR-1 | Reuse `agent-client-protocol` crate; hand-roll the 3 missing RPCs locally | Phase 2 |
| ADR-2 | Async substrate = `smol`, not async-task/async-executor, not `boltz-scheduler` | Phase 2 |
| ADR-3 | Subprocess spawn/kill reuses `boltz-util::command`/`process` (already smol-based, already isolates macOS posix_spawn quirk) | Phase 2 |
| ADR-4 | Prompt queueing = per-session single-flight queue, not per-client global single-flight | Phase 6 |
| ADR-5 | Session persistence format: serde + schema-tag + `#[serde(flatten)]` catch-all for forward-compat | Phase 5 |
| ADR-6 | Permission-request API is async/channel-based, never blocks GPUI event loop | Phase 3 |

## Unresolved Questions Needing User Input (before implementation starts)

1. **MCP server integration scope** — is `mcpServers` passthrough-only (forward config to agent's `session/new`), or does the Rust port need to host/broker MCP servers itself? (Phase 2)
2. **Agent-specific CLI arg variants** — acpx's `agent-command.ts`/`agent-registry.ts` special-case Claude, Cursor, Codex, Gemini, Copilot, Devin. Which of these ship in v1 vs. deferred? (Phase 2)
3. **Terminal output streaming vs polling** — does the GPUI UI need live `terminal/output` streaming during a running command, or is poll-after-completion acceptable for v1? (Phase 3) — **Resolved for Phase 3's implementation:** shipped poll-style (`OutputSnapshot` return type keeps the door open for streaming later); confirm with user before Phase 4/6 lock in the public contract.
4. **Legacy model-metadata compat window** — how long must pre-`configOptions` legacy `models` metadata be supported? (Phase 2/4)
5. **Error-handling strategy** — `thiserror` enums for public API + `anyhow` only at host boundary (proposed default, matches `http_client`'s `pub use anyhow::Result` pattern) — confirm or override. (All phases)
6. **Terminal manager scope** — is full descendant-process-group tracking (884-line TS file) required for v1, or can the port start with kill-of-direct-child-only? (Phase 3) — **Resolved:** not required. `util::process::Child::spawn` (ADR-3) already starts every terminal command as its own POSIX session leader, so a single `killpg` on its pid already reaps its whole process-group tree without separate PID-tree tracking; see `terminal::tracking` module docs.
7. **Session state directory** — `~/.acpx/sessions` equivalent location/name for the Rust port (e.g. platform-appropriate `dirs::state_dir()` vs. app-provided path)? (Phase 5)

Each is repeated in its owning phase file's Overview/Next-steps section.
