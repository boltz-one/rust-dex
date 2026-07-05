# Phase 9: Test Coverage, Legacy Migration Fidelity, Liveness Cleanup, Architecture ADR

## Context links

- Plan: [plan.md](./plan.md)
- Research: dedicated research pass (Windows spawn, Qoder, misc LOW gaps group) + reconnect research pass (liveness.rs findings)
- Original port plan phase to correct after this lands: [phase-05-session-persistence.md](../20260705-1718-acpx-to-acp-crate-port/phase-05-session-persistence.md) (gaps 30,31), [phase-06-prompt-queueing-cancellation.md](../20260705-1718-acpx-to-acp-crate-port/phase-06-prompt-queueing-cancellation.md) (gaps 32,33 — note: gap 32 is actually in `permissions/`, ported in the original Phase 3, not Phase 6; gap 33 is in `runtime/engine/reconnect/`, ported in the original Phase 4 — see this phase's housekeeping step for the correction to the user's suggested original-phase mapping), [phase-04-runtime-engine-public-contract.md](../20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md) (gaps 33, 35)
- Depends on: all MEDIUM-tier phases (4,5,6,7) merged
- Parallel: [Phase 8](./phase-08-agent-quirks-shutdown-persistence.md) (file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/session/persistence/parse.rs`, `crates/acp/src/session/model_state.rs`, `crates/acp/src/session/conversation_model/trim.rs` (doc comment only — no logic change, this file's logic was already finalized in Phases 3 and 6), `crates/acp/src/permissions/resolve_tests.rs`, `crates/acp/src/runtime/engine/reconnect/liveness.rs`, one new `docs/decisions/0010-connected-session-long-lived-client.md`. No other files.

## Overview

- **Priority:** P3 (LOW — mix of test-only additions, a documentation-fidelity note, and one small cleanup decision)
- **Status:** pending
- **Description:** 5 gaps: (30) `model_control` legacy-migration default isn't backfilled at parse time (only compensated for at read time elsewhere); (31) `trim_runtime_text` counts Unicode scalar values, not UTF-16 code units like acpx — likely accept-as-is; (32) `permissions/resolve.rs`'s `PermissionPolicyAction::Approve` arm and `DenyAll`/`ApproveReads` policy-override modes lack direct tests; (33) `reconnect/liveness.rs`'s `stored_process_status` is fully implemented+tested but orphaned, duplicating `manager/mod.rs`'s own inline liveness check; (35) `ConnectedSession`'s long-lived-client-per-session architecture (a real, already-implemented, legitimate simplification vs. acpx's ephemeral-client-per-turn model) was never formally documented as an accepted architectural deviation.

## Correction to the task brief's original-phase-file mapping

The originating task brief suggested "Phase 6 for gap 32/33 if relevant" when mapping fixes back to the original 6-phase port plan's files. Verified against the actual original plan: gap 32 (`permissions/resolve_tests.rs`'s missing test cases) belongs to the original **Phase 3** (`phase-03-permissions-filesystem-terminal.md` — that phase owns all of `permissions/`), and gap 33 (`reconnect/liveness.rs`) belongs to the original **Phase 4** (`phase-04-runtime-engine-public-contract.md` — that phase owns `runtime/engine/reconnect/`). Neither belongs to the original Phase 6 (`queue/*`, `perf_metrics`), which this whole plan explicitly does not touch (per plan.md's "Already confirmed fine — do not touch" list). This phase's housekeeping step corrects both original files' status text accordingly, not Phase 6's.

## Key Insights (from research)

- **Gap 30**: `session/persistence/parse.rs` (91 lines) does a schema-tag sniff + `serde_json::from_value` with zero `model_control`/`available_models`-specific logic — all backward-compat defaulting is delegated to `#[serde(default)]` (per ADR-5 from the original Phase 5). acpx's `parse.ts::assignParsedModelState` explicitly backfills `state.model_control` (`config_option` vs `legacy_set_model`) whenever it's absent but `available_models` is present, **mutating the parsed object itself** so every downstream consumer (including re-serialization) sees the backfilled value. Rust's `session/model_state.rs::legacy_model_state`/`advertised_model_state` reconstruct the equivalent state **on-demand at read time**, but never persist the backfill onto `SessionAcpxState` itself — meaning a round-trip (load → don't touch model state → save) would NOT carry the backfilled `model_control` forward the way acpx's mutate-on-parse approach does, since Rust never writes it back.
- **Gap 31**: `trim_runtime_text` (`trim.rs:20-30`) uses `.chars().count()`/`.chars().take()` (Unicode scalar values). acpx's `.slice(0, max-3)` operates on UTF-16 code units — divergence only for text with astral-plane characters (e.g. some emoji, rare CJK extension characters) near a truncation boundary. The existing doc comment claims exact parity with acpx — this claim is only true for BMP-only text and should be corrected regardless of whether the counting method itself changes.
- **Gap 32**: `permissions/resolve_tests.rs` has 9 tests covering `ApproveAll`/`DenyAll`/`ApproveReads` *modes* and `Deny`/`Escalate` policy *actions*, plus policy-overrides-`ApproveAll`-mode. Missing: a test for `PermissionPolicyAction::Approve` (a matched policy rule resolving to auto-approve) at all, and no test proves policy-override ordering holds specifically against `DenyAll`/`ApproveReads` modes (only implicitly exercised against `ApproveAll`). This is pure test-writing — the underlying `resolve.rs` logic (`policy match → mode match` dispatch, L136 vs L96-99) is already correct and unchanged.
- **Gap 33**: `stored_process_status(record) -> StoredProcessStatus` (`liveness.rs`, `{NoPidRecorded, Alive, Dead}`) is diagnostic-only per its own module doc — `manager/mod.rs:100` has its own separate inline check (`connected.client.state().last_known_pid.is_some_and(is_process_alive)`) that doesn't use this module's enum or function at all. This is a genuine "two implementations of the same concept" situation, not a missing-feature gap — the decision is whether to (a) make `manager/mod.rs` consume `liveness::stored_process_status` instead of its own inline check (unifying on one implementation), or (b) delete `liveness.rs` as genuinely redundant. Per YAGNI, since `manager/mod.rs`'s inline check already works and is tested (implicitly, via the existing reconnect tests), and `liveness.rs`'s 3-state enum (`NoPidRecorded` vs `Dead` distinction) carries slightly richer diagnostic information the inline boolean check doesn't — recommend (a), consuming it for the richer diagnostic, over (b) deleting working code.
- **Gap 35**: no code change — a documentation-only fix. `docs/decisions/` uses a consistent format (`# NNNN. Title` → `Status`/`Date`/`Lane` → `Context`/`Decision`/`Alternatives Considered`, per `0002-async-substrate-smol.md`'s structure). Next available number: `0010` (0007-0009 are reserved for an unrelated concurrent workstream per this plan's scope boundary).

## Requirements

1. `session/persistence/parse.rs` (or `model_state.rs`, whichever is the more natural home for a parse-time mutation — confirm during implementation) backfills `SessionAcpxState.model_control` at parse time when absent but `available_models` is present, matching acpx's `assignParsedModelState` exactly (`config_option` if a model-designated config option exists among `config_options`, else `legacy_set_model`).
2. `trim_runtime_text`'s doc comment is corrected to note the Unicode-scalar-vs-UTF-16-code-unit distinction and that exact parity only holds for BMP-only text — no logic change (accept-as-is per the task's framing, this is a documentation-fidelity fix, not a behavior fix, unless implementation reveals a trivial fix is available, in which case prefer fixing it).
3. `permissions/resolve_tests.rs` gains: a test exercising `PermissionPolicyAction::Approve` end-to-end (a policy rule matches and resolves to auto-approve, independent of mode); a test proving policy-override ordering against `DenyAll` mode (a policy `Approve` rule overrides an otherwise-`DenyAll`-rejected request); a test proving the same against `ApproveReads` mode (a policy `Deny` rule overrides an otherwise-auto-approved read).
4. `manager/mod.rs`'s inline liveness check is replaced with a call to `reconnect::liveness::stored_process_status`, using its richer 3-state result where the calling code can make use of the `NoPidRecorded` vs `Dead` distinction (e.g. differentiated log messages), or at minimum mapped to the same boolean the inline check currently produces if no richer consumer exists yet.
5. New `docs/decisions/0010-connected-session-long-lived-client.md` documents `ConnectedSession`'s long-lived-client-per-session architecture as an accepted, deliberate deviation from acpx's ephemeral-client-per-turn + connection-pooling model, following the existing ADR format (`0002-async-substrate-smol.md`'s structure: Status/Date/Lane header, Context/Decision/Alternatives Considered sections).

## Architecture

```
crates/acp/src/
├── session/
│   ├── persistence/parse.rs   # + model_control backfill at parse time (or model_state.rs,
│   │                            #   TBD at implementation — whichever avoids duplicating the
│   │                            #   "has a model-designated config option" check already in
│   │                            #   model_state.rs)
│   └── model_state.rs          # (possible location for the backfill logic, see above)
├── permissions/resolve_tests.rs  # + 3 new tests (Approve arm, DenyAll override, ApproveReads override)
└── runtime/engine/reconnect/
    ├── mod.rs                  # (this phase's only edit here, if any) — manager/mod.rs consumes
    │                             #  liveness::stored_process_status instead of its own inline check
    │                             #  (the actual replaced call site lives in manager/mod.rs, which is
    │                             #  OUTSIDE this phase's declared scope boundary — see Risk Assessment)
    └── liveness.rs              # no logic change — already correct, consumed by this phase's fix

docs/decisions/0010-connected-session-long-lived-client.md   # NEW
```

## ADR Rationale

### ADR-11: `ConnectedSession` long-lived-client-per-session — formal documentation (gap 35, no code change)

- **Context:** acpx's client model creates/tears down a client connection per turn (ephemeral) with a connection-pooling layer amortizing subprocess spawn cost. This Rust port's `ConnectedSession` keeps one live `AcpClient` alive for the lifetime of a session (until explicitly closed or reconnected), a simplification that was implemented across the original Phases 4/6 but never called out as a deliberate top-level architectural choice in its own ADR — it was an implicit consequence of how the manager/reconnect state machine was designed, not a named decision.
- **Decision:** Add `docs/decisions/0010-connected-session-long-lived-client.md` documenting: the context (acpx's ephemeral model vs. this port's long-lived model), the decision (long-lived client per session, reconnect-on-demand rather than reconnect-per-turn), and alternatives considered (matching acpx's ephemeral model exactly — rejected because a GUI's per-session client naturally maps to "as long as the user has this session/tab open," making per-turn teardown/respawn pure overhead with no corresponding benefit in a single-long-running-process host, unlike acpx's CLI-per-invocation model where the process itself is inherently short-lived).
- **Why an ADR now instead of earlier:** this audit's own premise is that undocumented decisions look identical to unintentional bugs from the outside — the same reasoning applies to an architectural simplification as to a field-level behavioral divergence. Documenting it now closes that specific gap without touching any code (the architecture itself is already correct and well-tested per the audit's own "confirmed fine" list).

## Related code files

- `crates/acp/src/session/persistence/parse.rs` (full 91-line file).
- `crates/acp/src/session/model_state.rs` (`legacy_model_state` L18-31, `advertised_model_state` L45 — read in full to find the "has a model-designated config option" check to reuse, avoiding duplicating it in the parse-time backfill).
- `crates/acp/src/session/conversation_model/trim.rs` (`trim_runtime_text`, L20-30 — doc comment only).
- `crates/acp/src/permissions/resolve_tests.rs` (9 existing tests, listed in this plan's research — read in full before adding new ones to match the existing test-naming/structure convention).
- `crates/acp/src/permissions/resolve.rs` (`PermissionPolicyAction` dispatch L136-146, mode dispatch L96-99 — read only, no logic change).
- `crates/acp/src/runtime/engine/reconnect/liveness.rs` (`stored_process_status` L32-38, `StoredProcessStatus` L24-28).
- `crates/acp/src/runtime/engine/manager/mod.rs` (L96-100, the inline `is_process_alive` check — **this file is outside this phase's declared scope boundary**; see Risk Assessment for how to handle this).
- `docs/decisions/0002-async-substrate-smol.md` (format reference).
- Reference (read-only): `others/acpx/src/session/persistence/parse.ts` (`assignParsedModelState` L433-450).

## Implementation Steps

1. Read `session/model_state.rs` in full to find (or confirm the absence of) an existing "does this config-options list include a model-designated option" predicate — reuse it for the parse-time backfill rather than writing a second copy.
2. Add the `model_control` backfill to `parse.rs` (or call out to a function added in `model_state.rs` if the check needs data/imports not available in `parse.rs` without a new dependency edge — decide based on what keeps the change smallest and most local).
3. Correct `trim_runtime_text`'s doc comment (Requirement 2) — read the current comment's exact wording first, replace only the incorrect "matches acpx exactly" claim, keep the rest.
4. Read `permissions/resolve_tests.rs` in full, confirm the existing test-naming convention (e.g. `snake_case_describing_scenario`), add the 3 new tests per Requirement 3, following that convention.
5. Read `runtime/engine/manager/mod.rs`'s exact inline liveness check (L96-100) to confirm precisely what it currently does and how its boolean result is consumed downstream (log message? decision branch?) — this file is outside this phase's declared file-ownership scope boundary (only `crates/acp/src/runtime/engine/reconnect/liveness.rs` is listed), so implementing Requirement 4 requires either (a) a scope-boundary exception for this one call site in `manager/mod.rs`, or (b) deferring Requirement 4 to a future pass and documenting it as such. Given the change is a small, low-risk swap of one inline expression for a function call, prefer (a) with a single-line, tightly-scoped edit — document this exception explicitly in this phase's Implementation status once the edit is made.
6. Write `docs/decisions/0010-connected-session-long-lived-client.md` following `0002`'s exact section structure.
7. Unit tests for Step 2's backfill: a hand-constructed record JSON with `available_models` present, `model_control` absent, and a config-options list that does/doesn't include a model-designated option — assert the backfilled value matches acpx's exact branch logic in both cases; a round-trip test (parse → serialize → parse again) confirming the backfilled value persists (unlike today's read-time-only reconstruction).
8. `cargo fmt -p boltz-acp`, `cargo check -p boltz-acp --all-targets --features test-support`, `cargo test -p boltz-acp --features test-support`, `make check-all`.
9. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-05-session-persistence.md` (gaps 30, 31) and `phase-03-permissions-filesystem-terminal.md` (gap 32) and `phase-04-runtime-engine-public-contract.md` (gaps 33, 35) per this plan's housekeeping — note this corrects the task brief's suggested "Phase 6" mapping for gaps 32/33 to the actually-correct Phase 3/Phase 4 (see this file's "Correction" section above).

## Todo list

- [ ] `model_control` backfill at parse time, matching acpx's exact branch logic.
- [ ] `trim_runtime_text`'s doc comment corrected (Unicode scalar vs. UTF-16 caveat).
- [ ] `permissions/resolve_tests.rs`: `Approve` arm test, `DenyAll`-override test, `ApproveReads`-override test.
- [ ] `manager/mod.rs`'s inline liveness check replaced with `stored_process_status` (scope exception documented).
- [ ] `docs/decisions/0010-connected-session-long-lived-client.md` written.
- [ ] Unit tests: backfill correctness (both branches) + round-trip persistence.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green.
- [ ] Correct original plan's Phase 3, Phase 4, and Phase 5 status text (with the Phase 6→Phase 3/4 mapping correction noted).

## Success Criteria

- A record parsed with `available_models` present and `model_control` absent gets the correct backfilled value (`config_option` or `legacy_set_model`) matching acpx's branch logic, and that value survives a parse→serialize→parse round-trip.
- All 3 new `permissions/resolve_tests.rs` tests pass and genuinely exercise the previously-untested code paths (verified by temporarily reverting the relevant `resolve.rs` logic and confirming the new tests fail — a sanity check that they're not vacuously passing).
- `manager/mod.rs`'s liveness check produces the same or richer diagnostic information as before, with no behavior regression in the existing reconnect integration tests from Phase 2.
- `docs/decisions/0010-connected-session-long-lived-client.md` exists and follows the established ADR format.

## Risk Assessment

- **Scope-boundary exception for `manager/mod.rs`** (Requirement 4/Step 5) — explicitly flagged rather than silently done; if a stricter reading of the scope boundary is preferred, this single item can be deferred to a future pass without blocking the rest of this phase (it's the only requirement in this phase touching a file outside the declared boundary).
- **Parse-time backfill mutating persisted state** could interact with the debug-only `persisted_key_policy` regression test from the original Phase 5 if the backfilled field's key naming isn't already snake_case-compliant — verify no new violation is introduced.

## Security Considerations

- No new untrusted-input surface in any of this phase's 5 gaps — all are either test-only additions, a documentation correction, an internal-diagnostic-consolidation, or a parse-time computation over already-validated (post-schema-check) data.

## Next steps

- This is the last phase in the plan's suggested ordering. Once complete, all 35 audited gaps are addressed and every original port-plan phase file's status text accurately reflects the real, now-corrected completion state.
- No unresolved questions specific to this phase.
