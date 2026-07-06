# Phase 3: Conversation Trim Determinism (IndexMap) + Import Agent-Match Security Check

## Context links

- Plan: [plan.md](./plan.md)
- Research: [researcher-01](./research/researcher-01-high-priority-verification.md)
- Original port plan phase to correct after this lands: [phase-05-session-persistence.md](../20260705-1718-acpx-to-acp-crate-port/phase-05-session-persistence.md) (gaps 7,8)
- Parallel: [Phase 1](./phase-01-permission-policy-authenticate-wiring.md), [Phase 2](./phase-02-reconnect-hardening-claude-timeout.md) (file-disjoint)

## Scope boundary

Only touch: root `Cargo.toml` (add `indexmap.workspace = true` line to `crates/acp/Cargo.toml`'s dependencies — do NOT touch the root `[workspace.dependencies]` `indexmap` entry itself, it already exists at line 144), `crates/acp/Cargo.toml`, `crates/acp/src/session/conversation_model/{trim.rs,conversation.rs}`, `crates/acp/src/session/record.rs`, `crates/acp/src/session/import/agent_match.rs`. No other files.

## Overview

- **Priority:** P1 (HIGH — data-integrity gap for gap 7; trust-boundary security check for gap 8)
- **Status:** pending
- **Description:** Two small, independent, file-scoped fixes bundled into one phase because both are: (a) mechanical (no new logic design needed, just applying already-known-correct behavior), (b) small enough individually to not warrant a dedicated phase, (c) low-risk (no reconnect/permission-adjacent blast radius).

## Key Insights (from verification research)

- **Gap 7**: `conversation_model/trim.rs:43-56`'s own code comment already admits the bug: `std::collections::HashMap::drain().take(N)` keeps an *arbitrary* N entries, not the N most-recently-inserted, because `HashMap` has no stable iteration order. acpx's `Object.entries(...).slice(-100)` relies on JS's object-insertion-order guarantee. `request_token_usage`'s field type (`HashMap<String, SessionTokenUsage>`) is duplicated in **two** structs: `conversation_model/conversation.rs:58` (`SessionConversation`) and `session/record.rs:87` (`SessionRecord`) — both must change type together, or a (de)serialization mismatch results.
- **`indexmap` is already a workspace dependency** (root `Cargo.toml:144`, `indexmap = { version = "2.7.0", features = ["serde"] }`) — used elsewhere in the workspace (e.g. `schemars`'s `indexmap2` feature) but **not yet added to `crates/acp/Cargo.toml`**, confirmed absent via direct grep. This is a `.workspace = true` addition, not a new external dependency to vet.
- **Gap 8**: `session/import/agent_match.rs`'s `assert_expected_agent_command` currently ANDs 2 of acpx's 3 conditions (`archive_command_matches && state_command_matches`), omitting `archiveAgentNameMatches(...)` entirely. The 3rd condition's TS logic (`others/acpx/src/session/import.ts`'s `archiveAgentNameMatches`) is: if both archive and state commands already exactly equal the expected command, short-circuit true; otherwise fall back to comparing normalized agent *names* (`archiveAgentName`/`expectedAgentName`), treating either being absent (`== null`) as a pass. The call site (`session/import/mod.rs:50`) already has the archive's `agent_name` field in scope (`archive.session.agent_name`, confirmed present on `ExportedSession`) — **no signature change needed at the call site**, the fix is entirely inside `agent_match.rs`.

## Requirements

1. `SessionConversation.request_token_usage` and `SessionRecord.request_token_usage` both change type from `HashMap<String, SessionTokenUsage>` to `IndexMap<String, SessionTokenUsage>`. Both structs' `Serialize`/`Deserialize` derives must continue to round-trip correctly (IndexMap's serde support, enabled via the `serde` feature already declared at workspace level, preserves key order — confirms forward/backward-compat with existing on-disk records, since JSON object field order is not semantically meaningful for a `HashMap`-shaped field on read, only on this crate's own re-write).
2. `trim.rs`'s eviction logic changes from `drain().take(N)` to an order-preserving "keep the last N inserted" operation using `IndexMap`'s API (e.g. compute `len - N` and call `.shift_remove_index(0)` in a loop, or use `.split_off(len - N)` if that method exists on the pinned `indexmap` version — confirm exact API at implementation time) — must produce the exact same *set* of retained entries as acpx's `Object.entries(...).slice(-N)` for a given insertion sequence.
3. `agent_match.rs::assert_expected_agent_command` adds the 3rd condition (`archive_agent_name_matches`), ANDing it with the existing 2, matching acpx's exact short-circuit + fallback logic (see Key Insights).

## Architecture

```
crates/acp/Cargo.toml
└── [dependencies] + indexmap.workspace = true

crates/acp/src/session/
├── conversation_model/
│   ├── conversation.rs   # request_token_usage: IndexMap<String, SessionTokenUsage>
│   └── trim.rs           # eviction rewritten for IndexMap order-preserving semantics
├── record.rs              # request_token_usage: IndexMap<String, SessionTokenUsage>
└── import/
    └── agent_match.rs     # + archive_agent_name_matches() helper, wired into
                             # assert_expected_agent_command's AND chain
```

## ADR Rationale

No cross-phase ADR needed — both fixes are direct 1:1 behavioral corrections to match already-specified, already-audited acpx behavior (no new design surface, no alternatives to weigh). Documented here per the plan's "ADR Rationale section for non-trivial decisions" requirement by explicitly noting **why no ADR is needed**: gap 7 is a data-structure substitution with one obviously-correct answer (order-preserving map), and gap 8 is closing a security gap by porting a specification that already exists verbatim in acpx — there is no legitimate alternative design to weigh for either.

## Related code files

- `Cargo.toml` (root, L144 — read-only, confirms `indexmap` version/features, do not edit this line).
- `crates/acp/Cargo.toml` (add one new dependency line).
- `crates/acp/src/session/conversation_model/trim.rs` (L43-56 eviction block).
- `crates/acp/src/session/conversation_model/conversation.rs` (L47-59 `SessionConversation` struct, L58 the field).
- `crates/acp/src/session/record.rs` (L87 the field).
- `crates/acp/src/session/import/agent_match.rs` (L38-65 `assert_expected_agent_command`, full 66-line file).
- `crates/acp/src/session/import/mod.rs` (L50 call site — read-only, confirm `archive.session.agent_name` is already in scope, no signature change expected).
- Reference (read-only): `others/acpx/src/session/conversation-model.ts:872-887` (`trimConversationForRuntime`), `others/acpx/src/session/import.ts:131-199` (`assertExpectedAgentCommand` + `archiveAgentNameMatches`).

## Implementation Steps

1. Add `indexmap.workspace = true` to `crates/acp/Cargo.toml`'s `[dependencies]` section.
2. Change `SessionConversation.request_token_usage` (`conversation.rs:58`) and `SessionRecord.request_token_usage` (`record.rs:87`) from `HashMap<String, SessionTokenUsage>` to `indexmap::IndexMap<String, SessionTokenUsage>`. Update any other code that constructs/iterates this field (grep for `request_token_usage` across the crate to find all touch points — expect a handful in conversation-model record/update functions and persistence serialize/parse code; `IndexMap` is API-compatible enough with `HashMap` for `.insert`/`.get`/iteration that most call sites should be untouched, but verify).
3. Rewrite `trim.rs`'s eviction block: if `len > MAX_RUNTIME_REQUEST_TOKEN_USAGE`, remove entries from the *front* (oldest-inserted) until `len == MAX_RUNTIME_REQUEST_TOKEN_USAGE`, using whichever `IndexMap` method is idiomatic on the pinned version (`shift_remove_index(0)` in a loop is correct but O(n) per removal — if a bulk "keep last N" method exists, prefer it; document the choice). Preserve the existing doc comment's intent but update it to note the fix (remove the "this port approximates it" caveat, replace with "matches acpx's `slice(-N)` exactly via IndexMap's insertion-order guarantee").
4. In `agent_match.rs`, add `fn archive_agent_name_matches(archive_agent_name: Option<&str>, expected_agent_name: Option<&str>, archive_command: &str, state_command: &str, expected_agent_command: &str) -> bool` — port acpx's exact logic: `if archive_command == expected_agent_command && state_command == expected_agent_command { return true; }` then `archive_agent_name.is_none() || expected_agent_name.is_none() || archive_agent_name == expected_agent_name`. Wire into `assert_expected_agent_command`'s condition, ANDing with the existing 2 (need `archive.session.agent_name` and `import_options.expected_agent_name` passed in — confirm exact field/param names against the call site in `mod.rs:50`, which already has both in scope).
5. Unit tests: `trim.rs` — a synthetic conversation with >100 `request_token_usage` entries inserted in a known order, assert the retained set is exactly the last 100 inserted (order-sensitive assertion, not just count). `agent_match.rs` — the exact acpx test-spec-shaped cases: (a) archive+state commands both match expected but names differ → currently-missing check now rejects (this is the security-relevant regression test — a malicious/malformed archive with a matching command string but a spoofed differing agent name must be rejected when name mismatch matters per the short-circuit rule), (b) commands match exactly on both sides → passes regardless of names (short-circuit), (c) one side's name is `None` → passes (permissive fallback), (d) names present and equal → passes, (e) names present and different, commands don't both equal expected → rejected.
6. Real call-path check (lighter-weight than Phase 1/2's requirement since these aren't "wired-into-nothing" gaps but existing-code correctness fixes): run the existing `session/persistence` and `session/import` integration tests unmodified to confirm no regression, plus the new unit tests from Step 5.
7. `cargo fmt -p boltz-acpx`, `cargo check -p boltz-acpx --all-targets --features test-support`, `cargo test -p boltz-acpx --features test-support`, `make check-all`.
8. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-05-session-persistence.md` per plan.md's housekeeping (gaps 7, 8).

## Todo list

- [ ] Add `indexmap.workspace = true` to `crates/acp/Cargo.toml`.
- [ ] Change `request_token_usage`'s type in `conversation.rs` and `record.rs` to `IndexMap`.
- [ ] Rewrite `trim.rs`'s eviction to be order-preserving.
- [ ] Add `archive_agent_name_matches` to `agent_match.rs`, wire into the AND chain.
- [ ] Unit test: order-preserving eviction with a known insertion sequence.
- [ ] Unit tests: all 5 `archive_agent_name_matches` scenarios (including the security-relevant name-mismatch-rejection case).
- [ ] Confirm no regression in existing persistence/import tests.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green.
- [ ] Correct original plan's Phase 5 status text (gaps 7, 8).

## Success Criteria

- A test inserts 150 `request_token_usage` entries with distinguishable keys in a known order, triggers trim, and asserts the retained 100 are exactly entries 51-150 (by insertion order) — not merely "100 entries remain."
- A test constructs an import scenario where archive+state commands match the expected command but `archive_agent_name` differs from `expected_agent_name` (and the short-circuit condition does NOT apply, i.e. commands don't literally equal `expected_agent_command`) — asserts the import is rejected with the agent-mismatch error, proving the previously-missing check now actually rejects a case it didn't before.
- `cargo test -p boltz-acpx --features test-support` count grows by at least the new tests added, all green, no existing test broken by the `HashMap`→`IndexMap` type change.

## Risk Assessment

- **Serialization round-trip risk**: switching `HashMap` to `IndexMap` for a `#[serde]`-derived field must not change the on-disk JSON shape in a way that breaks reading old records written with `HashMap`'s (arbitrary) key order — since JSON object field order is not semantically checked on parse for a map-shaped field, this should be a non-issue, but verify with an explicit round-trip test reading a hand-written fixture with keys in a specific order.
- **Any other code depending on `HashMap`-specific API** (e.g. `.drain()` used elsewhere, or code relying on `HashMap`'s `Send`/`Sync`/`Default` impls in a way `IndexMap` doesn't satisfy identically) — grep thoroughly in Step 2 before assuming a drop-in type swap.
- **Gap 8's fix could reject previously-accepted imports** if any real-world export/import pair legitimately has mismatched agent names under a case the new check now catches — this is the intended, correct behavior per acpx's spec, but flag it in the phase's own test output/report as a deliberate behavior tightening, not a silent regression.

## Security Considerations

- Gap 8 IS the security fix — the entire point of this half of the phase. Verify against acpx's exact 3-condition logic, not an approximation, since this is explicitly called out in the original Phase 5's own Security Considerations section ("never trust import-time fields without re-validating") as a known-incomplete area.
- No new untrusted-input surface for gap 7 — purely an internal data-structure correctness fix.

## Next steps

- Proceed to [Phase 4](./phase-04-session-lifecycle-reconnect-model-state.md) (MEDIUM tier) once this phase and Phases 1/2 are merged.
- No unresolved questions specific to this phase.
