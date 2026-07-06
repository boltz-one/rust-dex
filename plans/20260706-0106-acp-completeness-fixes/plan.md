---
title: "Fix 35 completeness gaps found in 4-way acpx-to-crates/acp port audit"
description: "Wire orphaned/dead-code functions into real call paths and fix field-level behavioral divergences from acpx across permissions, reconnect, session persistence, and Windows spawn."
status: done
priority: P1
effort: 41h
branch: feat/acp-completeness
lane: high-risk
tags: [acp, agent-client-protocol, rust-port, completeness-audit, security, reconnect]
created: 2026-07-06
---

# Plan: Fix crates/acp Completeness Gaps (post-audit)

## Background

`plans/20260705-1718-acpx-to-acp-crate-port/` (6 phases) was marked "done" (274 tests, `make check-all` clean). A 4-way independent audit (4 agents, each reading acpx TS function-by-function against the Rust port) found a recurring pattern: functions ported + unit-tested but never wired into the real call path (orphaned), or wired with field-level behavioral divergence happy-path tests didn't catch. Two follow-up verification passes confirmed the top 10 highest-priority claims via direct grep/read — see:
- `research/researcher-01-high-priority-verification.md`
- `research/researcher-02-secondary-verification.md`

35 gaps total (8 HIGH, 13 MEDIUM, 14 LOW — see full list in the originating task brief, reproduced per-phase below). This plan fixes ALL 35. No implementation has started; this is planning only.

**Lane: high-risk** — gaps 1/2/3/25 touch permission enforcement (authorization) and its audit trail (`PermissionEscalationEvent`/`PermissionStats`); gap 8 is an import-time trust-boundary check. Per `.claude/workflows/feature-intake.md` this is a hard-gate lane: every phase below carries an ADR Rationale section for its non-trivial decisions.

## Scope boundary (applies to every phase)

Other unrelated Claude Code sessions may be concurrently active in this same repo on unrelated work (`crates/rope/`, `crates/ui/`, `examples/ui_gallery/`, `docs/decisions/0007-0009`). Every phase in this plan touches **only**: `crates/acp/**`, `crates/acp/Cargo.toml`, new lines appended to root `Cargo.toml` (never touch existing `rope`/`unicode-segmentation` lines), `plans/20260705-1718-acpx-to-acp-crate-port/**`, `plans/20260706-0106-acp-completeness-fixes/**`, and `docs/decisions/0001-0006.md` + one new `docs/decisions/0010-*.md` (never 0007-0009).

## Phases

| # | Phase | Tier | Gaps | Status | Effort | File-ownership (primary) |
|---|-------|------|------|--------|--------|---------------------------|
| 1 | [Permission Policy, Escalation Audit Trail, Authenticate RPC, Permission Stats](./phase-01-permission-policy-authenticate-wiring.md) | HIGH | 1,2,3,24,25 | done | 6h | `client/{handshake,handlers,state}.rs`, `runtime/public/contract/options.rs`, `permissions/*`, `runtime/engine/manager_spawn.rs` |
| 2 | [Claude Session-Create Timeout + Reconnect Hardening + Prompt-Turn Timeout Recovery](./phase-02-reconnect-hardening-claude-timeout.md) | HIGH | 4,5,6 | done | 10h | `runtime/engine/reconnect/*`, `runtime/engine/prompt_turn/*`, `agent_command/` (new claude quirks), `tests/fixtures/fake_agent/`, `tests/runtime_lifecycle.rs` |
| 3 | [Conversation Trim Determinism (IndexMap) + Import Agent-Match Security Check](./phase-03-conversation-trim-import-security.md) | HIGH | 7,8 | done | 3h | `Cargo.toml` (root+crate), `session/conversation_model/{trim,conversation}.rs`, `session/record.rs`, `session/import/agent_match.rs` |
| 4 | [Session Lifecycle Wiring: Close RPC, Control-Error Wrapping, Model Application, Reconnect Reconciliation, Load Drain](./phase-04-session-lifecycle-reconnect-model-state.md) | MEDIUM | 9,10,11,12,15,16,23 | done | 6h | `client/mod.rs`, `runtime/engine/manager/queue_control.rs`, `runtime/engine/connected_session.rs`, `runtime/engine/manager_spawn.rs`, `runtime/engine/session_options.rs`, `runtime/engine/reconnect/mod.rs`, `session/model_application.rs`, `session/model_state.rs`, `agent_command/model_support.rs` |
| 5 | [Runtime Contract Dynamism: Capabilities, Self-Describing Handle, Session Validation](./phase-05-runtime-contract-dynamism.md) | MEDIUM | 13,14,34 | done | 3h | `runtime/engine/manager/{mod,status}.rs`, `runtime/public/{handle_state,shared}.rs` |
| 6 | [Conversation/Tool-Use Fidelity + ClientOperation Progress Events](./phase-06-conversation-fidelity-client-operations.md) | MEDIUM | 17,18,19,20 | done | 4h | `session/conversation_model/{trim,tool_use,tool_call,record}.rs`, `filesystem.rs`, `terminal/mod.rs`, `runtime/public/events/types.rs` |
| 7 | [Windows Batch-Shell Agent Spawn + Claude Executable Resolution](./phase-07-windows-batch-shell-spawn.md) | MEDIUM | 21 (27 deferred, ADR-10) | done | 3h | `client/spawn.rs`, `agent_command/spawn_options.rs` |
| 8 | [Agent-Command Quirks & Shutdown/Persistence Refinements](./phase-08-agent-quirks-shutdown-persistence.md) | LOW | 22,26,28,29 | done | 3h | `client/shutdown.rs`, `agent_command/{command_args,registry}.rs`, `session/persistence/repository/{close,prune}.rs` |
| 9 | [Test Coverage, Legacy Migration Fidelity, Liveness Cleanup, Architecture ADR](./phase-09-test-coverage-adr-cleanup.md) | LOW | 30,31,32,33,35 | done | 3h | `session/persistence/parse.rs`, `session/model_state.rs`, `session/conversation_model/trim.rs` (doc only), `permissions/resolve_tests.rs`, `runtime/engine/reconnect/liveness.rs`, `docs/decisions/0010-*.md` |

**Total: 41h.**

## Execution order & parallelism

- **Tiers are strictly sequential**: all HIGH (1-3) land before MEDIUM (4-7) starts; all MEDIUM before LOW (8-9). Reason: several MEDIUM/LOW phases touch files a HIGH/MEDIUM phase already modified (e.g. `reconnect/mod.rs` touched by phase 2 then again by phase 4) — safe only because the earlier phase is fully merged first, not true file-disjoint parallelism across tiers.
- **Within HIGH tier, phases 1/2/3 are file-disjoint** (verified below) — can run in parallel.
- **Within MEDIUM tier, phases 4/5/6/7 are file-disjoint** (verified below) — can run in parallel.
- **Within LOW tier, phases 8/9 are file-disjoint** — can run in parallel.
- **Correction to the originating brief's suggested grouping**: the brief proposed `{1,2,3}/{4}/{5,6}/{7,8}` as 4 parallel HIGH groups. Verified via direct code research: gap 4's fix (Claude `session/new` timeout) must live inside `runtime/engine/reconnect/mod.rs::create_fresh_session` (the only place `session/new` is actually called in the reconnect-owning code path — NOT `client/mod.rs`, which only wraps the `initialize` handshake for Gemini's analogous timeout). That is the same file gap 5/6's reconnect-hardening work touches. Splitting gap 4 into its own phase would create a same-file conflict with phase 2. Merged into phase 2 instead — see phase 2's File-ownership matrix for the full justification.

### File-ownership matrix (HIGH tier)

| File | Phase 1 | Phase 2 | Phase 3 |
|---|---|---|---|
| `client/handshake.rs` | write | — | — |
| `client/handlers.rs` | write | — | — |
| `client/state.rs` | write | — | — |
| `runtime/public/contract/options.rs` | write | — | — |
| `permissions/*.rs` | write | — | — |
| `runtime/engine/manager_spawn.rs` | write (thread policy field only) | — | — |
| `runtime/engine/reconnect/{mod,replay,liveness}.rs` | — | write | — |
| `runtime/engine/prompt_turn/{task,mod,turn_result}.rs` | — | write | — |
| `agent_command/` (new claude quirks module) | — | write | — |
| `tests/fixtures/fake_agent/main.rs`, `tests/runtime_lifecycle.rs` | — | write | — |
| `Cargo.toml` (root + crate) | — | — | write |
| `session/conversation_model/{trim,conversation}.rs` | — | — | write |
| `session/record.rs` | — | — | write |
| `session/import/agent_match.rs` | — | — | write |

No row has two "write" cells → confirmed disjoint.

### File-ownership matrix (MEDIUM tier)

| File | Phase 4 | Phase 5 | Phase 6 | Phase 7 |
|---|---|---|---|---|
| `client/mod.rs` | write (new `session_close`, `session_load` suppression param) | — | — | — |
| `runtime/engine/manager/queue_control.rs` | write | — | — | — |
| `runtime/engine/connected_session.rs` | write | — | — | — |
| `runtime/engine/manager_spawn.rs` | write (model application call) | — | — | — |
| `runtime/engine/session_options.rs` | write (`_meta.claudeCode` builder) | — | — | — |
| `runtime/engine/reconnect/mod.rs` | write | — | — | — |
| `session/model_application.rs`, `session/model_state.rs`, `agent_command/model_support.rs` | write | — | — | — |
| `runtime/engine/manager/mod.rs` | — | write | — | — |
| `runtime/engine/manager/status.rs` | — | write | — | — |
| `runtime/public/{handle_state,shared}.rs` | — | write | — | — |
| `session/conversation_model/{trim,tool_use,tool_call,record}.rs` | — | — | write | — |
| `filesystem.rs`, `terminal/mod.rs` | — | — | write | — |
| `runtime/public/events/types.rs` | — | — | write | — |
| `client/spawn.rs` | — | — | — | write |
| `agent_command/spawn_options.rs` | — | — | — | write |

No row has two "write" cells → confirmed disjoint. Note phase 4 revisits `runtime/engine/reconnect/mod.rs` (already modified by HIGH phase 2) — safe because tiers run sequentially, phase 2 is merged first.

### File-ownership matrix (LOW tier)

| File | Phase 8 | Phase 9 |
|---|---|---|
| `client/shutdown.rs`, `agent_command/{command_args,registry}.rs` | write | — |
| `session/persistence/repository/{close,prune}.rs` | write | — |
| `session/persistence/parse.rs`, `session/model_state.rs` | — | write |
| `session/conversation_model/trim.rs` (doc comment only) | — | write |
| `permissions/resolve_tests.rs` | — | write |
| `runtime/engine/reconnect/liveness.rs` | — | write |
| `docs/decisions/0010-*.md` | — | write |

Disjoint.

## Cross-Phase ADR Index (new decisions, this plan)

| # | Decision | Resolved in |
|---|---|---|
| ADR-7 | `PermissionPolicy` threading shape: programmatic field on `AcpRuntimeOptions`, no CLI/config-file loader (this crate has no CLI) | Phase 1 (CONFIRMED) |
| ADR-8 | `PermissionEscalationEvent` surfaced via a synchronous non-blocking callback field (`on_permission_escalation`) on `AcpRuntimeOptions`, not a new `AcpRuntimeTurn`-scoped event-stream variant | Phase 1 (CONFIRMED) |
| ADR-9 | `is_error` reset semantics: port acpx's exact behavior (reset to a concrete bool on every result-triggering update) rather than keep Rust's current "sticky" `Option<bool>` behavior | Phase 6 (CONFIRMED) |
| ADR-10 | Windows batch-shell wrapping ported into the real agent-spawn path (`client/spawn.rs`) reusing existing pure helpers; wrapper-script `.exe`-target sniffing (acpx's `resolveWindowsWrapperExecutable`) and `resolveClaudeCodeExecutable` (gap 27) explicitly deferred with a documented TODO, not implemented this pass | Phase 7 (CONFIRMED — scope narrowed) |
| ADR-11 | `ConnectedSession`'s long-lived-client-per-session architecture formally documented as an accepted deviation from acpx's ephemeral-client-per-turn model (no code change) | Phase 9, `docs/decisions/0010-connected-session-long-lived-client.md` |

Existing ADR-1 through ADR-6 (`docs/decisions/0001-0006*.md`) are unaffected — not re-litigated here.

## Verification (every phase, after implementation)

```
cargo fmt -p boltz-acpx
cargo check -p boltz-acpx --all-targets --features test-support
cargo test -p boltz-acpx --features test-support   # must not regress below 274; each phase should grow it
make check-all
```

## Already confirmed fine — do not touch

Conversation-model truncation *order*, atomic same-directory writes, suffix-id ambiguity handling, symlink-escape rejection, terminal descendant-tracking simplification, `PermissionRequestHandler` non-blocking guarantee, ADR-1 through ADR-6, `perf_metrics`, the entire `queue/*` (Phase 6 of the original port) module. No phase in this plan touches `queue/*` except incidentally where a fix elsewhere calls into a queue-adjacent type it already used before.

## Locked-in Decisions (confirmed, no further user sign-off gate)

The 8 items below were open questions during plan drafting. Since none blocks starting implementation on technical merit alone (each has a clear default aligned with acpx fidelity, this crate's existing conventions, or a documented, low-risk deviation), they have been proactively decided rather than left pending. Each phase file's ADR/Risk/Next-steps sections have been updated to reflect these as final — implementation may proceed against them directly. Revisit only if implementation surfaces evidence a decision was wrong (e.g. a real test failure), not as a matter of routine re-confirmation.

1. **Gap 2 escalation-callback shape (ADR-8)** — **Decided: synchronous callback field.** `on_permission_escalation: Option<Arc<dyn Fn(PermissionEscalationEvent) + Send + Sync>>` on `AcpRuntimeOptions`, mirroring acpx's `onPermissionEscalation?: (event) => void`. Rationale: permission requests can occur outside an active prompt turn (e.g. during `terminal/create`), so a per-turn `AcpRuntimeEvent` variant would either drop those escalations or need an awkward turn-lookup; matches acpx's actual client-level placement and requires zero changes to the already-stable `AcpRuntimeTurn` event-stream contract. See Phase 1.
2. **Gap 19 is_error semantics (ADR-9)** — **Decided: port acpx's exact behavior.** `is_error` always resets to a concrete `bool` on every result-triggering update, matching acpx, not Rust's current "sticky" `Option<bool>` behavior. Rationale: this audit's entire premise is finding *undocumented* divergence from the reference implementation — an undocumented "improvement" is indistinguishable from a bug to a future maintainer diffing against acpx; acpx's behavior is itself a deliberate design choice (always computes a concrete boolean), not an oversight to defensively diverge from. See Phase 6.
3. **Gap 21 Windows scope (ADR-10)** — **Decided: narrow scope.** In scope: wrapping the real agent-spawn command in a shell when it resolves to `.cmd`/`.bat` (the actual bug — agent spawn fails outright on Windows today for npx-based agents). Deferred, not implemented: acpx's wrapper-script `.exe`-target content-sniffing (`resolveWindowsWrapperExecutable`) and gap 27 (`resolveClaudeCodeExecutable`, which depends on it) — left as a documented `// TODO(gap-21b, gap-27)`. Rationale: this crate's testing convention is real-subprocess/no-mocks; content-sniffing real `.cmd`/`.bat` wrapper scripts can't be genuinely tested without live Windows CI access, which this environment lacks — shipping untested Windows-specific script-parsing logic is riskier than shipping the mechanical, directly-testable half now and leaving a clear TODO for the rest. See Phase 7 (scope and effort already reduced from 5h to 3h to reflect this).
4. **Gap 12 model-application timing** — **Decided: apply unconditionally**, matching acpx's `applyRequestedModelIfAdvertised` (runs regardless of fresh-vs-resumed), rather than gating on `!resumed`. Rationale: acpx has no separate resumed-record code path here, so unconditional application is the faithful port; gating on `!resumed` would be a Rust-specific behavior change with no acpx precedent to justify it pre-emptively. See Phase 4.
5. **Gap 13 capabilities source** — **Decided: read from the live in-memory `connected.record`** when a session is currently connected (zero extra I/O), falling back to `session_store.load()` only for a not-currently-connected session — rather than acpx's always-reload-from-store behavior. Rationale: avoids a redundant disk read for the common case (checking capabilities of a session already being driven); the live/store divergence risk is theoretical (would require an external process mutating the on-disk record mid-session) and this crate has no multi-process users today. See Phase 5.
6. **Gap 3 SDK method availability** — **Decided: verify first, fall back if absent.** Whether `agent-client-protocol`'s Rust SDK exposes a typed `authenticate` method is unverified — Phase 1's Implementation Step 1 checks this before any other code in the phase. If absent, fall back to the hand-rolled-RPC pattern already established by ADR-1 (`jsonrpc_gap.rs`) for `session/set_mode` etc. This is a verify-then-branch procedural step, not an open design question — no separate confirmation needed either way.
7. **Gap 22 Qoder arg-injection** — **Decided: defer.** `buildQoderAcpCommandArgs` (Qoder-specific `--max-turns`/`--allowed-tools` CLI arg injection) is not ported this pass; only the two already-ported-but-orphaned helpers (`resolve_agent_close_after_stdin_end_ms`, `should_ignore_non_json_agent_output_line`) are wired in. Rationale: it's a narrow, single-agent CLI-arg feature with no reported need yet; deferring avoids speculative scope creep on a LOW-priority gap. See Phase 8.
8. **Gap 26 codex_compat wiring** — **Decided: wire it in now.** `is_codex_acp_command`/`is_legacy_zed_codex_acp_invocation` are wired into `agent_command/registry.rs`'s command-resolution path this pass. Rationale: low cost (a natural insertion point already exists in the resolution path), and leaving fully-implemented, tested detection logic permanently orphaned is worse than the small cost of wiring it in — unlike gap 22, this isn't new feature scope, just connecting existing logic. See Phase 8.
