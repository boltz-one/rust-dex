# Phase 6: In-Process Prompt Queueing + Cancellation

## Context links

- Plan: [plan.md](./plan.md)
- Depends on: [Phase 2](./phase-02-protocol-transport-lifecycle.md) (client transport), [Phase 4](./phase-04-runtime-engine-public-contract.md) (`prompt_turn`, `AcpRuntimeTurn` contract)
- Research: [researcher-02-acpx-architecture.md](./research/researcher-02-acpx-architecture.md) §6, Trade-offs

## Overview

- **Date:** 2026-07-05
- **Description:** Design and implement the Rust port's prompt concurrency model. This is explicitly **not** a 1:1 port — acpx's cross-process IPC lease/owner-daemon queue (`cli/queue/*`) is out of scope entirely, and even acpx's in-process behavior (single-active-prompt-per-client, not a real multi-item queue) is not the right fit for a GUI that may run several concurrent agent sessions in different tabs/panels. Also includes `perf-metrics.ts`/`perf-metrics-capture.ts` as an optional stretch goal.
- **Priority:** P2 (functionally important, but Phases 2-5 can all be verified independently first with single-turn tests; this phase is what makes concurrent multi-session usage safe)
- **Implementation status:** Done. `crates/acp/src/queue/{mod,session_queue,dispatcher}.rs` implement the per-session bounded FIFO + single-flight slot (ADR-4); `AcpRuntime::start_turn`/`run_turn` are now `async` and enqueue through `ConnectedSession::prompt_queue` instead of calling `prompt_turn::start_turn` directly (the sanctioned Phase 4 adjustment flagged in this file's Risk Assessment). `session/update` ordering (Requirement 4) is enforced via a new `ConnectedSession::update_order` lock, kept strictly separate from the queue's own lock per the Risk Assessment's deadlock warning. Bounded-queue capacity defaults (`DEFAULT_QUEUE_CAPACITY = 4`) and the cancel-vs-clear-queue split (`cancel` leaves the queue intact; `clear_queue`/`cancel_active_and_clear` are the explicit opt-ins) were **not** re-confirmed with the user before implementation (none was available) — proceeded with this file's own recommended defaults, as authorized by the task brief. Stretch goal (`perf_metrics.rs`, feature-gated) was **deferred**, not attempted, to keep scope on the core queueing/cancellation correctness and its test coverage. All 6 phases of the plan are now implementation-complete.
- **Review status:** Not reviewed

## Key Insights

- Per researcher-02's architecture read of `client.ts`: acpx's "queue" is **not actually a queue** — `activePrompt` is a single field holding at most one in-flight prompt promise per `AcpClient` instance, and `hasActivePrompt(sessionId?)` just checks whether that one slot is occupied. A second prompt request while one is active is *rejected*, not enqueued, in the base client. The only thing resembling ordering is the `sessionUpdateChain` — a promise chain ensuring `session/update` notifications are processed in the order they arrive, independent of prompt concurrency.
- acpx's CLI layer builds actual cross-process queueing (`cli/queue/ipc-server.ts`, `lease-store.ts`, `owner-turn-controller.ts`) on top of this single-active-prompt client because the CLI's use case is "many separate `acpx` process invocations against one long-lived owner daemon" — a problem this GPUI port does not have (single long-running GUI process owns the runtime directly, confirmed out of scope by the task brief).
- The GUI's actual concurrency need is different from both acpx layers: **one GPUI process, potentially many concurrent ACP sessions** (e.g. multiple chat tabs each talking to their own agent subprocess or their own session on a shared agent subprocess), where each individual *session* should still behave like acpx's client — one active prompt at a time, request queued or rejected — but unrelated sessions must not block each other.
- Phase 4's `connected_session.rs` already exposes `has_active_prompt`/`request_cancel_active_prompt` scoped to one session (not client-global) per that phase's design note — this phase builds the actual queue on top of that per-session scope.

## Requirements

1. Each ACP session has its own single-flight prompt slot plus a bounded FIFO queue of pending prompt requests (bounded to prevent unbounded memory growth if a UI bug fires prompts faster than they can be consumed — pick a small bound, e.g. matching what a reasonable UI would ever need queued, such as low single digits, and reject/backpressure beyond it rather than silently growing forever).
2. Sessions are independent: a slow/stuck prompt on session A must not delay session B's prompt from starting.
3. Cancellation targets the *active* prompt on a given session (matches acpx's per-session cancel semantics) — cancelling a session does not affect its queued-but-not-yet-started prompts unless explicitly requested to also clear the queue (decide and document which).
4. `session/update` notification processing remains strictly ordered per session (port the `sessionUpdateChain` guarantee at the session level, not globally) — this matters for correctness of transcript/event ordering shown in the UI even though it's orthogonal to prompt queueing itself.
5. The queue implementation must not depend on `boltz-scheduler` (ADR-2, Phase 2) — it's built on the same `smol`-based primitives as the rest of the crate (e.g. a `smol`-compatible mpsc channel or an internal `VecDeque` guarded by `parking_lot::Mutex` with a waiter list).

## Architecture

```
crates/acp/src/queue/
├── mod.rs             # SessionPromptQueue public API: enqueue(turn_request) -> AcpRuntimeTurn-compatible
│                       # handle, cancel_active(), queue_len()
├── session_queue.rs    # per-session FIFO + single-flight slot implementation
└── dispatcher.rs        # drains one session's queue, invokes Phase 4's prompt_turn::run_turn,
                          # re-arms the slot on completion, wakes the next queued request

crates/acp/src/perf_metrics.rs   # stretch goal, feature-gated: cargo feature "perf-metrics"
```

## ADR Rationale

### ADR-4: Prompt queueing model — per-session single-flight queue, not per-client global single-flight

- **Context:** acpx's base client has one active-prompt slot per `AcpClient` instance (effectively per-agent-process, since one client = one spawned agent subprocess in the CLI's model). The task's GUI host may run multiple agent sessions concurrently — either multiple subprocesses (one per session) or, if the port later supports it, multiple ACP sessions multiplexed over one agent subprocess connection (some agents support `session/new` multiple times over one connection). Either way, a single client-global "one prompt at a time, full stop" slot would serialize unrelated work a GUI user would reasonably expect to run in parallel (e.g. two chat tabs, two different agents).
- **Decision:** Move the single-flight slot from "per client" to "per session" (`SessionPromptQueue`, keyed by session id, each with its own slot + bounded FIFO). A prompt request for session A never waits on session B's queue. Within one session, ordering and single-flight semantics are preserved exactly as acpx does it (matches Requirement 1-2's mirroring of acpx's real guarantee), because ACP agents generally still expect one active `session/prompt` at a time *per session* even if the underlying transport could theoretically pipeline.
- **Why this over alternatives:**
  - *vs. keeping acpx's exact per-client behavior:* would mean opening a chat tab against agent X while another tab is mid-turn on agent X blocks the second tab entirely, even though the user has no mental model of "these two tabs share a queue" — bad UX for the stated use case (a GUI, not a scripted CLI pipeline where serialization was an intentional simplicity trade-off).
  - *vs. a single global queue across all sessions/clients:* strictly worse than per-client for a multi-session GUI; also loses the *only* good reason acpx has for its current design (natural backpressure against one script hammering one CLI invocation), which doesn't apply to a GUI issuing user-paced requests.
  - *vs. no queueing at all (reject concurrent prompts on the same session outright, no FIFO):* simpler, but a real GUI can plausibly fire two `startTurn` calls in quick succession for the same session (e.g. a fast double-submit, or a "regenerate" action issued before the prior turn's UI fully settles) — a small bounded FIFO absorbs this without the caller needing its own debounce logic, at low implementation cost given Phase 4 already scoped `has_active_prompt` per-session.

## Related code files

- `others/acpx/src/acp/client.ts` (lines ~225-250, ~easier to re-read the specific `activePrompt`/`hasActivePrompt`/`sessionUpdateChain` section during implementation rather than the whole 2023-line file again) — behavior this phase's per-session slot must match at the single-session granularity.
- `others/acpx/src/perf-metrics.ts` (88 lines), `perf-metrics-capture.ts` (130 lines) — stretch goal source.
- Explicitly **not** ported (reference only, confirms scope boundary): `others/acpx/src/cli/queue/{ipc-server,ipc-transport,ipc,lease-store,messages,owner-env,owner-turn-controller,paths}.ts`, `others/acpx/src/cli/session/{queue-owner-process,queue-owner-runtime}.ts`.
- Consumes: `crates/acp/src/runtime/engine/prompt_turn.rs`, `connected_session.rs` (Phase 4).

## Implementation Steps

1. Re-read `client.ts`'s `activePrompt`/`hasActivePrompt`/`sessionUpdateChain` section (not the whole file) to confirm the exact single-flight semantics being mirrored at session granularity (e.g. does a rejected concurrent prompt get a specific error code the UI should distinguish from other failures? Port that distinction.).
2. Design `SessionPromptQueue`'s state machine: `Idle -> Running(current_turn) -> Idle` with a `VecDeque<PendingPromptRequest>` consulted on transition back to `Idle`. Use `parking_lot::Mutex<QueueState>` for the shared state, `futures::channel::oneshot` per enqueued request to signal "your turn has started" back to the caller (so `AcpRuntimeTurn`'s handle can be returned to the caller only once the turn is actually running, matching acpx's synchronous-looking `startTurn` return shape).
3. Implement the bounded-FIFO backpressure: define the bound as a `SessionPromptQueue::with_capacity(n)` constructor parameter (not a hardcoded constant), so the GPUI app can tune it; document a sane default.
4. Implement `dispatcher.rs`: pulls the next request when the slot frees, calls Phase 4's `prompt_turn::run_turn`, re-arms on completion (success, error, or cancellation all count as "slot free").
5. Implement session-scoped `session/update` ordering: a per-session `smol`-compatible sequential task/actor (or a `Mutex`-guarded "apply update" critical section awaited in arrival order) — confirm this doesn't need to be more elaborate than a simple `Mutex` around the update-application function, since the actual work is likely just "append to conversation model / forward to event stream," not itself long-running.
6. Implement cancellation: `cancel_active(session_id)` targets only the running turn; a separate explicit `clear_queue(session_id)` (or a `cancel_active_and_clear` combined call) is offered for the "also drop pending queued requests" case — Requirement 3 says decide and document which is the default; recommend `cancel_active` alone does NOT clear the queue by default (matches least-surprise: cancelling "this" turn shouldn't silently discard a user's next already-submitted message), with the combined call available for an explicit "stop everything" UI action.
7. Wire `SessionPromptQueue` into Phase 4's `AcpRuntime::start_turn` as the actual entry point (Phase 4's `prompt_turn::run_turn` becomes an implementation detail the dispatcher calls, not something external callers invoke directly anymore — may require a small Phase 4 API adjustment; flag this explicitly to whoever reviews Phase 4 if it's already "done" by the time this phase starts).
8. (Stretch) Port `perf_metrics.rs`/`perf_metrics_capture.rs` behind a `perf-metrics` Cargo feature, off by default — counters/timings/gauges snapshot matching acpx's `PerfMetricsSnapshot` shape, useful for later profiling but not required for correctness.
9. Integration tests against the Phase 2 fake-agent binary: (a) two sessions' turns run concurrently and neither waits on the other (measure via timing or explicit synchronization points in the fake agent), (b) a second prompt on the same session while one is active gets queued and runs after the first completes, in submission order, (c) exceeding the bounded queue capacity on one session produces the documented backpressure error without affecting other sessions, (d) cancelling an active turn does not drop already-queued pending requests (per Step 6's default), (e) `session/update` notifications for one session are applied in arrival order even if two updates race at the transport layer.
10. `cargo fmt`, `cargo check -p boltz-acp`, `make check-all`.

## Todo list

- [x] Re-confirm exact acpx single-flight/session-update-chain semantics from `client.ts`.
- [x] Design `SessionPromptQueue` state machine (written down before coding).
- [x] Implement `queue/{mod,session_queue,dispatcher}.rs`.
- [x] Implement session-scoped `session/update` ordering.
- [x] Implement cancellation (`cancel_active` vs. `clear_queue` distinction, default behavior documented).
- [x] Wire into Phase 4's `AcpRuntime::start_turn`; flag any resulting Phase 4 API change.
- [ ] (Stretch, feature-gated) Port `perf_metrics.rs`/`perf_metrics_capture.rs`. — Deferred; not attempted.
- [x] Integration tests: cross-session concurrency, same-session FIFO ordering, backpressure, cancel-vs-queue independence, update ordering.
- [x] All new files < 200 lines (`queue/mod.rs` 162, `queue/session_queue.rs` 192, `queue/dispatcher.rs` 88). `manager.rs` (419) and `prompt_turn.rs` (417) grew past 200 lines from the small sanctioned additions on top of Phase 4's already-oversized files (370/393 lines before this phase) — not split further, consistent with Phases 3/4's own documented judgment calls.
- [x] `cargo check -p boltz-acp`, `make check-all`, `cargo fmt --all -- --check` green.

## Success Criteria

- Test proves two different sessions' `start_turn` calls both begin executing without either waiting for the other (not just "both eventually complete" — actually concurrent).
- Test proves a second `start_turn` on the same session, issued while the first is active, does not start until the first completes, and runs in submission order relative to any further queued requests.
- Test proves exceeding the configured queue bound on one session returns a specific, documented error/result variant (not a panic, not silent dropping) and does not affect a concurrently-running different session.
- Test proves cancelling an active turn leaves queued-but-not-started requests for that session intact, per the documented default from Step 6.

## Risk Assessment

- **Phase 4 API churn:** if Phase 4 ships with `prompt_turn::run_turn` as the only entry point and this phase needs to insert a queue in front of it, there's a real chance the public `AcpRuntime::start_turn` signature needs a follow-up change (e.g. returning a "queued" vs "started immediately" indicator) after Phase 4 was already reviewed as "done." Mitigate by flagging this dependency loudly in Phase 4's own Risk Assessment (already done) and, if possible, sequencing Phase 6's contract review jointly with Phase 4 rather than fully sequentially.
- **Deadlock risk between the per-session update-ordering lock and the prompt-queue lock:** if `dispatcher.rs` and the update-ordering mechanism from Step 5 ever need to hold both locks at once in different orders on different code paths, a deadlock is possible. Mitigate by keeping the two concerns' locks strictly separate and never acquiring one while holding the other (document this invariant directly in the code, not just this plan).
- **Bounded-queue starvation:** if the bound is too small for legitimate rapid-fire UI usage (e.g. a "stop and regenerate" flow that itself issues 2 requests in quick succession), users could see spurious backpressure errors. Mitigate with a documented, sensibly-sized default and a configurable override (Step 3).

## Security Considerations

- No new untrusted-input surface is introduced by queueing itself (it operates on already-validated internal requests from Phase 4/GPUI, not raw agent/network input) — the main consideration is resource exhaustion: the bounded FIFO (Requirement 1) is itself the mitigation against a UI or agent misbehavior causing unbounded memory growth.

## Next steps

- This is the last phase in the plan's suggested ordering. Once complete, the crate's scoped feature set (per the task brief's "Core + session persistence + queueing" selection) is done; `docs/decisions/NNNN-*.md` files should be written for ADR-1 through ADR-6 per `development-rules.md`'s ADR Boundary rule before merging, since these are architecture/API-shape decisions, not just implementation trivia.
- Confirm with the user, before starting this phase, the bounded-queue capacity default and the cancel-vs-clear-queue default behavior proposed in Step 6 — both are judgment calls this plan makes a recommendation on but does not treat as pre-approved.
