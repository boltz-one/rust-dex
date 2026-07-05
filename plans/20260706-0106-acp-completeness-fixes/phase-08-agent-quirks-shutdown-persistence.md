# Phase 8: Agent-Command Quirks & Shutdown/Persistence Refinements

## Context links

- Plan: [plan.md](./plan.md)
- Research: dedicated research pass (Windows spawn, Qoder, misc LOW gaps group)
- Original port plan phase to correct after this lands: [phase-02-protocol-transport-lifecycle.md](../20260705-1718-acpx-to-acp-crate-port/phase-02-protocol-transport-lifecycle.md) (gaps 22,26), [phase-05-session-persistence.md](../20260705-1718-acpx-to-acp-crate-port/phase-05-session-persistence.md) (gaps 28,29)
- Depends on: all MEDIUM-tier phases (4,5,6,7) merged
- Parallel: [Phase 9](./phase-09-test-coverage-adr-cleanup.md) (file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/client/shutdown.rs`, `crates/acp/src/agent_command/{command_args.rs,registry.rs}`, `crates/acp/src/session/persistence/repository/{close.rs,prune.rs}`. No other files.

## Overview

- **Priority:** P3 (LOW — edge cases, small individual size, bundled into one phase)
- **Status:** pending
- **Description:** 4 small gaps: (22) Qoder's already-ported stdin-close grace period and benign-output-line filter are never called from `client/shutdown.rs`; Qoder's CLI-arg injection (`buildQoderAcpCommandArgs`) is entirely unported — decide and document whether to add it now; (26) `codex_compat.rs`'s codex-acp detection functions are ported+tested but never consulted during agent-command resolution; (28) `close_session` always sends SIGTERM then SIGKILL regardless of a previously-recorded exit signal; (29) `PruneOptions` is missing a `older_than_ms` convenience field acpx has.

## Key Insights (from research)

- **Gap 22**: `resolve_agent_close_after_stdin_end_ms(agent_command) -> Result<u64>` and `should_ignore_non_json_agent_output_line(agent_command, trimmed_line) -> bool` (`command_args.rs`, both Qoder-specific: `qodercli` basename detection) are fully implemented and unit-tested, zero call sites outside tests. `client/shutdown.rs::shutdown_agent_process` hardcodes `SIGTERM_GRACE = 1500ms` and closes stdin immediately (`drop(child.stdin.take())`) with no per-agent-command delay before SIGTERM — this is where `resolve_agent_close_after_stdin_end_ms` needs to gate the stdin-close-to-SIGTERM timing. The benign-output-line filter's natural home is wherever non-JSON stdout lines are currently logged as warnings (need to locate this call site — likely in `client/transport.rs`'s read loop, note: that file is out of this phase's scope boundary, so if the filter's natural call site lives there, this phase can only add the filter function's wiring point as a TODO-flagged follow-up, or the scope boundary needs a one-line exception — confirm at implementation time and document whichever is chosen). `buildQoderAcpCommandArgs` (Qoder-specific `--max-turns`/`--allowed-tools` arg injection) has zero Rust port — per plan.md's Unresolved Questions #7, this phase proposes deferring it (document the decision, don't implement).
- **Gap 26**: `is_codex_acp_command(command, args)` and `is_legacy_zed_codex_acp_invocation(agent_command)` (`agent_command/codex_compat.rs`) are fully implemented+tested, zero call sites outside tests and their own re-export. `agent_command/registry.rs::built_in_agent_registry()` hardcodes the modern `@agentclientprotocol/codex-acp@^0.0.44` package name with no branch checking for the legacy Zed package name (`@zed-industries/codex-acp`). Per plan.md's Unresolved Questions #8, this phase proposes wiring both functions into the agent-command resolution path (low cost, natural insertion point already identified) rather than deferring.
- **Gap 28**: `close.rs::best_effort_terminate` (Unix) unconditionally does `libc::kill(pid, SIGTERM); libc::kill(pid, SIGKILL);` back-to-back with no gap/check — acpx's `killSignalCandidates(lastAgentExitSignal)` skips the SIGTERM step entirely when the last known exit signal was already `SIGKILL` (`["SIGKILL"]` only, vs. the default `["SIGTERM","SIGKILL"]`, vs. `[normalized, "SIGKILL"]` for any other recorded signal). `record.last_agent_exit_signal: Option<String>` already exists and is populated (`lifecycle.rs`) — `close.rs` simply never reads it.
- **Gap 29**: `PruneOptions` (`prune.rs`) has `before: Option<String>` but no `older_than_ms: Option<u64>` convenience field acpx's equivalent type has (`before?: Date`, `olderThanMs?: number`) — zero current callers of either field in production code, so this is a pure API-completeness addition with no immediate behavior-wiring risk.

## Requirements

1. `client/shutdown.rs::shutdown_agent_process` uses `resolve_agent_close_after_stdin_end_ms(agent_command)` to determine the delay between closing stdin and sending SIGTERM (currently immediate), for the specific agent command being shut down.
2. The benign-output-line filter (`should_ignore_non_json_agent_output_line`) is wired into wherever non-JSON agent stdout lines are currently logged/warned about — if that call site is outside this phase's declared scope boundary (`client/transport.rs`), document this explicitly and either (a) request a one-line scope exception for that specific call site, or (b) defer with a clear TODO — do not silently skip without a documented reason.
3. `buildQoderAcpCommandArgs` is explicitly deferred (not implemented) — document the decision in this phase's Implementation status.
4. `is_codex_acp_command`/`is_legacy_zed_codex_acp_invocation` are wired into `agent_command/registry.rs`'s (or `command_args.rs`'s) command-resolution path, so a legacy Zed-package codex-acp invocation is correctly detected and can be handled distinctly from the modern package (matching whatever distinct handling acpx applies once detected — check `others/acpx/src/acp/codex-compat.ts`'s actual downstream usage of these predicates before wiring, to confirm what "handling" means here, not just detection for its own sake).
5. `close_session` (`close.rs`) reads `record.last_agent_exit_signal` and applies acpx's exact `killSignalCandidates` logic: `None` → `[SIGTERM, SIGKILL]`; `Some("SIGKILL")` → `[SIGKILL]` only; `Some(other)` → `[other, SIGKILL]`.
6. `PruneOptions` gains `older_than_ms: Option<u64>`, converted to an equivalent `before` cutoff at the point prune logic consumes it (mirroring acpx's `Date.now() - olderThanMs` conversion, or wherever the pure-time-to-cutoff math naturally belongs).

## Architecture

```
crates/acp/src/
├── client/shutdown.rs        # shutdown_agent_process: + agent-command-aware stdin-close delay
├── agent_command/
│   ├── command_args.rs        # resolve_agent_close_after_stdin_end_ms /
│   │                            # should_ignore_non_json_agent_output_line — no logic change,
│   │                            # wired in by this phase
│   └── registry.rs             # + codex-acp legacy-vs-modern detection consulted during
│                                 # command resolution
└── session/persistence/repository/
    ├── close.rs                # best_effort_terminate: + killSignalCandidates-equivalent logic
    └── prune.rs                 # PruneOptions: + older_than_ms field + cutoff conversion
```

## ADR Rationale

No cross-phase ADR needed for gaps 28/29 (direct, unambiguous ports of already-specified acpx behavior). Gap 22's `buildQoderAcpCommandArgs` deferral and gap 26's wiring decision are both explicitly called out in plan.md's Unresolved Questions (#7, #8) rather than requiring a full ADR — they're scope calls, not architectural decisions with alternatives to weigh.

## Related code files

- `crates/acp/src/client/shutdown.rs` (full 109-line file — `shutdown_agent_process` L28-58, `SIGTERM_GRACE`/`SIGKILL_GRACE` constants L20-21).
- `crates/acp/src/agent_command/command_args.rs` (`resolve_agent_close_after_stdin_end_ms` L112-119, `should_ignore_non_json_agent_output_line` L127-133).
- `crates/acp/src/agent_command/codex_compat.rs` (`is_codex_acp_command` L7-9, `is_legacy_zed_codex_acp_invocation` L17-19).
- `crates/acp/src/agent_command/registry.rs` (`built_in_agent_registry` L20-38).
- `crates/acp/src/session/persistence/repository/close.rs` (full 104-line file — `best_effort_terminate` L14-24, `close_session` L42-59).
- `crates/acp/src/session/persistence/repository/prune.rs` (`PruneOptions` L17-22).
- `crates/acp/src/session/record.rs` (`last_agent_exit_signal` field, L68 — read only).
- Reference (read-only): `others/acpx/src/acp/agent-command.ts` (`buildQoderAcpCommandArgs` L98-119), `others/acpx/src/acp/codex-compat.ts` (full file, to confirm downstream usage of the two predicates), `others/acpx/src/session/persistence/repository.ts` (`killSignalCandidates` L295-306, call site L441; `PruneOptions` type L308-314).

## Implementation Steps

1. Wire `resolve_agent_close_after_stdin_end_ms` into `shutdown_agent_process`: after closing stdin, wait the resolved delay (via whatever timer primitive the function's shutdown sequence already uses for its existing grace periods) before sending SIGTERM.
2. Locate the actual call site for non-JSON agent-stdout-line warnings (likely `client/transport.rs`, outside this phase's declared scope) — if confirmed there, either request the scope exception or document the deferral; if it's actually reachable within scope (e.g. logged from `shutdown.rs` itself during drain), wire `should_ignore_non_json_agent_output_line` there directly.
3. Document the `buildQoderAcpCommandArgs` deferral decision in this file's Implementation status once finalized (Requirement 3).
4. Read `others/acpx/src/acp/codex-compat.ts` in full to confirm what downstream behavior the two predicates actually gate (e.g. does detecting a legacy Zed invocation change which args get built, or just log a deprecation notice?) — wire the equivalent behavior into `registry.rs`/`command_args.rs`'s resolution path.
5. In `close.rs::best_effort_terminate`, read `record.last_agent_exit_signal`, compute the acpx-matching signal candidate list, iterate it with `try`/swallow-per-signal semantics (matching acpx's `try { process.kill(...) } catch {}` per-signal tolerance).
6. Add `older_than_ms: Option<u64>` to `PruneOptions`; at the point prune logic reads `before`, if `older_than_ms` is set and `before` is not, compute an equivalent cutoff (current time minus `older_than_ms`) — confirm exact timestamp type/format used elsewhere in the prune logic for consistency.
7. Unit tests: shutdown delay actually elapses before SIGTERM for a Qoder-detected command (and doesn't for a non-Qoder command, matching the default); `close_session`'s signal-candidate list for all 3 `last_agent_exit_signal` cases (`None`, `Some("SIGKILL")`, `Some("SIGTERM")` or other); `PruneOptions::older_than_ms`'s cutoff conversion.
8. Real call-path integration test: spawn a fake agent under a command name that triggers Qoder detection (or a test-only override if the detection is strictly basename-based and the fake-agent binary's name can't easily be changed — check `resolve_agent_close_after_stdin_end_ms`'s exact matching rule first), shut it down, confirm (via timing or a fake-agent-side log of when SIGTERM was received relative to stdin closing) the delay was actually applied.
9. `cargo fmt -p boltz-acp`, `cargo check -p boltz-acp --all-targets --features test-support`, `cargo test -p boltz-acp --features test-support`, `make check-all`.
10. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-02-protocol-transport-lifecycle.md` (gaps 22,26) and `phase-05-session-persistence.md` (gaps 28,29) per plan.md's housekeeping.

## Todo list

- [ ] `shutdown_agent_process` applies `resolve_agent_close_after_stdin_end_ms`'s delay.
- [ ] `should_ignore_non_json_agent_output_line` wired in (or deferral documented with reason).
- [ ] `buildQoderAcpCommandArgs` deferral documented.
- [ ] `is_codex_acp_command`/`is_legacy_zed_codex_acp_invocation` wired into command resolution.
- [ ] `close_session` applies `killSignalCandidates`-equivalent logic based on `last_agent_exit_signal`.
- [ ] `PruneOptions.older_than_ms` added + wired into cutoff computation.
- [ ] Unit tests for all of the above.
- [ ] Integration test: real Qoder-detected shutdown delay observed.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green.
- [ ] Correct original plan's Phase 2 and Phase 5 status text.

## Success Criteria

- A test proves a Qoder-detected agent's shutdown actually waits the configured delay before SIGTERM is sent, while a non-Qoder agent's shutdown is unaffected (regression guard).
- A test proves `close_session` does NOT send SIGTERM when `last_agent_exit_signal == Some("SIGKILL")`, only SIGKILL — a real behavior change from today's unconditional both-signals approach.
- `PruneOptions { older_than_ms: Some(n), before: None, .. }` produces the same prune result as an equivalent hand-computed `before` timestamp, verified in a test.

## Risk Assessment

- **Gap 22's benign-output-filter call site may fall outside this phase's declared file scope** — if so, this phase must not silently skip it; either get a scope exception or leave a clearly-documented deferred TODO, per Requirement 2.
- **Gap 26's actual downstream behavior is unverified** — Step 4 requires reading acpx's `codex-compat.ts` usage before wiring anything, to avoid wiring detection with no corresponding behavior change (dead wiring is only marginally better than dead code).

## Security Considerations

- Gap 28's signal-selection logic operates on already-locally-recorded process state (`last_agent_exit_signal`), no new untrusted-input surface.

## Next steps

- Proceed to [Phase 9](./phase-09-test-coverage-adr-cleanup.md) in parallel (file-disjoint).
- plan.md #7 (Qoder arg-injection deferral) and #8 (codex_compat wiring scope) are both confirmed per this phase's proposed defaults — no further sign-off needed.
