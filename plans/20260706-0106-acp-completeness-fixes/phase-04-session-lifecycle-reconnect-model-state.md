# Phase 4: Session Lifecycle Wiring — Close RPC, Control-Error Wrapping, Model Application, Reconnect Reconciliation, Load Drain

## Context links

- Plan: [plan.md](./plan.md)
- Research: [researcher-02](./research/researcher-02-secondary-verification.md) + this plan's dedicated research pass (session close, model application, handle_state, reconnect replay dead code)
- Original port plan phase to correct after this lands: [phase-04-runtime-engine-public-contract.md](../20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md) (gaps 9,12,15,16)
- Depends on: [Phase 2](./phase-02-reconnect-hardening-claude-timeout.md) (this phase revisits `reconnect/mod.rs` — must start from Phase 2's already-tested, merged version, not run concurrently with it)
- Parallel (once HIGH tier is merged): [Phase 5](./phase-05-runtime-contract-dynamism.md), [Phase 6](./phase-06-conversation-fidelity-client-operations.md), [Phase 7](./phase-07-windows-batch-shell-spawn.md) (file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/client/mod.rs`, `crates/acp/src/runtime/engine/manager/queue_control.rs`, `crates/acp/src/runtime/engine/connected_session.rs`, `crates/acp/src/runtime/engine/manager_spawn.rs`, `crates/acp/src/runtime/engine/session_options.rs`, `crates/acp/src/runtime/engine/reconnect/mod.rs`, `crates/acp/src/session/model_application.rs`, `crates/acp/src/session/model_state.rs`, `crates/acp/src/agent_command/model_support.rs`. No other files. This phase's edits to `reconnect/mod.rs` must be layered on top of Phase 2's already-merged changes (do not revert Phase 2's Claude-timeout branch or test additions).

## Overview

- **Priority:** P2 (MEDIUM — 7 gaps bundled because they all share a theme: "wire an already-implemented, already-tested pure function into the real session-creation/reconnection call path" — and several share the exact same files, making separate phases file-unsafe for parallel execution)
- **Status:** pending
- **Description:** Gaps 9 (session/close RPC never sent), 10 (Claude Code `_meta.claudeCode.options` never sent), 11 (`maybe_wrap_session_control_error` never called), 12 (`SessionAgentOptions.model` never applied to a fresh session), 15 (reconnect post-replay model-state reconciliation dead code + stale config-options on fresh session), 16 (session/load missing replay-suppression/drain), 23 (legacy Claude model-metadata parsing never called on a live response).

## Key Insights (from research)

- **Gaps 15 and 23 share state**: acpx's `applyReconnectedModelState` needs a `legacyModelMetadataPresent` flag to decide whether to call `removeModelConfigOptions` — this flag comes from the same response `.meta` field gap 23's fix (`model_state_from_session_response`) needs to inspect. Fixing them together avoids computing the same "does this response have legacy model metadata" check twice.
- **Gap 9**: the SDK (`agent-client-protocol-schema`, confirmed present at the pinned version) already has typed `CloseSessionRequest`/`CloseSessionResponse` (method `"session/close"`) and `ListSessionsRequest`/`Response` (`"session/list"`) — this is a pure wiring gap, not a missing-SDK-support gap (unlike gap 3's `authenticate`, which was unverified). acpx only sends `session/close` when `discardPersistentState: true` (`manager.ts`'s `closeBackendSession`, called from `AcpRuntimeManager.close` only in that branch) — `manager/queue_control.rs::close()`'s current signature/params must be checked for an equivalent discard flag before wiring this in.
- **Gap 11**: `maybe_wrap_session_control_error(method, error, context) -> Option<String>` already exists and is tested (`session_control_errors.rs:68-84`) — zero call sites. The fix is inserting it into `connected_session.rs`'s `set_session_mode`/`set_session_config_option` error paths, before the existing generic `wrap_err`/`normalize_agent_error` handling, with acpx's exact per-call context strings (`for mode "{mode_id}"`, `for "{config_id}"="{value}"`).
- **Gap 12**: the pure logic (`assert_requested_model_supported`, `resolve_requested_model_id` in `model_request.rs`; `current_model_id_from_set_model_response` in `model_application.rs`) already exists — `model_application.rs`'s own doc comment explicitly says this phase (referred to there as "Phase 4") is expected to call it after issuing its own `session/set_config_option` request. The missing piece is the live-client-calling half: after `connect_and_load_session` returns in `manager_spawn.rs`'s `spawn_connected_session`, if `input.session_options.model` is set, call `set_session_config_option` (or whatever the model-setting RPC actually is — confirm against `assert_requested_model_supported`'s exact contract) and apply the result via `current_model_id_from_set_model_response`.
- **Gap 15**: `sync_advertised_model_state`/`remove_model_config_options`/`clear_omitted_fresh_session_config_options` **do not exist at all** in the Rust port (not just uncalled — never written). This is genuinely new code, ported from `others/acpx/src/runtime/engine/reconnect.ts`'s `applyReconnectedModelState`/`clearOmittedFreshSessionConfigOptions` and `others/acpx/src/session/mode-preference.ts`'s `syncAdvertisedModelState`. `create_fresh_session` only calls `apply_config_options_to_record` today — if the fresh `session/new` response omits `config_options`, a stale value from an earlier connection attempt survives uncorrected.
- **Gap 16**: no suppression flag or drain/idle-wait primitive exists anywhere on `ConnectedSession`/`AcpClient` today — this is net-new state, not a missing call. acpx's `waitForSessionUpdateDrain` polls two counters (`observedSessionUpdates`/`processedSessionUpdates`) until idle-for-N-ms or a hard timeout. The Rust equivalent needs the same two counters added to `ConnectedSession` (or wherever notification-forwarding already happens) plus a suppression toggle consulted by the notification-forwarding path during `session/load`.
- **Gap 10**: `SessionAgentOptions` (model/allowedTools/maxTurns/systemPrompt) is already persisted but never surfaced into a `_meta.claudeCode.options` field on `session/new`'s request params. acpx's `buildClaudeCodeOptionsMeta` builds this unconditionally (not gated on `is_claude_acp` — Claude-only fields in a `_meta` object are simply ignored by non-Claude agents per JSON-RPC's `_meta` convention) at the `createSession` call site.
- **Where `create_fresh_session`/`session_new` calls actually live**: `reconnect/mod.rs::create_fresh_session` is the one production call site of `AcpClient::session_new` (per Phase 2's just-completed work). This is where gap 10's `_meta` field and gap 12's model-application call (or its trigger condition) both naturally attach, alongside gap 15/23's response-inspection logic — all four gaps converge on the same function, reinforcing why this phase bundles them.

## Requirements

1. `manager/queue_control.rs::close()` sends a real `session/close` RPC via a new `AcpClient::session_close(session_id)` method, gated on (a) the existing discard-vs-not distinction already present in `close()`'s parameters (confirm exact param name/shape first) and (b) `agent_capabilities.session_capabilities.close` being advertised — matching acpx's `supportsCloseSession()` gate. Unsupported-close errors are mapped the same way other unsupported-control errors are (reuse gap 11's `maybe_wrap_session_control_error`-informed pattern or the existing `AcpRuntimeErrorCode::BackendUnsupportedControl`).
2. `reconnect/mod.rs::create_fresh_session` builds a `_meta.claudeCode.options` object from `record`'s persisted `SessionAgentOptions` (model/allowedTools/maxTurns/systemPrompt) and passes it in `session/new`'s params, mirroring `buildClaudeCodeOptionsMeta`'s exact field mapping. This applies unconditionally (not gated on `is_claude_acp`) per acpx's actual behavior — non-Claude agents ignore unknown `_meta` fields.
3. `connected_session.rs`'s `set_session_mode`/`set_session_config_option` wrap RPC errors via `maybe_wrap_session_control_error` (with acpx's exact per-call context strings) before falling through to the existing generic error path.
4. After a session is created/connected (`manager_spawn.rs::spawn_connected_session`, after `connect_and_load_session` returns), if `input.session_options.model` is a non-empty string, the model is actually applied to the live agent connection via the appropriate RPC (confirmed via `model_request.rs`'s existing pure functions), and the resulting current-model-id is persisted onto the record.
5. `create_fresh_session` clears `record.acpx.config_options` when the fresh `session/new` response omits `config_options` entirely (port `clear_omitted_fresh_session_config_options`). `connect_and_load_session`'s tail (after any acquisition path) calls a ported `sync_advertised_model_state`/`apply_reconnected_model_state` equivalent, using a `legacy_model_metadata_present` flag derived from inspecting the response's `.meta` (shared with Requirement 6).
6. `create_fresh_session` and `acquire_via_rpc` (the Load/Resume RPC paths) pass each response's `.meta` alongside `.config_options` into the model-state application logic, so `model_state_from_session_response` (currently orphaned) actually gets called with real data — Claude ACP adapters that only advertise models via legacy `_meta.models` now get correct model state.
7. `acquire_via_rpc`'s `session/load` branch suppresses forwarding `session/update` notifications to the live consumer until the load RPC completes and a drain/idle-wait (new counter-based primitive on `ConnectedSession`, or the RPC completion itself if a full idle-wait proves unnecessary in Rust's concurrency model — decide during implementation, matching this phase's ADR below) confirms no further replay updates are in flight.

## Architecture

```
crates/acp/src/
├── client/mod.rs
│   └── + pub async fn session_close(&self, session_id: SessionId) -> Result<()>  (or similar,
│         matching the existing session_new/session_load/session_resume signature convention)
├── runtime/engine/
│   ├── manager/queue_control.rs   # close() calls client.session_close() when discarding +
│   │                                # capability-gated, before the existing local cleanup
│   ├── connected_session.rs       # set_session_mode/set_session_config_option error paths
│   │                                # call maybe_wrap_session_control_error first;
│   │                                # + observed/processed session-update counters (gap 16)
│   ├── manager_spawn.rs           # + model-application call after connect_and_load_session
│   ├── session_options.rs         # + build_claude_code_options_meta(SessionAgentOptions) -> Value
│   └── reconnect/mod.rs           # create_fresh_session: + _meta.claudeCode.options in session/new
│                                    #   params; + clear_omitted_fresh_session_config_options;
│                                    #   + pass response.meta into model-state application
│                                    # acquire_via_rpc: load branch gains suppression + drain wait
├── session/
│   ├── model_application.rs       # (logic already exists) — no change expected beyond confirming
│   │                                # the live-calling half in manager_spawn.rs matches its contract
│   └── model_state.rs             # + sync_advertised_model_state / apply_reconnected_model_state /
│                                    #   remove_model_config_options (NEW — ported from acpx,
│                                    #   does not exist in Rust today)
└── agent_command/model_support.rs  # (logic already exists) — wired into reconnect/mod.rs's
                                       # response-handling per Requirement 6
```

## ADR Rationale

### Phase-local ADR: session-update drain mechanism for gap 16

- **Context:** acpx's `waitForSessionUpdateDrain` is a polling loop with a real timer (1s idle / 5s hard cap by default) — a genuine wait, not just a synchronization point, because Node's single-threaded event loop can't otherwise guarantee "no more updates are queued." Rust's `ConnectedSession` has an explicit task/channel architecture where "is anything still in flight" may be answerable more precisely (e.g. checking a channel's pending-message count or an explicit in-flight counter) rather than needing a wall-clock idle-wait.
- **Decision:** Implement the two counters (`observed_session_updates`/`processed_session_updates`, or equivalently-named) on `ConnectedSession` regardless of whether a full idle-wait timer is needed, since they're needed for gap 16's suppression logic either way (suppression must know when to stop suppressing). Whether the "drain complete" signal is a wall-clock idle-wait (matching acpx exactly) or a precise "counters equal, channel empty" check (a Rust-idiomatic improvement) is decided at implementation time based on what `ConnectedSession`'s actual concurrency primitives support cheaply — document the actual choice in this file's Implementation status once known, per the same reasoning as Phase 2's analogous idle-drain-wait decision for gap 6.
- **Why:** avoids over-engineering a wall-clock timer if Rust's task model can answer "is it safe to stop suppressing" deterministically and immediately; falls back to acpx's exact timing behavior if it can't.

## Related code files

- `crates/acp/src/client/mod.rs` (existing `session_new`/`session_load`/`session_resume`/`prompt`/`cancel_session`/`set_session_mode`/`set_session_config_option` methods, L158-244 — pattern to follow for new `session_close`).
- `crates/acp/src/runtime/engine/manager/queue_control.rs` (`close()`, L74-115).
- `crates/acp/src/runtime/engine/connected_session.rs` (`set_session_mode` L130-135, `set_session_config_option` L158-170).
- `crates/acp/src/session_control_errors.rs` (`maybe_wrap_session_control_error`, L68-84 — read only, no logic change).
- `crates/acp/src/runtime/engine/manager_spawn.rs` (`spawn_connected_session`, L31-166; `ensure_session`/session-creation flow, L77-148 in `manager/mod.rs`).
- `crates/acp/src/agent_command/model_request.rs` (`assert_requested_model_supported` L85-135, `resolve_requested_model_id` L50-79).
- `crates/acp/src/session/model_application.rs` (`current_model_id_from_set_model_response` L19-27, module doc L1-12 explicitly naming this phase as the intended caller).
- `crates/acp/src/session/model_state.rs` (`advertised_model_state`, `apply_advertised_model_state`, `clear_advertised_model_state`, `apply_config_options_model_state` — existing narrower pieces, read in full before adding the new reconnect-reconciliation functions).
- `crates/acp/src/agent_command/model_support.rs` (`model_state_from_legacy_response` L86-105, `model_state_from_session_response` L108-114).
- `crates/acp/src/runtime/engine/session_options.rs` (`SessionAgentOptions`, L15-21).
- `crates/acp/src/runtime/engine/reconnect/mod.rs` (`create_fresh_session` L232-250, `acquire_via_rpc` L149-229 — **as modified by Phase 2**, read the post-Phase-2 version before editing).
- Reference (read-only): `others/acpx/src/acp/client.ts` (`closeSession` L1263-1274, `listSessions` L1277-1280, `supportsCloseSession` L513-515), `others/acpx/src/runtime/engine/manager.ts` (`closeBackendSession` L1395-1432, `createAndSaveRuntimeRecord`'s model-application call site L737-780), `others/acpx/src/acp/session-control-errors.ts` (call sites in `client.ts` L1107, L1126, L1179-1210), `others/acpx/src/session/model-application.ts` (`applyRequestedModelIfAdvertised`), `others/acpx/src/acp/agent-command.ts` (`buildClaudeCodeOptionsMeta` L300-321, `resolveClaudeCodeSettingSources` L326-330), `others/acpx/src/runtime/engine/reconnect.ts` (`applyReconnectedModelState` L357-437, `clearOmittedFreshSessionConfigOptions` L423-427), `others/acpx/src/session/mode-preference.ts` (`syncAdvertisedModelState` L160), `others/acpx/src/acp/client.ts` (`loadSessionWithOptions` ~L960-991, `waitForSessionUpdateDrain` L1985-2011).

## Implementation Steps

1. Add `AcpClient::session_close(&self, session_id: SessionId) -> Result<()>` to `client/mod.rs`, following the existing method pattern (raw SDK request via the same connection object, typed `CloseSessionRequest`/`CloseSessionResponse`).
2. In `manager/queue_control.rs::close()`, before the existing local-cleanup/shutdown logic, check the discard-vs-not parameter (confirm its exact current name/shape) and `agent_capabilities.session_capabilities.close`; if both true, call `session_close`, swallow resource-not-found errors (mirror acpx's `isAcpResourceNotFoundError` tolerance), map unsupported-close via the same pattern as gap 11's fix.
3. In `connected_session.rs`, wrap `set_session_mode`'s and `set_session_config_option`'s RPC error paths with `maybe_wrap_session_control_error(SessionControlMethod::SetMode, &err, Some(&format!("for mode \"{mode_id}\"")))` / the config-option equivalent, before the existing `normalize_agent_error`/`wrap_err` fallback — only replace the message when `maybe_wrap_session_control_error` returns `Some`.
4. Add `build_claude_code_options_meta(options: &SessionAgentOptions) -> Value` to `session_options.rs` (or `reconnect/mod.rs` if more natural given call-site locality — decide based on which file already has the right imports), porting `buildClaudeCodeOptionsMeta`'s field mapping exactly (model/allowedTools/maxTurns/systemPrompt → nested `{claudeCode:{options:{...}}}`), plus `resolveClaudeCodeSettingSources`'s env-var check (`ACPX_CLAUDE_INCLUDE_USER_SETTINGS` — keep or rename per this crate's `ACP_`-prefix convention, matching Phase 2's naming decision for consistency).
5. In `reconnect/mod.rs::create_fresh_session`, pass the new `_meta` object into the `session/new` request params (check `NewSessionRequest`'s exact field for arbitrary `_meta` — likely a `meta: Option<Meta>` field already used elsewhere in the crate for agent-session-id extraction, confirm same field can carry outbound custom data or if it's inbound-only, requiring a different SDK field).
6. In `manager_spawn.rs::spawn_connected_session`, after `connect_and_load_session` returns successfully, if `input.session_options.as_ref().and_then(|o| o.model.as_deref())` is non-empty: call `resolve_requested_model_id`/`assert_requested_model_supported` against the record's current model state, issue the model-setting RPC (confirm exact method — likely `set_session_config_option` with a model-designated config id, per `model_request.rs`'s contract), apply `current_model_id_from_set_model_response` to the record.
7. Port `clear_omitted_fresh_session_config_options`, `sync_advertised_model_state`/`apply_reconnected_model_state`, `remove_model_config_options` into `session/model_state.rs` (new functions, direct translation from the cited TS). Wire `clear_omitted_fresh_session_config_options` into `create_fresh_session` (Requirement 5) and `apply_reconnected_model_state` into `connect_and_load_session`'s tail (after acquisition, before returning `ConnectAndLoadSessionResult` — this requires extending `Acquired`/`ConnectAndLoadSessionResult` to carry `config_options_present`/`legacy_model_metadata_present` flags, derived from each response's `.config_options`/`.meta`).
8. Wire `model_state_from_session_response` (gap 23) into the same response-handling code path added in Step 7 — the `legacy_model_metadata_present` flag both Step 7 and gap 23 need is the same boolean, compute once.
9. For gap 16: add `observed_session_updates`/`processed_session_updates` counters (or equivalent) to `ConnectedSession`; add a suppression toggle consulted by wherever `session/update` notifications are currently forwarded from the transport to the live consumer. In `acquire_via_rpc`'s load branch, set suppression before calling `session_load`, clear it after the drain condition (per this phase's ADR) is satisfied.
10. Unit tests: `build_claude_code_options_meta`'s field mapping (pure function, easy to test in isolation); `clear_omitted_fresh_session_config_options`/`apply_reconnected_model_state`/`remove_model_config_options` (pure logic, test each branch); `maybe_wrap_session_control_error`'s call-site integration (verify the wrapped message actually reaches the caller, not just that the function itself works — it's already unit-tested in isolation).
11. **Real call-path integration tests** (required — several of these gaps are "orphaned function" fixes): (a) spawn the fake agent, close a session with discard=true, confirm (via the fake agent logging received RPCs) that `session/close` was actually sent; (b) spawn with a fake-agent capability profile that does NOT advertise `session_capabilities.close`, confirm `close()` does NOT attempt the RPC and doesn't error; (c) set an invalid mode via `set_session_mode` against a fake agent configured to reject it, confirm the resulting error message reflects `maybe_wrap_session_control_error`'s wrapping, not the raw JSON-RPC error; (d) create a session with `session_options.model` set, confirm (via fake-agent RPC logging) the model-setting RPC was actually sent and the record's current-model-id reflects the response; (e) create a fresh session where the fake agent's `session/new` response omits `config_options` after an earlier connection had some — confirm the stale value is cleared; (f) load a session with the fake agent's `_meta.models` legacy shape (extend the fixture if needed) — confirm `model_state_from_session_response`'s legacy fallback actually populates model state.
12. `cargo fmt -p boltz-acpx`, `cargo check -p boltz-acpx --all-targets --features test-support`, `cargo test -p boltz-acpx --features test-support`, `make check-all`.
13. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md` per plan.md's housekeeping (gaps 9, 12, 15, 16).

## Todo list

- [ ] `AcpClient::session_close`.
- [ ] `close()` wired to send `session/close`, capability-gated, discard-gated.
- [ ] `maybe_wrap_session_control_error` wired into `connected_session.rs`.
- [ ] `build_claude_code_options_meta`, wired into `create_fresh_session`'s `session/new` params.
- [ ] Model application wired into `manager_spawn.rs` after session creation.
- [ ] `clear_omitted_fresh_session_config_options`, `apply_reconnected_model_state`, `remove_model_config_options` ported + wired.
- [ ] `model_state_from_session_response` wired into reconnect's response-handling.
- [ ] Session-update counters + suppression toggle on `ConnectedSession`; load-path drain wired in.
- [ ] Unit tests for each new/wired pure function.
- [ ] Integration tests (a)-(f) against the real fake-agent binary.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green.
- [ ] Correct original plan's Phase 4 status text (gaps 9, 12, 15, 16).

## Success Criteria

- Fake-agent RPC log confirms `session/close` is sent exactly when discard=true AND the capability is advertised, and never otherwise.
- A rejected `set_session_mode` call surfaces a message distinguishably different from the raw JSON-RPC error (contains the "for mode ..." context acpx adds), provable by string-matching the returned error in a test.
- A session created with `session_options.model` set ends up with the record's current-model-id matching what the fake agent's mocked response designates — not just that a config-option RPC was sent, but that the *result* was applied.
- A fresh-session-after-earlier-connection scenario proves stale `config_options` are cleared, not carried over.
- `cargo test -p boltz-acpx --features test-support` count grows, all green.

## Risk Assessment

- **`Acquired`/`ConnectAndLoadSessionResult` struct extension** (Step 7) changes an internal type Phase 2 just finished testing — re-run Phase 2's full test suite after this change to confirm no behavioral regression, not just a compile-pass.
- **Model-application ordering** (CONFIRMED, plan.md Unresolved Questions #4): apply unconditionally, matching acpx, including for resumed sessions. If implementation reveals an unexpected test failure suggesting the resumed case needs different handling, treat that as a new bug report, not a reopening of this decision.
- **`_meta` field collision risk**: if `NewSessionRequest`'s meta field is used for something else in this crate already (e.g. Devin identity spoofing per the original Phase 2's implementation notes), the new Claude Code options meta must be merged, not overwritten — check `handshake.rs`'s Devin-identity meta-construction code (if any) before assuming a clean slate.

## Security Considerations

- `session/close` failures must be handled gracefully (swallow resource-not-found, matching acpx) — a close failure must not leave the local session state in an inconsistent "neither open nor closed" condition; the existing local-cleanup logic in `close()` should still run regardless of the RPC outcome (best-effort RPC, guaranteed local cleanup).
- No new untrusted-input surface — all new code paths operate on already-validated internal state (persisted `SessionAgentOptions`, already-authenticated agent responses).

## Next steps

- Proceed to [Phase 5](./phase-05-runtime-contract-dynamism.md), [Phase 6](./phase-06-conversation-fidelity-client-operations.md), [Phase 7](./phase-07-windows-batch-shell-spawn.md) in parallel (file-disjoint, verified in plan.md's matrix) once this phase merges.
- plan.md Unresolved Questions #4 (model-application timing) is confirmed — apply unconditionally per Requirement 4/Step 6; no further review gate.
