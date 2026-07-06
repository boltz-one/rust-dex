# Phase 2: Claude Session-Create Timeout + Reconnect State Machine Hardening + Prompt-Turn Timeout Recovery

## Context links

- Plan: [plan.md](./plan.md)
- Research: [researcher-01](./research/researcher-01-high-priority-verification.md), [researcher-02](./research/researcher-02-secondary-verification.md)
- Original port plan phase to correct after this lands: [phase-04-runtime-engine-public-contract.md](../20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md) (gap 6), [phase-02-protocol-transport-lifecycle.md](../20260705-1718-acpx-to-acp-crate-port/phase-02-protocol-transport-lifecycle.md) (gap 4)
- Parallel: [Phase 1](./phase-01-permission-policy-authenticate-wiring.md), [Phase 3](./phase-03-conversation-trim-import-security.md) (file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/runtime/engine/reconnect/{mod.rs,replay.rs,liveness.rs}`, `crates/acp/src/runtime/engine/prompt_turn/{task.rs,mod.rs,turn_result.rs}`, `crates/acp/src/agent_command/` (new file for Claude-specific timeout resolution, mirroring `gemini_quirks.rs`), `crates/acp/tests/fixtures/fake_agent/main.rs`, `crates/acp/tests/runtime_lifecycle.rs` (extend), possibly a new `crates/acp/tests/reconnect_lifecycle.rs`. No other files.

## Overview

- **Priority:** P1 (HIGH — this is the single highest-risk file in the whole crate per the original port plan's own Risk Assessment, currently at **zero** unit tests)
- **Status:** pending
- **Description:** Three related fixes, all landing in the reconnect/prompt-turn machinery: (a) port acpx's Claude-specific 60s `session/new` timeout (currently `AcpError::ClaudeAcpSessionCreateTimeout` is defined but never constructed anywhere), (b) add comprehensive real unit + integration test coverage for every `AcquisitionPath` branch in the reconnect state machine (currently 0 unit tests, 1 integration test covering only the Resume-succeeds happy path), (c) port acpx's "did the agent actually reply before the RPC timed out" fallback in `prompt_turn` so a timeout doesn't produce a hard failure when the agent had, in fact, already sent `session/update`s.

## Key Insights (from verification research)

- **`AcquisitionPath` has exactly 4 variants** (`reconnect/mod.rs:111-116`): `Resume`, `Load`, `RequireSameSession`, `CreateFresh`. Chosen by `choose_acquisition_path` (L119-132) based on `agent_capabilities.session_capabilities.{resume,load}` + `same_session_only`. `acquire_via_rpc` (L149-229) does the actual RPC for Resume/Load; on failure: same-session-only → `SessionResumeRequired`; else `should_fallback_to_new_session` (L100-109, classifies by RPC error code + whether conversation already has agent messages) → `create_fresh_session` with `load_error` recorded; else propagate the normalized error. **Timeout is structurally excluded from fallback** — `AcpError::Timeout` propagates via `?` before `should_fallback_to_new_session` is ever consulted, matching acpx's `isHardReconnectFailure` short-circuit but implemented as early-`?`-propagation rather than an explicit check.
- **Zero `#[test]` in `mod.rs` or `replay.rs`.** `liveness.rs` has 2 tests but they're the *only* tested file in the whole module, and `stored_process_status` (its sole function) has zero production call sites — `manager/mod.rs` does its own separate inline `is_process_alive` check, duplicating the concept (this specific piece — gap 33 — is fixed in Phase 9, out of scope here; do not touch `liveness.rs`'s logic in this phase beyond what test-writing requires, i.e. leave the orphan-vs-wire decision to Phase 9).
- **`tests/runtime_lifecycle.rs` has exactly 3 tests**, only 1 touches reconnect at all (`reconnect_after_agent_crash_resumes_backend_session`), and it exercises exactly one branch: `AcquisitionPath::Resume` → success. `Load`, `RequireSameSession`, resume-failure→fallback, and every replay-failure-rollback path (mode/model/config-option) are **completely untested**, unit or integration.
- **`tests/fixtures/fake_agent/main.rs` cannot simulate most of the untested branches today**: `initialize` hardcodes `sessionCapabilities.resume: {}`/`loadSession: false` (no toggle) — `Load` and `RequireSameSession` paths are unreachable without fixture changes. `session/resume` always returns success (no way to simulate a resume RPC error). There is **no `session/load` match arm at all** (falls into a catch-all returning `{}`). No delay/timeout toggle exists for `resume`/`load` (only `initialize` and `session/prompt` have delay toggles). **Fixture extension is a required part of this phase**, not optional — see Implementation Steps.
- **Gap 6's fix is a "thread an existing value through," not new logic**: `session::conversation_model::record::has_agent_reply_after_prompt(conversation, prompt_message_id)` (`record.rs:70-92`) is an exact, already-unit-tested port of acpx's `hasAgentReplyAfterPrompt`. It has zero call sites outside its own test module. `record_prompt_submission`'s returned message-id (needed as the `prompt_message_id` argument) is currently discarded at `prompt_turn/mod.rs:92`. The fix is: (1) capture that id, (2) thread it into `task.rs`'s timeout branch (currently L69-76, unconditionally converts any timeout to `Err(turn_result_from_timeout(...))`), (3) before converting, lock the conversation and call `has_agent_reply_after_prompt` — if true, produce a `Completed`/`end_turn` result instead of `Failed`.
- **acpx additionally does a best-effort idle-drain wait** (`waitForSessionUpdatesIdle`, 1s idle / 5s cap) before the reply-check, to give any in-flight-but-not-yet-processed `session/update` a chance to land. Rust's `task.rs` already has a `drain_task` that gets cancelled (`.cancel().await`, L80) before persistence — decide whether this is a sufficient substitute or whether a real idle-wait must be added (see ADR below).
- **Claude timeout wraps `session/new`, not `initialize`** — this happens inside `reconnect/mod.rs::create_fresh_session` (the only production call site of `AcpClient::session_new`), which already wraps the call in a **generic** caller-supplied `timeout: Option<Duration>` via `with_timeout` (`mod.rs:240`). The Claude-specific fix *layers on top of*, does not replace, this generic timeout — it needs to (a) detect `is_claude_acp_command` from `record.agent_command` (the function `is_claude_acp_command(command, args)` already exists in `agent_command/agent_detect.rs`, just needs the agent-command string split into `(command, args)` first — check `agent_command/command_args.rs` for an existing `split_command_line`-equivalent to reuse), (b) if true, use a Claude-specific default timeout (60s, env-overridable) instead of (or in addition to, whichever is smaller — match acpx: it's the *only* timeout for Claude, not layered with a separate generic one, so this may mean **substituting** the generic timeout with the Claude-specific one when `is_claude_acp_command` is true, not racing both), (c) on timeout, construct `AcpError::ClaudeAcpSessionCreateTimeout(message)` via a new `build_claude_acp_session_create_timeout_message()` (mirrors `gemini_quirks.rs::build_gemini_acp_startup_timeout_message`) instead of the generic `AcpError::Timeout`.
- **This is the file gap 4 shares with gaps 5/6** — the plan's brief originally suggested gap 4 as an independent parallel group; verified false (see plan.md's correction note). All three land in this one phase.

## Requirements

1. `create_fresh_session` in `reconnect/mod.rs` detects Claude ACP commands and applies `resolve_claude_acp_session_create_timeout_ms()` (new fn, env `ACP_CLAUDE_ACP_SESSION_CREATE_TIMEOUT_MS`, default 60000, mirroring `resolve_gemini_acp_startup_timeout_ms`'s pattern) instead of the generic caller-supplied timeout, when `is_claude_acp_command` returns true for the resolved `(command, args)`.
2. On a Claude-specific `session/new` timeout, `create_fresh_session` returns `Err(AcpError::ClaudeAcpSessionCreateTimeout(build_claude_acp_session_create_timeout_message()))` instead of the generic `AcpError::Timeout`.
3. Every `AcquisitionPath` branch has at least one direct unit test (pure-function level where possible: `choose_acquisition_path`, `should_fallback_to_new_session`) AND at least one integration test against the real (extended) fake-agent binary exercising the full RPC-level flow.
4. Every replay-failure path (`replay_desired_mode`/`replay_desired_model`/`replay_desired_config_options` each failing) has a test proving the documented rollback behavior (record's `acp_session_id`/`agent_session_id`/`acpx` state restored to pre-replay values on failure).
5. `prompt_turn`'s timeout handling checks `has_agent_reply_after_prompt` before converting a timeout to a hard failure; if the agent replied, the turn reports a successful completion (`end_turn`-equivalent), matching acpx.
6. `tests/fixtures/fake_agent/main.rs` gains the toggles needed to simulate: resume-RPC-error, load-RPC-error/success (a real `session/load` handler), a load/resume-only capability profile (no resume, or neither), and a resume/load response delay for timeout testing — without breaking any existing test that depends on current fixture defaults.

## Architecture

```
crates/acp/src/
├── runtime/engine/reconnect/
│   ├── mod.rs      # + is_claude_acp session/new timeout branch in create_fresh_session;
│   │                 # + #[cfg(test)] mod tests covering choose_acquisition_path,
│   │                 #   should_fallback_to_new_session, and acquire_session's branch dispatch
│   ├── replay.rs   # + #[cfg(test)] mod tests covering each replay fn's success + failure/rollback
│   └── liveness.rs # untouched (Phase 9 owns stored_process_status's wire-or-remove decision)
├── runtime/engine/prompt_turn/
│   ├── mod.rs      # capture record_prompt_submission's returned message id, thread into task.rs
│   └── task.rs     # timeout branch calls has_agent_reply_after_prompt before hard-failing
├── agent_command/
│   └── claude_quirks.rs   # NEW: resolve_claude_acp_session_create_timeout_ms(),
│                           # build_claude_acp_session_create_timeout_message() — mirrors
│                           # gemini_quirks.rs's shape exactly
tests/
├── fixtures/fake_agent/main.rs   # + env toggles for resume/load error injection, capability
│                                  # profile selection, resume/load response delay
└── runtime_lifecycle.rs (or new reconnect_lifecycle.rs)  # + integration tests per Requirement 3/4
```

## ADR Rationale

### ADR (phase-local, not cross-phase): idle-drain wait before the timeout reply-check

- **Context:** acpx's `runPromptTurn` calls `waitForSessionUpdatesIdle({idleMs: 1000, timeoutMs: 5000})` (best-effort, swallows its own timeout) before checking `hasAgentReplyAfterPrompt`, to give a race-condition update time to land. Rust's `task.rs` has no idle-wait primitive; it has `drain_task.cancel().await` (L80), called during normal shutdown of the turn, not specifically as a pre-check delay.
- **Decision:** Do not port a literal idle-wait timer for this phase. Instead, ensure `drain_notifications` (the task already folding `session/update`s into `connected.conversation`, L122-149) has had a chance to process any update that arrived before the timeout fired, by checking `has_agent_reply_after_prompt` only after the drain task's current queue is empty (poll/yield once, or use whatever synchronization primitive already exists between the prompt task and the drain task — inspect `prompt_turn/mod.rs`'s task-spawning code at implementation time to find the cheapest correct join point).
- **Why:** acpx's idle-wait exists because Node's event loop and its `TimeoutError` racing are less deterministic about in-flight I/O than Rust's task model with an explicit drain-task handle. Porting a literal 1s/5s timer would add real latency to every timed-out turn for a race window Rust's cooperative-yield model can likely close more cheaply. If implementation reveals the drain task's completion isn't a reliable synchronization point (e.g. it runs on a separate executor thread with no join handle), fall back to porting the literal idle-wait timer instead — document whichever approach is actually taken in this file's Implementation status once known.

## Related code files

- `crates/acp/src/runtime/engine/reconnect/mod.rs` (368 lines — `AcquisitionPath` L111-116, `choose_acquisition_path` L119-132, `should_fallback_to_new_session` L100-109, `acquire_via_rpc` L149-229, `create_fresh_session` L232-250, `acquire_session` L252-306, `connect_and_load_session` L312-360+).
- `crates/acp/src/runtime/engine/reconnect/replay.rs` (`replay_fresh_session_preferences` L130-151, `replay_desired_mode` L29-49, `replay_desired_model` L52-97, `replay_desired_config_options` L100-122, `has_preferences_to_replay` L157-161).
- `crates/acp/src/error.rs` (`SessionResumeRequired`, `SessionModeReplay`, `SessionModelReplay`, `SessionConfigOptionReplay`, `ClaudeAcpSessionCreateTimeout`, `Timeout` variants — read only, no new variants needed).
- `crates/acp/src/runtime/engine/prompt_turn/task.rs` (timeout branch L69-76), `prompt_turn/mod.rs` (L92, discarded message-id), `prompt_turn/turn_result.rs` (`turn_result_from_timeout` L34-43, `turn_result_from_stop_reason` L11-32).
- `crates/acp/src/session/conversation_model/record.rs` (`has_agent_reply_after_prompt` L70-92, `record_prompt_submission` — read to confirm exact return type).
- `crates/acp/src/agent_command/gemini_quirks.rs` (pattern to mirror for the new `claude_quirks.rs`), `crates/acp/src/agent_command/agent_detect.rs` (`is_claude_acp_command`, already exists), `crates/acp/src/client/mod.rs:88-116` (existing Gemini timeout-race pattern, read-only reference — this phase does NOT touch `client/mod.rs`).
- `crates/acp/tests/fixtures/fake_agent/main.rs` (209 lines — `initialize` L172-191 hardcoded capabilities, `session/resume` L195, no `session/load` arm, env toggles doc L7-43).
- `crates/acp/tests/runtime_lifecycle.rs` (3 existing tests, `reconnect_after_agent_crash_resumes_backend_session` L102).
- Reference (read-only): `others/acpx/src/runtime/engine/reconnect.ts` (680 lines — full branch enumeration in researcher-02's report above), `others/acpx/src/acp/agent-command.ts` (`resolveClaudeAcpSessionCreateTimeoutMs` L132, `buildClaudeAcpSessionCreateTimeoutMessage` L266, Claude timeout call site `createSession` L912-939), `others/acpx/src/runtime/engine/prompt-turn.ts` (69 lines, reply-check fallback L19-69), `others/acpx/src/session/conversation-model.ts` (`hasAgentReplyAfterPrompt` L672-687).

## Implementation Steps

1. **Design first, code second** (per the original Phase 4's own recommendation for this exact file): before writing any test or the Claude-timeout branch, write out (as code comments in `reconnect/mod.rs`'s module doc, extending the existing state diagram) every branch this phase must cover: `{Resume-success, Resume-fail-hard-timeout, Resume-fail-fallback-to-CreateFresh, Resume-fail-RequireSameSession-error, Load-success, Load-fail-fallback, Load-fail-RequireSameSession-error, RequireSameSession-immediate-error, CreateFresh-plain, CreateFresh-with-ClaudeTimeout, replay-mode-fail-rollback, replay-model-fail-rollback, replay-config-option-fail-rollback}` — 13 distinct scenarios. Confirm this list against the acpx `reconnect.ts` branch enumeration in researcher-02's report before proceeding.
2. Add `agent_command/claude_quirks.rs`: `resolve_claude_acp_session_create_timeout_ms() -> Duration` (env `ACP_CLAUDE_ACP_SESSION_CREATE_TIMEOUT_MS`, default 60_000ms — confirm whether to keep acpx's `ACPX_`-prefixed env var name or rename to `ACP_`-prefixed per this crate's identity convention, check how `gemini_quirks.rs` named its own env var for precedent) and `build_claude_acp_session_create_timeout_message(program: &str) -> String` (mirror `gemini_quirks`'s message-builder signature). Re-export from `agent_command/mod.rs`.
3. In `reconnect/mod.rs::create_fresh_session`, determine `is_claude_acp_command` from `record.agent_command` (split into `(command, args)` via whatever helper `command_args.rs` already exposes for this — check before writing new splitting logic, DRY). If true, replace the generic `timeout` parameter's effect for this call with `resolve_claude_acp_session_create_timeout_ms()`; on `Err(AcpError::Timeout(_))` from the `with_timeout(...).await??` at L240, map to `AcpError::ClaudeAcpSessionCreateTimeout(build_claude_acp_session_create_timeout_message(program))`.
4. Extend `tests/fixtures/fake_agent/main.rs`: add `ACP_FAKE_AGENT_SESSION_CAPABILITIES` (or similar) env toggle accepting a value like `"resume"`/`"load"`/`"none"` to control what `initialize` advertises (default stays `"resume"` to avoid breaking existing tests); add a real `session/load` match arm returning a `LoadSessionResponse`-shaped success by default; add `ACP_FAKE_AGENT_RESUME_ERROR_CODE`/`ACP_FAKE_AGENT_LOAD_ERROR_CODE` (numeric, when set the corresponding RPC returns a JSON-RPC error with that code instead of success); add `ACP_FAKE_AGENT_RESUME_DELAY_MS`/`ACP_FAKE_AGENT_LOAD_DELAY_MS` (mirrors existing `PROMPT_DELAY_MS` pattern) for timeout-path tests. Update the module doc comment's env-var table.
5. Write `reconnect/mod.rs`'s `#[cfg(test)] mod tests`: unit tests for `choose_acquisition_path` (all 2x2x2-relevant combinations of resume/load capability x same_session_only), `should_fallback_to_new_session` (every classified error code + the "conversation already has agent messages" guard), and any other pure-logic branch identified in Step 1 that doesn't require a live client.
6. Write `replay.rs`'s `#[cfg(test)] mod tests`: for each of `replay_desired_mode`/`replay_desired_model`/`replay_desired_config_options`, a success case and a failure case; for the failure cases, assert the specific `AcpError::SessionModeReplay`/`SessionModelReplay`/`SessionConfigOptionReplay` variant is returned (may need a fake/mock `AcpClient` or a real fake-agent instance configured to reject the specific RPC — prefer the real fake-agent per the plan's "no mocks for dead-code fixes" rule if `AcpClient` isn't cheaply fakeable in a unit-test context; if the existing test infra already has a lightweight in-process client double, confirm and reuse it).
7. Write new integration tests (extend `tests/runtime_lifecycle.rs` or add `tests/reconnect_lifecycle.rs`) against the real, now-extended fake-agent binary: (a) resume-fails-with-fallback-eligible-code → confirms `CreateFresh` ran and a fresh backend session id resulted, with `load_error` recorded on the record; (b) resume-fails-with-hard-timeout → confirms the error propagates as a timeout, no fallback attempted; (c) `RequireSameSession` policy + resume/load failure → confirms `AcpError::SessionResumeRequired`, no fallback; (d) `Load`-only capability profile → confirms the `session/load` RPC path runs end-to-end; (e) rollback test: force a config-option replay failure after a successful `CreateFresh`, confirm the record's `acp_session_id`/`agent_session_id`/`acpx` fields match their pre-replay values, not the just-created (then-rolled-back) ones; (f) Claude session-create timeout: spawn a fake agent whose command line satisfies `is_claude_acp_command` (may need a `--` args heuristic check, or run the fake-agent binary under a wrapper name/arg the detector matches — check `is_claude_acp_command`'s exact matching rule first) with `ACP_FAKE_AGENT_LOAD_DELAY_MS`-equivalent for `session/new` exceeding a shortened test timeout, confirm `AcpError::ClaudeAcpSessionCreateTimeout` is the resulting error, not a generic timeout.
8. For gap 6: in `prompt_turn/mod.rs`, capture `record_prompt_submission`'s returned message id (currently discarded at L92) into a local binding, thread it into the `run_turn_task` call/struct that eventually reaches `task.rs`. In `task.rs`'s timeout branch (L69-76), before constructing `turn_result_from_timeout`, lock `connected.conversation` and call `has_agent_reply_after_prompt(&conversation, &prompt_message_id)`; if true, construct the equivalent of a successful/`end_turn` `TurnResult` (reuse `turn_result_from_stop_reason` with the right stop-reason string, or add a small dedicated constructor if the existing one doesn't fit — check `turn_result.rs`'s exact shape first) instead of `turn_result_from_timeout`.
9. Resolve the idle-drain-wait ADR question during implementation (see ADR above) — document the actual approach taken in this file once decided.
10. Integration test for gap 6: fake agent configured to respond to `session/prompt` with a delay exceeding a short test timeout, but to still emit a `session/update` (via the existing `ACP_FAKE_AGENT_PROMPT_UPDATE_COUNT` toggle) before that delay elapses — confirm the turn reports success/`end_turn`, not `Failed{code:"TIMEOUT"}`. A second test: fake agent times out with **no** update sent at all — confirm the turn still reports `Failed{code:"TIMEOUT"}` (regression guard, the fix must not turn ALL timeouts into false successes).
11. `cargo fmt -p boltz-acp`, `cargo check -p boltz-acp --all-targets --features test-support`, `cargo test -p boltz-acp --features test-support`, `make check-all`.
12. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md` (gap 6) and `phase-02-protocol-transport-lifecycle.md` (gap 4) per plan.md's housekeeping.

## Todo list

- [ ] Write out all 13 reconnect branch scenarios as a design doc-comment before coding (Step 1).
- [ ] `agent_command/claude_quirks.rs`: timeout resolver + message builder.
- [ ] `create_fresh_session`: Claude-specific timeout substitution + error mapping.
- [ ] Extend fake-agent fixture: capability-profile toggle, real `session/load` arm, resume/load error-injection toggles, resume/load delay toggles.
- [ ] Unit tests: `choose_acquisition_path`, `should_fallback_to_new_session` (all branches).
- [ ] Unit tests: each replay fn's success + failure/rollback path.
- [ ] Integration tests: all 6 scenarios from Step 7 against the real fake-agent binary.
- [ ] Gap 6: thread prompt-message-id through `prompt_turn/mod.rs` → `task.rs`.
- [ ] Gap 6: timeout branch checks `has_agent_reply_after_prompt` before hard-failing.
- [ ] Integration tests: timeout-with-late-reply succeeds, timeout-with-no-reply still fails (regression guard).
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green; test count grows substantially past 274 (this phase alone should add ~15-20 tests given the branch count).
- [ ] Correct original plan's Phase 2 (gap 4) and Phase 4 (gap 6) status text.

## Success Criteria

- All 13 branches enumerated in Step 1 have at least one passing test (unit or integration), each traceable to a specific test name in this file's final Implementation status note.
- The rollback test (Step 7e) proves record state is genuinely restored on replay failure, not just that an error is returned.
- The Claude timeout test (Step 7f) proves the resulting error is specifically `AcpError::ClaudeAcpSessionCreateTimeout` with a message mentioning the diagnostic guidance acpx's `buildClaudeAcpSessionCreateTimeoutMessage` provides (approve-all / nonInteractivePermissions=deny / fallback guidance) — not a generic timeout error.
- The timeout-reply-check test (Step 10) proves a turn that times out at the RPC level but already received a `session/update` reports success, while a genuinely silent timeout still reports failure (both directions tested, not just the happy path).
- `cargo test -p boltz-acp --features test-support` passes, count grows from 274 by roughly the number of new tests added in this phase.

## Risk Assessment

- **This is the highest-risk file in the crate; a rushed test-writing pass could produce tests that pass without actually exercising the intended branch** (e.g. a "resume fails" test that accidentally hits `RequireSameSession` instead of `CreateFresh` fallback due to a fixture misconfiguration). Mitigate: each integration test must assert on an observable side effect unique to its intended branch (e.g. backend session id changed = fresh session; `load_error` field populated = fallback path taken; specific `AcpError` variant = hard-failure path), not just "no panic occurred."
- **Fixture changes could silently break the one existing reconnect test** (`reconnect_after_agent_crash_resumes_backend_session`) if the new capability-profile toggle's default doesn't exactly preserve today's hardcoded `resume: {}` behavior. Mitigate: default the new toggle to the current hardcoded value, run the existing test first after each fixture change before adding new toggle-driven tests.
- **The idle-drain-wait ADR's fallback path** (Step 9) could require nontrivial additional design if `drain_task` has no cheap synchronization point — budget contingency time in the 10h estimate for this specifically.

## Security Considerations

- Reconnect resolves a persisted session record by id and re-attaches to a backend session — no new untrusted-input surface is introduced by this phase's fixes (same resolution logic as before, just now tested). No security-relevant behavior change beyond what's explicitly listed in Requirements.
- The Claude-specific timeout message (`build_claude_acp_session_create_timeout_message`) should not leak raw file paths or credentials — mirror acpx's message content exactly (it's a generic diagnostic string, no dynamic secrets).

## Next steps

- Proceed to [Phase 4](./phase-04-session-lifecycle-reconnect-model-state.md) (MEDIUM tier) once this phase and Phases 1/3 are merged — Phase 4 revisits `reconnect/mod.rs` again for model-state reconciliation (gap 15) and legacy metadata (gap 23); it must start from this phase's already-tested version, not run concurrently with it.
- Unresolved question carried forward from plan.md: none specific to this phase beyond the idle-drain-wait ADR's fallback decision (resolved during implementation, not blocking planning sign-off).
