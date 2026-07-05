# Phase 6: Conversation/Tool-Use Fidelity + ClientOperation Progress Events

## Context links

- Plan: [plan.md](./plan.md)
- Research: dedicated research pass (conversation_model trim/tool_use, import agent_match group)
- Original port plan phase to correct after this lands: [phase-05-session-persistence.md](../20260705-1718-acpx-to-acp-crate-port/phase-05-session-persistence.md) (gaps 17,18,19), [phase-04-runtime-engine-public-contract.md](../20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md) (gap 20)
- Depends on: [Phase 3](./phase-03-conversation-trim-import-security.md) (this phase revisits `session/conversation_model/trim.rs`, already modified there for gap 7 — must start from Phase 3's merged version)
- Parallel (once HIGH tier is merged): [Phase 4](./phase-04-session-lifecycle-reconnect-model-state.md), [Phase 5](./phase-05-runtime-contract-dynamism.md), [Phase 7](./phase-07-windows-batch-shell-spawn.md) (file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/session/conversation_model/{trim.rs,tool_use.rs,tool_call.rs,record.rs}`, `crates/acp/src/filesystem.rs`, `crates/acp/src/terminal/mod.rs`, `crates/acp/src/runtime/public/events/types.rs`. No other files. `trim.rs` edits here are additive to Phase 3's IndexMap change — do not revert Phase 3's eviction-order fix.

## Overview

- **Priority:** P2 (MEDIUM)
- **Status:** pending
- **Description:** 4 gaps, all inside the conversation-model/tool-call data layer plus one cross-cutting mechanism (`ClientOperation`) incorrectly dismissed as CLI-only: (17) tool-result `output` field never truncated, (18) `to_raw_input` double-JSON-encodes string values, (19) `is_error` reset semantics diverge from acpx on partial tool-call updates, (20) `ClientOperation`/`onOperation` progress events — a real non-CLI runtime-engine mechanism in acpx — dismissed as out-of-scope and never wired.

## Key Insights (from research)

- **Gap 17**: `trim.rs`'s tool-result trimming loop only trims `result.content`'s `Text` variant; `result.output: Option<Value>` is untouched. acpx's `trimRuntimeToolResult` also trims `output` — but **only when it's a raw string** (`typeof result.output === "string"`), leaving object/array outputs alone. The Rust fix needs a `Value::String(s)` match arm, not a blanket trim of any `Value`.
- **Gap 18**: `to_raw_input`'s `Some(value) => trim_runtime_text(&value.to_string(), ...)` calls `serde_json::Value::to_string()` unconditionally — for `Value::String("abc")` this produces the JSON-quoted `"\"abc\""`, not the raw `abc`. acpx's `toRawInput` special-cases `typeof value === "string"` to return the raw string (trimmed), only falling to `JSON.stringify` for non-string values.
- **Gap 19**: acpx's `statusIndicatesError(status)` always returns a concrete `boolean` (`false` when `status` is absent/non-string), and `applyToolResultUpdate` always writes that concrete boolean into `is_error` on any result-triggering update (raw_output present OR title/kind/status present). Rust's current code computes `is_error: update.status.as_deref().map(status_indicates_error)` — `None` when `status` is absent — then `upsert_tool_result` does `is_error.unwrap_or(fallback.is_error)`, which **preserves** a prior `true` across an update with no `status` field (e.g. a title-only update). This means a previously-recorded error can survive stale after a should-have-cleared update, diverging from acpx.
- **Gap 20**: `filesystem.rs`'s doc comment claims `ClientOperation`/`onOperation` is "CLI-only... out of this crate's scope" — **this is factually wrong**, confirmed via direct read of `others/acpx/src/runtime/engine/manager.ts:1101-1109`, which wires `onClientOperation` (fed by `filesystem.ts`/`terminal-manager.ts`'s `emitOperation` calls) into `recordClientOperation(turn.conversation, ...)` AND `emitRuntimeTurnEvent(task, {type:"client_operation", ...})` — i.e. it's a first-class **runtime-engine** event (both persisted into conversation state and surfaced live to the event stream), at the same tier as `onSessionUpdate`, which this Rust port already fully implements. `record_client_operation` (`conversation_model/record.rs:135-142`) already exists, fully implemented, zero call sites.

## Requirements

1. `trim.rs`'s tool-result trimming: when `result.output` is `Value::String(s)`, replace it with `Value::String(trim_runtime_text(s, MAX_RUNTIME_TOOL_IO_CHARS))`; non-string `output` values are left untouched (matching acpx's `typeof === "string"` guard exactly).
2. `to_raw_input(value: Option<&Value>) -> String`: if `value` is `Some(Value::String(s))`, return `trim_runtime_text(s, MAX_RUNTIME_TOOL_IO_CHARS)` directly (no re-encoding); for any other `Some(value)`, keep the existing `trim_runtime_text(&value.to_string(), ...)` path; `None` unchanged (`"{}"`).
3. Per ADR-9 (CONFIRMED, plan.md): change the tool-call update logic so `is_error` is always set to a concrete `bool` (derived from `status_indicates_error(status.as_deref())`, `false` when absent) on every result-triggering update — removing the "preserve prior value when status is absent" behavior. This is a deliberate behavior change from Rust's current "sticky" semantics to match acpx exactly.
4. Correct `filesystem.rs`'s doc comment to remove the false "CLI-only, out of scope" claim (do this regardless of the rest of this phase's outcome — it's actively misleading documentation).
5. Add an operation-progress callback to `FilesystemHandlers` and `TerminalManagerOptions`/`TerminalManager` (mirroring acpx's `onOperation` on both `filesystem.ts` and `terminal-manager.ts`), threaded up through to a new `AcpRuntimeEvent::ClientOperation{method, status, summary, details, timestamp}` variant (fields matching acpx's `ClientOperation` type) and to `record_client_operation`'s actual invocation (currently orphaned).

## Architecture

```
crates/acp/src/
├── session/conversation_model/
│   ├── trim.rs      # + output-string trimming in the tool-result loop (gap 17)
│   ├── tool_use.rs  # to_raw_input: + Value::String special case (gap 18)
│   ├── tool_call.rs # apply_tool_call_update: is_error always concrete bool (gap 19)
│   └── record.rs    # record_client_operation — no logic change, wired in by this phase (gap 20)
├── filesystem.rs     # doc comment fix; + operation-progress callback field on FilesystemHandlers,
│                     #   emitted around fs/read_text_file + fs/write_text_file
├── terminal/mod.rs   # + operation-progress callback field on TerminalManagerOptions/TerminalManager,
│                     #   emitted around terminal/create|output|wait_for_exit|kill|release
└── runtime/public/events/types.rs
    └── + AcpRuntimeEvent::ClientOperation { method, status, summary, details: Option<String>,
          timestamp }
```

## ADR Rationale

### ADR-9: `is_error` reset semantics — port acpx exactly (CONFIRMED)

- **Context:** Rust's current `Option<bool>`-based "sticky" semantics (a prior `true` survives a status-absent update) could be read as an intentional improvement (avoid silently un-flagging a real error) or as an accidental divergence. The task's own framing treats either choice as acceptable **if made deliberately**.
- **Decision:** This phase implements acpx's exact behavior (always-concrete-bool, reset on every result-triggering update) as the default, because: (a) the entire premise of this audit is finding *undocumented* divergence from the reference implementation — an undocumented "improvement" is indistinguishable from a bug to a future maintainer diffing behavior against acpx; (b) acpx's behavior is itself deliberate (its own code computes a concrete boolean, never `undefined`), so matching it isn't blindly copying a bug, it's copying a design choice; (c) if the "sticky" behavior is later judged genuinely more correct for this crate's use case, it should be *reintroduced* deliberately, with its own ADR, rather than silently kept by not fixing this gap.
- **Alternative considered:** keep Rust's current sticky behavior, document it as an intentional deviation. Rejected as this phase's default per (a)/(b) above. **Confirmed** (plan.md Unresolved Questions #2) — implement the always-concrete-bool reset behavior; the sticky alternative is not planned.

### Phase-local ADR: `ClientOperation` gets a dedicated event variant, not folded into an existing one

- **Context:** `AcpRuntimeEvent`'s existing variants (`TextDelta`, `Status`, `ToolCall`, `Done`, `Error`) don't naturally fit a filesystem/terminal progress notification — `ToolCall` is scoped to actual tool-call lifecycle, not raw fs/terminal RPC progress.
- **Decision:** Add a new `AcpRuntimeEvent::ClientOperation{..}` variant mirroring acpx's `ClientOperation` type shape exactly (`method: ClientOperationMethod`-equivalent string/enum, `status: running|completed|failed`, `summary: String`, `details: Option<String>`, `timestamp`).
- **Why:** matches acpx's actual placement (same tier as `onSessionUpdate`, both feeding `emitRuntimeTurnEvent`) — a host UI can show "reading file…"/"running command…" live status exactly as acpx's `--json` CLI output does, without overloading an existing variant's semantics.

## Related code files

- `crates/acp/src/session/conversation_model/trim.rs` (tool-result loop, as left by Phase 3).
- `crates/acp/src/session/conversation_model/tool_use.rs` (`to_raw_input`, L16-21).
- `crates/acp/src/session/conversation_model/tool_call.rs` (`apply_tool_call_update`, L73-87; `upsert_tool_result` in `tool_use.rs` L64-93).
- `crates/acp/src/session/conversation_model/record.rs` (`record_client_operation`, L135-142).
- `crates/acp/src/filesystem.rs` (doc comment L13-15, `FilesystemHandlers` L30-59).
- `crates/acp/src/terminal/mod.rs` (`TerminalManagerOptions` L45-51, `TerminalManager` L56-63).
- `crates/acp/src/runtime/public/events/types.rs` (`AcpRuntimeEvent` enum, L64-100).
- Reference (read-only): `others/acpx/src/session/conversation-model.ts` (`trimRuntimeToolResult` L934-941, `toRawInput` L304-314, `statusIndicatesError` L280-286, `applyToolResultUpdate` L403-422), `others/acpx/src/types.ts` (`ClientOperationMethod`/`ClientOperationStatus`/`ClientOperation` L130-147), `others/acpx/src/filesystem.ts` (`emitOperation` L233-234), `others/acpx/src/acp/terminal-manager.ts` (`emitOperation` L417-418), `others/acpx/src/acp/client.ts` (`onOperation` wiring L468-477), `others/acpx/src/runtime/engine/manager.ts` (`onClientOperation` L1101-1109, alongside `onSessionUpdate` L1091-1099).

## Implementation Steps

1. Fix `filesystem.rs`'s doc comment immediately (Requirement 4) — small, independent, do first.
2. `trim.rs`: add the `Value::String(s)` trim arm for `result.output` in the tool-result loop.
3. `tool_use.rs`: rewrite `to_raw_input` per Requirement 2.
4. `tool_call.rs`/`tool_use.rs`: change `apply_tool_call_update`'s `is_error` computation to always produce a concrete `bool` (`status_indicates_error(update.status.as_deref())`, `false` when `status` is `None`) and change `upsert_tool_result` to accept `is_error: bool` (not `Option<bool>`) for this call path, removing the `unwrap_or(fallback.is_error)` sticky-merge — confirm `upsert_tool_result`'s other call sites (if any) still make sense with a non-optional `is_error` parameter, or keep the `Option<bool>` signature but always pass `Some(concrete_value)` from this call site if other callers genuinely need the optional-preserve semantics (check before changing the shared function's signature).
5. Add `AcpRuntimeEvent::ClientOperation{..}` variant to `events/types.rs`.
6. Add an operation-progress callback field to `FilesystemHandlers` (e.g. `on_operation: Option<Arc<dyn Fn(ClientOperation) + Send + Sync>>` or route through the existing permission-handler-adjacent construction pattern for consistency) — emit around `fs/read_text_file`/`fs/write_text_file`'s start/success/failure, mirroring acpx's `emitOperation` call sites in `filesystem.ts`.
7. Same for `TerminalManagerOptions`/`TerminalManager`, emitting around `terminal/create`/`terminal/output`/`terminal/wait_for_exit`/`terminal/kill`/`terminal/release`, mirroring `terminal-manager.ts`'s `emitOperation` call sites.
8. Wire both new callbacks into the runtime-engine layer: the callback calls `record_client_operation(&mut conversation, ...)` (finally giving it a call site) AND forwards the operation as an `AcpRuntimeEvent::ClientOperation` into the active turn's event stream, matching acpx's `manager.ts:1101-1109`'s dual behavior (persisted + streamed).
9. Unit tests: `trim.rs`'s string-output truncation (and non-string output left untouched); `to_raw_input`'s string vs. non-string paths (a string input round-trips without extra quotes; a JSON object still gets `JSON.stringify`-equivalent encoding); `apply_tool_call_update`'s is_error reset (a title-only update after a prior error-flagged update clears `is_error` to `false`, matching ADR-9).
10. Real call-path integration test: drive a real fs/terminal operation through the fake-agent-backed test harness (a filesystem read/write and/or a terminal command), confirm an `AcpRuntimeEvent::ClientOperation` event is actually observed in the turn's event stream and `record_client_operation` was actually invoked (assert on the conversation's `updated_at` timestamp changing or an equivalent observable side effect), not just that the pure callback-wiring compiles.
11. `cargo fmt -p boltz-acp`, `cargo check -p boltz-acp --all-targets --features test-support`, `cargo test -p boltz-acp --features test-support`, `make check-all`.
12. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-05-session-persistence.md` (gaps 17,18,19) and `phase-04-runtime-engine-public-contract.md` (gap 20) per plan.md's housekeeping.

## Todo list

- [ ] Fix `filesystem.rs`'s false doc comment.
- [ ] `trim.rs`: trim string `output` fields.
- [ ] `to_raw_input`: string special-case.
- [ ] `apply_tool_call_update`: `is_error` always concrete bool (ADR-9).
- [ ] `AcpRuntimeEvent::ClientOperation` variant.
- [ ] `FilesystemHandlers`/`TerminalManagerOptions`: operation-progress callback.
- [ ] Wire callbacks to `record_client_operation` + event-stream emission.
- [ ] Unit tests for all 4 gaps' specific behavior.
- [ ] Integration test: real fs/terminal operation observed as a `ClientOperation` event.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green.
- [ ] Correct original plan's Phase 4 and Phase 5 status text.

## Success Criteria

- A tool-result with a string `output` field longer than the char limit is truncated in a test; a tool-result with an object `output` field is left byte-for-byte unchanged.
- `to_raw_input(Some(Value::String("abc".into())))` returns `"abc"` (3 chars, no quotes) in a test, not `"\"abc\""`.
- A tool-call sequence: error-flagged update → title-only update → asserts `is_error` is `false` after the second update (proving the reset, not the sticky-preserve).
- A real fs read/write or terminal command driven through a test produces an observable `AcpRuntimeEvent::ClientOperation` in the event stream.

## Risk Assessment

- **`upsert_tool_result`'s signature change** (Step 4) could affect other call sites if the function is shared beyond this one path — verify all callers before committing to a non-optional `is_error` parameter.
- **ADR-9 is confirmed** — implement Requirement 3 / Step 4 as specified (always-concrete-bool reset); no branch condition on user preference remains.

## Security Considerations

- `ClientOperation` events may include file paths (`summary`/`details` fields, e.g. "Read /path/to/file") — same sensitivity class as existing tool-call events already exposed through `AcpRuntimeEvent`; no new exposure tier introduced, but confirm no unintended leakage of absolute paths beyond what acpx already exposes.

## Next steps

- Proceed to [Phase 8](./phase-08-agent-quirks-shutdown-persistence.md) (LOW tier) once all MEDIUM-tier phases merge.
- ADR-9 (plan.md Unresolved Questions #2) is confirmed — no further sign-off needed before this phase is considered final.
