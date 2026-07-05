# Phase 2: Protocol/Transport + Client Lifecycle + Agent Command Resolution

## Context links

- Plan: [plan.md](./plan.md)
- Previous: [Phase 1](./phase-01-crate-scaffolding.md)
- Next: [Phase 3](./phase-03-permissions-filesystem-terminal.md), [Phase 4](./phase-04-runtime-engine-public-contract.md), [Phase 6](./phase-06-prompt-queueing-cancellation.md)
- Research: [researcher-01-rust-acp-ecosystem.md](./research/researcher-01-rust-acp-ecosystem.md) (Q1-Q3, async/subprocess trade-offs), [researcher-02-acpx-architecture.md](./research/researcher-02-acpx-architecture.md) (§1-2, §7)

## Overview

- **Date:** 2026-07-05
- **Description:** Port the JSON-RPC/ndjson transport, subprocess spawn/handshake/shutdown lifecycle, error normalization, and agent-command resolution (per-agent CLI arg building, model support, registry). This is the foundation every other phase's client interaction depends on.
- **Priority:** P1 (blocks Phase 3, 4, 6)
- **Implementation status:** Done. `cargo check -p boltz-acp` clean, `cargo test -p boltz-acp --features test-support` 73/73 pass (70 unit + 3 real-subprocess integration tests in `tests/client_lifecycle.rs` against a real compiled fake-agent binary at `tests/fixtures/fake_agent/main.rs`), `cargo fmt --all -- --check` clean for `crates/acp/`, `make check-all` passes workspace-wide. All files < 200 lines. Deviation from ADR-1's original text: on inspecting the vendored SDK, all 3 "gap" methods (`session/set_mode`, `session/set_config_option`, `terminal/release`) turned out to already be fully typed in `agent-client-protocol-schema` — `jsonrpc_gap.rs` is a thin pass-through, not a hand-rolled implementation (see its module doc for detail). Agent-specific coverage per the resolved open question: Claude/Cursor/Codex/Gemini/Copilot/Devin detection predicates ported; Gemini version-probing, Copilot `--help` probing, and Devin's Windsurf identity-spoofing runtime quirks deferred (noted in `agent_command/mod.rs` module doc).
- **Review status:** Not reviewed

## Key Insights

- `others/acpx/src/acp/client.ts` is 2023 lines — by far the largest single file in scope. It bundles handshake, session creation, prompt send, shutdown, and single-active-prompt state tracking. The Rust port must split this across ~6 files to respect the <200-line convention; state-tracking for prompt concurrency moves to Phase 6 per ADR-4, not ported 1:1 here.
- `others/acpx/src/acp/jsonrpc.ts` (message-shape type guards) and `jsonrpc-error.ts` (error code table) are **replaced**, not ported, by reusing `agent-client-protocol`'s own JSON-RPC 2.0 message types — hand-rolling parsers here would duplicate the SDK (violates DRY). Only the acpx-specific error-code-to-`AcpError` mapping table needs porting (already scaffolded in Phase 1's `error.rs`).
- `agent-client-protocol` (Rust SDK, crates.io, github.com/agentclientprotocol/rust-sdk) covers: `initialize`, `authenticate`, `session/new`, `session/prompt`, `session/cancel` (Agent trait) and `fs/read_text_file`, `fs/write_text_file`, `terminal/create`, `terminal/output`, `terminal/kill`, `request_permission` (Client trait). It does **not** yet expose `session/set_mode`, `session/set_config_option`, `terminal/release` as typed Rust methods (per researcher-01 Q2) — these three must be hand-rolled as raw JSON-RPC calls against the same connection object (see ADR-1).
- `crates/util/src/command.rs` + `crates/util/src/command/darwin.rs` + `crates/util/src/process.rs` already provide a cross-platform `Command`/`Child`/`Stdio` built on `smol::process`, with `#[cfg(target_os = "macos")]` routed to a `posix_spawn`-based implementation (avoiding `fork()` issues Zed hit historically) and POSIX process-group `SIGKILL` (`libc::killpg`) vs. a Windows TODO. This is the exact "idiomatic pattern to imitate" the task brief pointed at (alongside `http_client`'s wasm/non-wasm split) — reuse it, don't reimplement subprocess spawn/kill.
- `crates/gpui_linux` and `crates/util` both already depend on `smol` (workspace version `"2.0"`) for non-wasm targets. This means `smol` is already a de-facto part of this workspace's non-tokio async stack, stronger grounds than researcher-01's "Option B, harder to integrate" caveat suggested — see ADR-2.
- `others/acpx/src/acp/terminal-manager.ts` (Phase 3) and `others/acpx/src/spawn-command-options.ts` both need the command-line-building logic ported here (`buildSpawnCommandOptions`) — Phase 3 depends on this phase's `agent_command`/`spawn_options` module, so Phase 3 cannot start in parallel with Phase 2's second half.

## Requirements

1. Client can spawn an ACP agent subprocess (using `util::command`), perform ACP `initialize` handshake advertising `fs`/`terminal` capabilities, and return a typed handle.
2. ndjson framing over stdio: incremental line-buffered read (agent output is streamed, not atomic — a partial JSON object across two `read()` calls must not be dropped or double-parsed).
3. Shutdown sequence matches acpx: close stdio, SIGTERM with 1.5s grace, then SIGKILL with 1s grace (reuse `util`'s process-group kill primitive rather than re-deriving signal numbers).
4. Error normalization: agent-side JSON-RPC errors and process-exit-during-prompt conditions map onto `AcpError` variants from Phase 1, matching acpx's `error-normalization.ts`/`error-shapes.ts` classification (e.g. distinguishing "query closed before response" vs "resource not found" vs "generic RPC error").
5. Agent command resolution: given a named agent (`"claude"`, `"cursor"`, etc.) or a raw command string, resolve to `(program, args)` — port `agent-registry.ts` + `agent-command.ts` + `codex-compat.ts` + `model-support.ts`. Exact per-agent coverage is an open question (see Unresolved Questions) — implement the registry mechanism generically first, wire in agent-specific quirks incrementally.
6. The three RPC methods missing from the Rust SDK (`session/set_mode`, `session/set_config_option`, `terminal/release`) are callable through the same connection with full request/response typing (not stringly-typed `serde_json::Value` at call sites).
7. MCP server config (`mcp-servers.ts`) is threaded into the `session/new` request params — scope limited to passthrough (see Unresolved Questions #1).

## Architecture

```
crates/acp/src/
├── client/
│   ├── mod.rs           # pub struct AcpClient; builder; public spawn()/shutdown()/session_new()
│   ├── spawn.rs         # subprocess spawn via util::command::{Command, Stdio}; wait-for-spawn
│   ├── transport.rs     # ndjson read loop (smol::io::BufReader + AsyncBufReadExt::read_line)
│   │                    # + write loop; wraps agent_client_protocol::ClientSideConnection
│   ├── handshake.rs     # initialize request/response, capability negotiation
│   ├── shutdown.rs      # SIGTERM(1.5s)->SIGKILL(1s) grace sequence, exit-reason classification
│   └── state.rs         # agent_started_at, last_known_pid, last_agent_exit — NOT prompt-queue
│                         # state (that's Phase 6); mirrors acpx's non-queue client.ts fields only
├── jsonrpc_gap.rs        # hand-rolled: SetSessionModeRequest/Response,
│                         # SetSessionConfigOptionRequest/Response, ReleaseTerminalRequest/Response
│                         # + helper to send them over the shared connection
├── agent_command/
│   ├── mod.rs
│   ├── registry.rs       # ports agent-registry.ts — name -> command resolution
│   ├── command_args.rs   # ports agent-command.ts — per-agent CLI arg construction
│   ├── codex_compat.rs   # ports codex-compat.ts (20 lines, small)
│   ├── model_support.rs  # ports model-support.ts — configOptions vs legacy `models` metadata
│   └── spawn_options.rs  # ports spawn-command-options.ts — shared with Phase 3 terminal-manager
├── mcp_servers.rs         # ports mcp-servers.ts
├── auth_env.rs            # ports auth-env.ts
├── error_normalization.rs # ports error-normalization.ts
├── error_shapes.rs        # ports error-shapes.ts
├── session_control_errors.rs # ports session-control-errors.ts
├── agent_session_id.rs    # ports agent-session-id.ts (35 lines, trivial)
└── version.rs             # crate version constant (adapt, not port — no CLI to report)
```

## ADR Rationale

### ADR-1: Reuse `agent-client-protocol`; hand-roll the 3 missing RPCs locally

- **Context:** The official Rust ACP SDK (`agent-client-protocol` + `agent-client-protocol-schema`, github.com/agentclientprotocol/rust-sdk, used by Zed) covers the bulk of the protocol surface acpx needs, but researcher-01 found `session/set_mode`, `session/set_config_option`, and `terminal/release` are not exposed as typed Rust methods yet.
- **Decision:** Reuse the SDK for everything it covers (initialize, session/new, session/prompt, session/cancel, fs/*, terminal/create|output|kill, request_permission). For the 3 gaps, hand-roll typed request/response structs in `jsonrpc_gap.rs` and send them as raw JSON-RPC requests over the same `ClientSideConnection`/transport the SDK already manages (the SDK's connection object exposes a generic request-send primitive since it's built on jsonrpc-2.0 framing — confirm the exact extension point against the SDK's public API at implementation time; if the connection type doesn't expose a raw-send escape hatch, fall back to sending these 3 methods over our own `transport.rs` write-half directly, keyed by request id, and route their responses through the same read loop before they reach the SDK's dispatcher).
- **Why this over alternatives:** (a) Hand-rolling the entire protocol duplicates ~500-800 LOC of JSON-RPC framing/type definitions the SDK already gives us for free — violates DRY, and diverges from the reference Rust implementation Zed already ships. (b) Waiting on upstream to add the 3 methods blocks this port indefinitely with no committed timeline. (c) Contributing upstream first is the *better long-term* move but is not a blocking dependency — track as a stretch goal (open a PR against `agentclientprotocol/rust-sdk` once the hand-rolled version proves the wire shape works), not a Phase 2 requirement.

### ADR-2: Async substrate = `smol`, not bare `async-task`+`async-executor`, not `boltz-scheduler`

- **Context:** This workspace has zero tokio dependencies. Researcher-01 recommended `async-task`+`async-executor` (a lightweight, futures-compatible executor pair) as the primary fit, with `smol` as a heavier-but-still-viable alternative. Separately, `crates/scheduler` (`boltz-scheduler`) already defines a `Scheduler` trait used by GPUI's own foreground/background task dispatch, built on `async-task::Runnable`.
- **Decision:** Use `smol` directly as `acp`'s async substrate for subprocess I/O and its internal task spawning (`smol::spawn`, `smol::io::{BufReader, AsyncBufReadExt}`, `smol::Timer`). Do **not** build a bespoke `async-task`+`async-executor` pair, and do **not** make `acp`'s core engine depend on `boltz-scheduler::Scheduler`.
- **Why this over alternatives:**
  - *vs. bare async-task+async-executor (researcher-01's top pick):* `smol` already ships this exact combination internally (smol re-exports/builds on `async-task`, `async-io`, `async-executor`) plus a ready-made process module and stdio async I/O — hand-assembling the same primitives ourselves duplicates what `smol` already packages, and `smol` is *already a workspace dependency* (`crates/util`, `crates/gpui_linux`), unlike bare `async-executor` which isn't used anywhere in this codebase yet. Reusing an existing dependency over introducing a new one is the DRY-correct call here, reversing researcher-01's ranking now that direct evidence of in-repo smol usage is available (researcher-01 did not have `crates/util` in its search budget).
  - *vs. `boltz-scheduler::Scheduler`:* that trait is `AppContext`-bound (its `SessionId` type identifies a GPUI *window* session, an unrelated concept that would collide/confuse with ACP's own "session" vocabulary throughout this crate) and is designed for classifying UI-thread-adjacent foreground vs. background work inside a running GPUI `App`. Coupling `acp`'s core engine to it would make the crate untestable without a full GPUI `TestAppContext`, contradicting the requirement that the embeddable engine be usable standalone (e.g. spawning a real fake-agent subprocess in a plain `#[test]`, no GPUI window needed).
  - *vs. tokio:* explicitly excluded — zero tokio anywhere in this workspace, confirmed by direct grep.
- **GPUI integration:** the GPUI app (`crates/app`, not part of this port) bridges `acp`'s smol-driven futures into the UI by spawning them via `cx.background_executor().spawn(...)` (GPUI's own executor) wrapping the `acp` future, or by running `acp`'s smol tasks on smol's own global executor and communicating results back to GPUI's foreground executor via a channel (`futures::channel::mpsc` or the workspace's `flume`). Either bridging strategy is an app-layer decision outside this crate's scope — `acp` only needs to guarantee its own futures are `Send + 'static` so either bridge works.

### ADR-3: Subprocess spawn/kill reuses `boltz-util::command`/`process`, not a new implementation

- **Context:** acpx's `client-process.ts` implements spawn-readiness waiting, stdio requiring, and SIGTERM/SIGKILL grace-period kill. The task brief pointed at `http_client`'s wasm/non-wasm cfg split as the pattern to imitate for platform-specific code; direct inspection during planning found `crates/util/src/command.rs` (+ `command/darwin.rs`) and `crates/util/src/process.rs` already implement exactly this: a cross-platform `Command`/`Child`/`Stdio` wrapper on `smol::process`, macOS routed through a dedicated `posix_spawn`-based module avoiding a known `fork()`-related crash-reporter issue, POSIX kill via `libc::killpg` (process-group), Windows kill via `Child::kill()` with a `TODO` noting job-object cleanup is not yet implemented upstream in `util`.
- **Decision:** `acp`'s `client/spawn.rs` and `client/shutdown.rs` depend on `util` (`boltz-util`) and build directly on `util::command::{Command, Stdio}` / `util::process::Child` rather than writing new spawn/kill code. The SIGTERM(1.5s)→SIGKILL(1s) grace sequence from acpx is implemented as a thin wrapper in `shutdown.rs` calling `util`'s primitives with the acpx timing constants.
- **Why this over alternatives:** (a) Reimplementing subprocess spawn duplicates real, already-hardened platform-specific code (the darwin posix_spawn path exists because of a real historical bug class) — DRY violation and a security/reliability regression risk if reinvented naively. (b) `util`'s existing Windows kill is a known-incomplete `TODO` (no job-object semantics) — this port inherits that same gap rather than fixing it silently; flagged explicitly under Risk Assessment below so it isn't lost. (c) Depending on `util` is cheap: it's already a near-universal workspace dependency, adds no new external crate.

## Related code files

- `others/acpx/src/acp/client.ts` (2023 lines) — primary source, split across `client/*.rs`.
- `others/acpx/src/acp/client-process.ts` (286 lines) — spawn/kill helpers, superseded by `util::command`/`process` per ADR-3; port only the acpx-specific bits with no `util` equivalent (grace-period timing constants, `resolveAgentSessionCwd` WSL path translation — Windows/WSL edge case, flag as low-priority).
- `others/acpx/src/acp/jsonrpc.ts`, `jsonrpc-error.ts` — reference only; not ported (superseded by SDK types + `error.rs`'s code table from Phase 1).
- `others/acpx/src/acp/error-normalization.ts` (304 lines), `error-shapes.ts` (147 lines), `session-control-errors.ts` (63 lines).
- `others/acpx/src/acp/agent-command.ts` (383 lines), `others/acpx/src/agent-registry.ts` (299 lines), `others/acpx/src/acp/model-support.ts` (267 lines), `others/acpx/src/acp/codex-compat.ts` (20 lines).
- `others/acpx/src/spawn-command-options.ts` (183 lines), `others/acpx/src/mcp-servers.ts` (187 lines), `others/acpx/src/acp/auth-env.ts` (187 lines), `others/acpx/src/acp/agent-session-id.ts` (35 lines), `others/acpx/src/version.ts` (72 lines, adapt not port).
- `crates/util/src/command.rs`, `crates/util/src/command/darwin.rs`, `crates/util/src/process.rs` — reuse targets for ADR-3.
- `crates/http_client/Cargo.toml`, `crates/http_client/src/http_client.rs` — wasm/non-wasm cfg-split pattern reference.
- `others/acpx/test/mock-agent.ts` — behavior reference for this phase's fake-agent test binary (see Implementation Steps).

## Implementation Steps

1. Add `util` as a dependency of `crates/acp/Cargo.toml` (`util.workspace = true`, non-wasm target block per ADR-3).
2. Add `agent-client-protocol`, `agent-client-protocol-schema` as dependencies (already added to workspace deps in Phase 1).
3. Read the SDK's `Agent`/`Client` trait docs (docs.rs) to confirm exact method signatures before writing `client/mod.rs` — do not guess signatures from researcher-01's summary alone.
4. Write `client/spawn.rs`: build a `util::command::Command` from resolved `(program, args, cwd, env)`, spawn with piped stdin/stdout/stderr, wrap the `util::process::Child`'s stdio handles for the transport layer.
5. Write `client/transport.rs`: wrap child stdout in `smol::io::BufReader`, read newline-delimited JSON via `AsyncBufReadExt::read_line` in a loop, parse each line as an SDK message type, dispatch. Write side: serialize + append `\n` + write to child stdin, flush.
6. Write `client/handshake.rs`: send `initialize` with `fs`/`terminal` capability flags, capture agent capabilities from response (`loadSession`, `resumeSession`, `closeSession`, `listSessions`).
7. Write `client/shutdown.rs`: close stdin, send SIGTERM via `util::process::Child::kill` variant (confirm `util` exposes a signal-specific kill, not just SIGKILL — if it only exposes `killpg(SIGKILL)`, extend `util` or send `libc::kill(pid, SIGTERM)` directly here with a comment explaining why this one signal isn't routed through `util`), wait up to 1.5s via `control::with_timeout`, escalate to `util`'s SIGKILL path, wait up to 1s, record exit reason.
8. Write `client/state.rs`: `agent_started_at: DateTime<Utc>`, `last_known_pid: Option<u32>`, `last_agent_exit: Option<AgentExitInfo>` struct — no prompt-queue fields (Phase 6 owns those).
9. Write `jsonrpc_gap.rs`: define `SetSessionModeRequest { session_id: String, mode_id: String }` etc. (mirror the ACP schema's naming/casing conventions exactly — check `agent-client-protocol-schema` docs for the wire format of adjacent methods to match casing/field-naming style), implement send via the transport's raw-request escape hatch (or fallback per ADR-1).
10. Port `agent_command/registry.rs`, `command_args.rs`, `codex_compat.rs`, `model_support.rs`, `spawn_options.rs`, `mcp_servers.rs`, `auth_env.rs`, `error_normalization.rs`, `error_shapes.rs`, `session_control_errors.rs`, `agent_session_id.rs` — direct line-by-line translation is appropriate here (these are largely pure functions / data transforms), keep each file under 200 lines by splitting where the TS source already exceeds it (`agent-command.ts` 383 lines → likely 2 Rust files).
11. Write a real fake-ACP-agent test binary (`crates/acp/tests/fixtures/fake_agent/main.rs`, wired as a `[[bin]]` in `Cargo.toml` gated by `test-support` feature or a `[dev-dependencies]`-only test harness crate) that speaks minimal ndjson JSON-RPC over stdio — behavior modeled on `others/acpx/test/mock-agent.ts`. No mocks: integration tests spawn this real compiled binary via `env!("CARGO_BIN_EXE_...")`.
12. Integration tests: spawn fake agent, complete `initialize` handshake, verify capability negotiation, verify shutdown grace sequence (kill fake agent's ability to respond, confirm SIGKILL fires after grace period), verify error normalization for at least one "agent exited mid-prompt" scenario.
13. `cargo fmt`, `cargo check -p boltz-acp`, `make check-all`.

## Todo list

- [ ] Confirm `agent-client-protocol` SDK's exact trait method signatures against docs.rs before coding.
- [ ] Confirm whether the SDK's connection type exposes a raw-request escape hatch for ADR-1's 3 gap methods; document the actual approach taken (escape hatch vs. parallel write-half) in this file once known.
- [ ] Confirm `util::process`/`util::command` exposes (or can be cheaply extended with) a SIGTERM-specific kill, not just SIGKILL.
- [ ] Port `client/{spawn,transport,handshake,shutdown,state}.rs`.
- [ ] Port `jsonrpc_gap.rs`.
- [ ] Port `agent_command/*.rs`, `mcp_servers.rs`, `auth_env.rs`, `error_normalization.rs`, `error_shapes.rs`, `session_control_errors.rs`, `agent_session_id.rs`, `version.rs`.
- [ ] Write fake-agent test binary.
- [ ] Integration tests: handshake, shutdown grace sequence, error normalization.
- [ ] All new files < 200 lines.
- [ ] `cargo check -p boltz-acp`, `make check-all`, `cargo fmt --all -- --check` green.

## Success Criteria

- Fake-agent integration test proves: spawn → initialize handshake → capability negotiation → clean shutdown, using only real subprocess I/O (no mocks).
- A second integration test proves the SIGTERM→SIGKILL grace sequence actually escalates (fake agent ignores SIGTERM, test confirms SIGKILL is sent after the grace window and the process is confirmed dead).
- `session/set_mode`, `session/set_config_option`, `terminal/release` are callable with typed request/response structs (even if backed by the ADR-1 fallback path).
- Agent-command registry resolves at least the agents explicitly named in the acpx source without hardcoded per-agent logic leaking outside `agent_command/`.

## Risk Assessment

- **Inherited Windows kill gap:** `util::process::Child::kill` on Windows has no job-object cleanup (upstream `TODO`). Grandchildren of the agent process may survive a kill on Windows. This is inherited risk, not introduced by this port — document it in the crate's rustdoc so GPUI app authors know the limitation exists.
- **SDK API surface mismatch:** if the actual `agent-client-protocol` Rust SDK's method signatures differ meaningfully from researcher-01's summary (docs.rs page may have moved/changed since research), Step 3 (verify against docs.rs before coding) is the mitigation — do not skip it.
- **ndjson framing edge cases:** partial-line reads across multiple `poll()` calls, or an agent that emits non-JSON diagnostic lines on stdout instead of stderr, could desync the parser. Mitigation: buffer until a full line is available before parsing (matches acpx's `TextDecoder` buffering behavior); log and skip (don't crash on) unparseable lines, matching acpx's tolerant `isAcpJsonRpcMessage` guard behavior.
- **Cross-phase coupling:** Phase 3's terminal-manager needs `agent_command::spawn_options` — Phase 3 cannot fully start until this phase's `spawn_options.rs` lands, even though other Phase 3 files (permissions) have no such dependency.

## Security Considerations

- Command-line construction (`agent_command::command_args`) must not allow injection via untrusted config values (e.g. a session's persisted `agentCommand` string) — port acpx's `splitCommandLine` quoting/escaping logic faithfully rather than doing naive string splitting, since a malformed split could let arguments swallow flags they shouldn't.
- Auth credentials (`auth-env.ts` → `auth_env.rs`) are passed as subprocess environment variables — ensure they are never logged (check any `Debug`/`log::debug!` call sites near env construction) and are scoped to the child process only, not inherited further than necessary.
- Error messages surfaced to the UI (via `AcpError`) must not leak raw file paths or environment variable values from auth/env contexts beyond what acpx already exposes — audit `error_normalization.rs` port for parity.

## Next steps

- Proceed to [Phase 3](./phase-03-permissions-filesystem-terminal.md) (needs `agent_command::spawn_options`) and [Phase 6](./phase-06-prompt-queueing-cancellation.md) planning (needs the client transport shape finalized) once this phase's client lifecycle is stable.
- Unresolved question carried forward: **MCP server integration scope** — passthrough-only vs. local hosting/brokering. Get user input before finalizing `mcp_servers.rs`'s public shape, since a passthrough-only design is a much smaller surface than a hosting design.
- Unresolved question carried forward: **which agent-specific CLI variants ship first** (Claude/Cursor/Codex confirmed relevant since acpx has dedicated compat files; Gemini/Copilot/Devin coverage unclear) — get user input before Step 10.
