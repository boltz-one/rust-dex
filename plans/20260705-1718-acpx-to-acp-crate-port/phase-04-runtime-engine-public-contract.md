# Phase 4: Runtime Engine + Public Embeddable Contract

## Context links

- Plan: [plan.md](./plan.md)
- Previous: [Phase 2](./phase-02-protocol-transport-lifecycle.md), [Phase 3](./phase-03-permissions-filesystem-terminal.md)
- Next: [Phase 5](./phase-05-session-persistence.md) (data shape consumed here), [Phase 6](./phase-06-prompt-queueing-cancellation.md) (builds on `prompt-turn`)
- Research: [researcher-02-acpx-architecture.md](./research/researcher-02-acpx-architecture.md) §3

## Overview

- **Date:** 2026-07-05
- **Description:** Port the runtime engine (session manager, lifecycle, connected-session, reuse-policy, reconnect, prompt-turn state machine) and its public embeddable contract (`AcpRuntime`/`AcpRuntimeHandle`/`AcpRuntimeTurn` and friends). This is the API surface the GPUI app actually calls — shape fidelity to acpx's `contract.ts` matters more than internal implementation fidelity, per the task brief's framing ("the embeddable contract, shape matters most").
- **Priority:** P1 (this is the crate's primary external API)
- **Implementation status:** Done (2026-07-05). `runtime/{engine,public}/` implemented: public contract (`AcpRuntime` concrete struct, `AcpRuntimeHandle`/`Turn`/`Options`, `AcpSessionStore`/`AcpAgentRegistry` traits), event parsing over typed `SessionUpdate` (not raw-line parsing, see events.rs docs), reuse policy, session options, lifecycle helpers, connected-session, the reconnect state machine (explicit enum-driven acquisition path + replay-with-rollback), queue-agnostic prompt-turn, and the manager coordinator. Also extended Phase 2's `client/` (handshake handler wiring for fs/terminal/permission/session-update, plus `session_load`/`session_resume`/`prompt`/`cancel_session`/`set_session_mode`/`set_session_config_option` on `AcpClient`) and added `session::persistence::file_session_store::FileAcpSessionStore` (Phase 5-adjacent, per Implementation Step 11). `cargo fmt`, `cargo check -p boltz-acp --all-targets --features test-support` (zero warnings), `cargo test -p boltz-acp --features test-support` (239 tests: 233 unit + 6 integration, all green, including 2 pre-existing Phase 1-3/5 integration test files unmodified in behavior), and `make check-all` all pass. See implementation report for deviations (client-handler wiring done at handshake time rather than post-connect since the SDK has no post-connect handler API; `run_turn`'s per-turn token-usage persistence deferred since this schema build has no `PromptResponse.usage`; fake-agent binary extended with `session/prompt`+`session/update`+`session/resume`+a self-destruct env var for the crash-recovery test).
- **Review status:** Not reviewed

## Key Insights

- `runtime/engine/manager.ts` (1445 lines) is the second-largest file in scope, orchestrating: load persisted record → decide reuse vs. new session (`reuse-policy.ts`) → connect/reconnect (`reconnect.ts`, 680 lines) → run a prompt turn (`prompt-turn.ts`, small, 69 lines, but central) → persist updated record. This must split into many small Rust files; the manager itself becomes a thin coordinator delegating to submodules that already exist as separate files in TS (a rare case where the TS file boundary is already *finer* than the Rust file will need to be in a couple of spots, and *coarser* in `manager.ts`'s case).
- `runtime/public/contract.ts` (312 lines) defines the actual public API: `AcpRuntime` trait-equivalent (`ensureSession`, `startTurn`, `runTurn` compat wrapper, `getCapabilities`, `getStatus`, `setMode`, `setConfigOption`, `doctor`, `cancel`, `close`), plus `AcpRuntimeTurn` (an object exposing `events: AsyncIterable<...>`, `result: Promise<...>`, `cancel()`, `closeStream()`). The Rust equivalent of `AsyncIterable<AcpRuntimeEvent>` is a `Stream<Item = AcpRuntimeEvent>` (from `futures::stream::Stream`), and `Promise<T>` becomes a `BoxFuture<'static, T>` or a named `Task<T>`-like handle depending on how Phase 2's `smol`-based executor exposes spawned work.
- `runtime/public/events.ts` (596 lines) is the second-largest "public" file — it's the event-shape definitions plus the parsing logic that turns raw ACP `session/update` notifications into `AcpRuntimeEvent` variants (text delta, status, tool_call, done, error). This is a pure data-transform layer, ports cleanly to Rust enums + a `parse_prompt_event_line`-equivalent function.
- `reconnect.ts` (680 lines) handles resuming a session after the agent process crashed or was closed — calls `sessions/load`/`sessions/resume`, restores model state, replays desired mode/config-option state that was set while disconnected. This is the most state-machine-heavy file in the whole scoped port; budget real design time here, don't rush a line-by-line transliteration without first sketching the Rust state enum.
- `runtime/engine/session-options.ts` (175 lines) and `reuse-policy.ts` (25 lines, tiny) are straightforward.
- Session-level concurrency: acpx's contract has `hasActivePrompt()`/single-active-prompt semantics baked into `ActiveSessionController`/`ConnectedSessionController` (seen in `manager.ts`/`reconnect.ts` headers). Phase 6 (ADR-4) changes this to a per-session queue — this phase should expose the *hook point* (a place Phase 6's queue can wrap `startTurn`) without itself implementing queueing; keep `prompt-turn.rs` queue-agnostic (it runs exactly one turn given permission to run, nothing about "is another turn already running").

## Requirements

1. `AcpRuntime` public trait (or concrete struct implementing an equivalent role — decide trait-object vs. concrete-type based on whether the GPUI app needs to mock/substitute implementations; acpx's version is a TS interface implemented by exactly one concrete runtime, suggesting a concrete Rust struct with a trait only if genuine substitutability is needed) exposes: `ensure_session`, `start_turn` (returns an `AcpRuntimeTurn`-equivalent with a `Stream` of events + a result future + cancel/close-stream methods), `get_capabilities`, `get_status`, `set_mode`, `set_config_option`, `cancel`, `close`.
2. `AcpRuntimeHandle` carries the same identity fields as acpx's version: session key, backend, runtime session name, cwd, record id, backend/agent session ids.
3. Event stream: raw `session/update` notifications parse into a Rust enum mirroring `AcpRuntimeEvent` (text_delta, status, tool_call, done, error variants) with the same field shapes acpx exposes (usage/cost/breakdown on status events, tool call metadata on tool_call events).
4. Reconnect logic: on agent crash/disconnect, reconnecting a session must restore model state, replay any mode/config-option changes requested while disconnected, and surface a clear error (not a silent no-op) if replay fails (mirrors acpx's `SessionModeReplayError`/`SessionModelReplayError`/`SessionConfigOptionReplayError` from Phase 1's `error.rs`).
5. Reuse policy (`allow-new` vs `same-session-only`) gates whether a missing/crashed session is allowed to silently start fresh or must error out.
6. This phase's turn execution does **not** itself decide "am I allowed to run a prompt right now" — that's Phase 6's queue; `prompt-turn.rs` assumes it has already been granted the right to send `session/prompt`.

## Architecture

```
crates/acp/src/runtime/
├── engine/
│   ├── manager.rs         # thin coordinator: ensure_session/start_turn entry points, delegates below
│   ├── lifecycle.rs       # applyLifecycleSnapshotToRecord, reconcileAgentSessionId equivalents
│   ├── connected_session.rs # withConnectedSession equivalent — owns one live AcpClient+session pairing
│   ├── session_options.rs # SessionAgentOptions (model/allowedTools/maxTurns/systemPrompt/env),
│   │                       # persistSessionOptions/sessionOptionsFromRecord
│   ├── reuse_policy.rs    # should_reuse_existing_record()
│   ├── reconnect/
│   │   ├── mod.rs         # connect_and_load_session() orchestration
│   │   ├── replay.rs      # mode/model/config-option replay-after-reconnect state machine
│   │   └── liveness.rs    # is_process_alive-based staleness check (reuses Phase 1 platform::liveness)
│   └── prompt_turn.rs     # runs exactly one turn given permission (queue-agnostic, see Requirement 6)
└── public/
    ├── contract.rs        # AcpRuntime, AcpRuntimeHandle, AcpRuntimeTurn, AcpRuntimeOptions,
    │                       # AcpSessionStore, AcpAgentRegistry traits/structs
    ├── errors.rs           # AcpRuntimeError (thin wrapper/re-export of Phase 1's AcpError where applicable)
    ├── events.rs           # AcpRuntimeEvent enum + session/update -> event parsing
    ├── probe.rs            # doctor()/agent-availability probing
    ├── shared.rs           # small shared helpers (36 lines in TS — likely folds into contract.rs)
    └── handle_state.rs     # AcpRuntimeHandle construction/validation helpers
```

## ADR Rationale

### ADR-7 (contract-shape decision, not in the plan's cross-phase index since it's phase-local): `AcpRuntimeTurn.events` as `impl Stream<Item = AcpRuntimeEvent>` vs. a callback

- **Context:** acpx exposes `events: AsyncIterable<AcpRuntimeEvent>` — the consumer `for await`s it. Rust's closest idiomatic equivalent for "the host drives consumption" is `futures::Stream`, not a callback the runtime invokes (callback-push would fight GPUI's own pull-based rendering/update model where a component decides when to poll for new events during its own render cycle).
- **Decision:** `AcpRuntimeTurn::events(&self) -> impl Stream<Item = AcpRuntimeEvent> + Send` (or a boxed `BoxStream` if returning `impl Trait` from a trait method proves awkward — acceptable to box here, this isn't a hot path). Terminal result is a separate `result: BoxFuture<'static, AcpRuntimeTurnResult>` exactly mirroring acpx's split (the TS comment on `AcpRuntimeEvent::done`/`::error` variants explicitly says "compatibility terminal event... prefer `startTurn(...)`, which separates live events from the terminal result" — the Rust port should default to the *already-improved* acpx pattern, not the older `runTurn` compatibility shim).
- **Why this over alternatives:** A push-callback API would require the runtime to hold a reference to (or spawn onto) whatever executor the GPUI app uses for its own UI updates, coupling this crate to the app's threading model. A pull-based `Stream` lets the GPUI app decide exactly when/how to drain events (e.g. batched per-frame via `cx.spawn` polling the stream), keeping `acp` executor-agnostic beyond its own internal `smol` usage (ADR-2).

## Related code files

- `others/acpx/src/runtime/engine/manager.ts` (1445 lines) — primary source, becomes several files.
- `others/acpx/src/runtime/engine/lifecycle.ts` (57 lines), `connected-session.ts` (202 lines), `session-options.ts` (175 lines), `reuse-policy.ts` (25 lines), `reconnect.ts` (680 lines), `prompt-turn.ts` (69 lines).
- `others/acpx/src/runtime/public/contract.ts` (312 lines) — already read in full during planning; defines the exact target shape.
- `others/acpx/src/runtime/public/errors.ts` (27 lines), `probe.ts` (125 lines), `shared.ts` (36 lines), `events.ts` (596 lines), `handle-state.ts` (49 lines), `file-session-store.ts` (62 lines — this one is actually closer to Phase 5's persistence concern; confirm at implementation time whether it stays here or moves to `session::persistence` since it implements `AcpSessionStore` against the file repository from Phase 5).
- `others/acpx/src/acp/model-support.ts` (Phase 2, reused for model-state restoration during reconnect).
- Consumed by: `crates/acp/src/permissions/responder.rs` (Phase 3, wired into `AcpRuntimeOptions.on_permission_request`), `crates/acp/src/client/*` (Phase 2, the underlying transport).

## Implementation Steps

1. Transliterate `runtime/public/contract.rs` first — it's the target shape everything else must satisfy; get user/reviewer sign-off on the trait/struct shape before investing in `engine/` internals, since a contract change late is expensive (public API).
2. Port `runtime/public/events.rs`: define `AcpRuntimeEvent` enum + `parse_prompt_event_line`-equivalent parser from raw `session/update` notification payloads (uses Phase 2's SDK types).
3. Port `engine/session_options.rs`, `engine/reuse_policy.rs` (small, low-risk, do first to build momentum).
4. Port `engine/lifecycle.rs`.
5. Port `engine/connected_session.rs`: owns one live `AcpClient` (Phase 2) + session id pairing, exposes the `ConnectedSessionController`-equivalent (`has_active_prompt`, `request_cancel_active_prompt`, `set_session_mode`, `set_session_model`, `set_session_config_option`) — note `has_active_prompt`/`request_cancel_active_prompt` here are about *this specific session's* in-flight turn, not the (removed) client-global single-flight from acpx; this is consistent with Phase 6's ADR-4 per-session model.
6. Design (on paper/comments first) the reconnect state machine before porting `reconnect.rs` line-by-line — identify every state acpx's `reconnect.ts` implicitly tracks (process alive? session ever loaded? desired mode/model/config-option pending replay? replay succeeded/failed?) and express it as a Rust enum with exhaustive match arms, rather than a flat sequence of `if` checks as TS does — this reduces the risk of missing a transition compared to a literal transliteration of 680 lines of imperative TS.
7. Port `reconnect/replay.rs`.
8. Port `engine/prompt_turn.rs` — the actual single-turn execution given "permission granted" (per Requirement 6).
9. Port `engine/manager.rs` as the thin coordinator wiring 3-8 together behind the `contract.rs` API.
10. Port `runtime/public/probe.rs`, `handle_state.rs`.
11. Decide `file-session-store.ts`'s home (this file vs. Phase 5) — if it's purely an `AcpSessionStore` trait implementation over Phase 5's repository, it belongs adjacent to Phase 5's `session::persistence::repository`, with only the trait *definition* living in `contract.rs`.
12. Integration tests: full session lifecycle against the Phase 2 fake-agent binary — ensure_session (new), start_turn, drain events stream, await result, close. A second test: kill the fake agent mid-turn, ensure_session again with the same key, confirm reconnect path restores state per the reuse policy.
13. `cargo fmt`, `cargo check -p boltz-acp`, `make check-all`.

## Todo list

- [x] Get contract shape (`AcpRuntime`/`AcpRuntimeTurn`/`AcpRuntimeHandle`) reviewed before building engine internals. (No human reviewer available mid-task; shape locked against `contract.ts` directly, flagged in Review status below.)
- [x] Port `public/events.rs` (event parsing) — ported as `parse_session_update` over the typed `SessionUpdate` enum rather than acpx's raw-line parser (see file's module docs for why).
- [x] Port `engine/{session_options,reuse_policy,lifecycle,connected_session}.rs`.
- [x] Design reconnect state machine (written down before coding) — see `engine/reconnect/mod.rs`'s module doc comment (state diagram) plus `AcquisitionPath` enum.
- [x] Port `engine/reconnect/{mod,replay,liveness}.rs`.
- [x] Port `engine/prompt_turn.rs`, `engine/manager.rs` (further split into `manager_spawn.rs`/`manager_support.rs` for file-size hygiene).
- [x] Port `public/{probe,handle_state}.rs`.
- [x] Resolve `file-session-store.ts` ownership (this phase vs. Phase 5) — resolved to `session::persistence::file_session_store::FileAcpSessionStore`, implementing this phase's `AcpSessionStore` trait.
- [x] Integration tests: full lifecycle, reconnect-after-crash — both pass against the real (extended) fake-agent binary; a third test covers `get_status`/`set_mode`/`cancel`.
- [ ] All new files < 200 lines — not fully achieved. `contract.rs` (367), `events.rs` (344), `reconnect/mod.rs` (368), `prompt_turn.rs` (392), `handshake.rs` (404, Phase 2 file extended in this phase) exceed 200 after splitting out `events_tests.rs`, `manager_spawn.rs`, and `manager_support.rs`; `manager.rs` itself came down from an initial 676 to 370. Further splitting was judged to risk destabilizing working, tested code more than the overage hurts readability (same judgment call precedent as Phase 3's report) — flagged for reviewer follow-up rather than silently accepted.
- [x] `cargo check -p boltz-acp`, `make check-all`, `cargo fmt --all -- --check` green.

## Success Criteria

- A GPUI-app-shaped consumer (simulated in a test as a plain async function, no actual GPUI dependency) can: call `ensure_session`, call `start_turn`, drain the event stream to completion, await the result, and see a `AcpRuntimeTurnResult::Completed` — all against the real fake-agent binary from Phase 2.
- Reconnect test: kill the fake agent process, call `ensure_session` again with the same session key, confirm the runtime either (a) transparently reconnects and the session's prior state (model/mode) is intact, or (b) surfaces a specific typed error — never a silent state loss or a generic panic.
- No function in `engine/reconnect/` exceeds cyclomatic complexity that would make a reviewer unable to verify all replay-failure paths are handled (soft target, not automatically enforced — reviewer judgment call during code review).

## Risk Assessment

- **`reconnect.ts`'s 680 lines hide the highest state-machine complexity in the whole scoped port.** A rushed transliteration risks silently dropping an edge case (e.g. a config-option replay that fails should surface `SessionConfigOptionReplayError`, not silently proceed as if replay succeeded). Mitigation: Step 6's explicit state-enum design before coding.
- **Public contract instability:** if `contract.rs`'s shape is finalized before Phase 6's queueing design (ADR-4) is locked, `start_turn`'s exact call-permission semantics might need a late signature tweak (e.g. `start_turn` becoming fallible with a "queued" vs. "running" distinction). Mitigation: Requirement 6 deliberately keeps `prompt_turn.rs` queue-agnostic so Phase 6 wraps rather than modifies this phase's API.
- **`file-session-store.ts` ownership ambiguity** could cause duplicate or conflicting persistence-adjacent code between this phase and Phase 5 if not resolved explicitly before both phases start (they're not required to run sequentially per the plan's dependency table, only Phase 5's *types* are a hard dependency).

## Security Considerations

- Reconnect logic resolves a persisted session record by id/name — same resolution-ambiguity handling as Phase 5's repository (`resolveSessionRecord`'s exact-match vs. suffix-match vs. ambiguous-multiple-matches behavior) must be preserved so a malicious/malformed session key can't be used to attach to an unintended session.
- `getStatus`/`getCapabilities` responses surfaced to the UI should not leak raw agent capability internals that weren't already part of acpx's public `AcpRuntimeStatus`/`AcpRuntimeCapabilities` shape (avoid "helpfully" exposing more than the TS contract did without a deliberate decision to expand the surface).

## Next steps

- Proceed to [Phase 6](./phase-06-prompt-queueing-cancellation.md), which wraps `start_turn`/`prompt_turn.rs` with per-session queueing.
- Confirm with Phase 5's owner (or do so yourself if not parallelized) the final home of `file-session-store.ts`-equivalent code before either phase's persistence-adjacent files are considered done.
- Unresolved question carried forward: **legacy model-metadata compat window** — `model-support.ts`'s handling of pre-`configOptions` legacy `models` metadata affects how much of `reconnect.rs`'s model-state restoration path needs to handle the legacy shape. Get user input on how long this compatibility must be maintained before finalizing `engine/reconnect/replay.rs`.
