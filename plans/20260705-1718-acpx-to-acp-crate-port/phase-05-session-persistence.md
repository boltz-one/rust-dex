# Phase 5: Session Persistence (Versioned Serde Format)

## Context links

- Plan: [plan.md](./plan.md)
- Depends on: [Phase 1](./phase-01-crate-scaffolding.md) (types/error skeletons only — can run in parallel with Phase 2/3)
- Consumed by: [Phase 4](./phase-04-runtime-engine-public-contract.md) (`AcpSessionStore`/`file-session-store` wiring)
- Research: [researcher-02-acpx-architecture.md](./research/researcher-02-acpx-architecture.md) §4

## Overview

- **Date:** 2026-07-05
- **Description:** Port the on-disk session record format, its forward/backward-compat versioning policy, the conversation model (message history with truncation limits), and the repository (atomic write, index, import/export). Establish the serde-based analog to acpx's `persisted-key-policy.ts` guarantees.
- **Priority:** P2 (does not block Phase 2/3's protocol work; blocks Phase 4's full reconnect-with-persistence tests and Phase 6's per-session queue persistence, if any)
- **Implementation status:** Done
- **Review status:** Not reviewed

## Key Insights

- `session/conversation-model.ts` (941 lines) is the largest file in this phase's scope — larger than `session/persistence/parse.ts` (878 lines) and `manager.ts` combined-adjacent scope. It defines truncation limits (`MAX_RUNTIME_MESSAGES = 200`, 8000 chars agent text, 4000 chars thinking, 4000 chars tool I/O, 100 request-token-usage entries logged) and the clone/trim/record functions applied on every prompt/update. This is dense pure-data-transform logic — mechanical to port but large; budget accordingly.
- `persisted-key-policy.ts` (117 lines, read in full during planning) is a **validation-only** policy: it asserts that every persisted key is `snake_case` except an explicit allowlist of PascalCase "tag" keys (`User`, `Agent`, `Resume`, `Text`, `Mention`, `Image`, `Audio`, `Thinking`, `RedactedThinking`, `ToolUse` — acpx's internally-tagged enum discriminants) and an explicit set of "opaque" paths that skip descent entirely (raw agent capability blobs, tool-call raw input, desired/current config options). It does **not** itself define forward/backward compatibility — that's `session/persistence/parse.ts`'s job (defaults for missing optional fields, tolerate unknown fields).
- `session/persistence/repository.ts` (461 lines, read in full during planning) implements: atomic write (temp file + rename), an index file for fast listing avoiding a full directory scan, exact-id / suffix-id resolution with explicit ambiguity errors (`SessionResolutionError` when 2+ records match), directory-walk session discovery (find a session by walking up from cwd toward a git-repo-root boundary), and prune-by-age with optional history-file cleanup. All of this ports close to 1:1 — it's already well-factored pure I/O + list-filtering logic.
- The session record's top-level schema tag is a string literal constant: `SESSION_RECORD_SCHEMA = "acpx.session.v1"`. The Rust port needs its own namespaced tag (not literally `"acpx.session.v1"` — this is a new format, not byte-compatible with acpx's files) — see ADR-5 for the exact strategy.
- acpx's session storage root is `path.join(os.homedir(), ".acpx", "sessions")` — hardcoded to the user's home directory. The Rust port's equivalent location is an open question (see Unresolved Questions) since a GPUI desktop app may have its own app-data-directory convention distinct from a dotfile-in-home CLI convention.

## Requirements

1. `SessionRecord` (Rust struct, serde `Serialize`/`Deserialize`) mirrors acpx's field set: record id, ACP session id, agent session id, agent command, cwd, name, timestamps, event log pointer, closed/exit metadata, protocol version, agent capabilities snapshot, conversation (messages + token usage + cost), acpx-equivalent extra state (mode/model/config-option desired-vs-current), imported-from provenance.
2. Forward-compat: unknown top-level and nested fields in an on-disk record must not cause a parse failure — preserved on next write (round-trip fidelity), mirroring acpx's parse.ts tolerance.
3. Backward-compat: missing optional fields on an older-format record get sensible defaults on load, not a parse error.
4. Conversation model: message history capped at 200 runtime messages with the same per-field char limits acpx enforces (8000/4000/4000/100), applied consistently so persisted files don't grow unboundedly.
5. Repository: atomic write-to-temp-then-rename; an index for O(1)-ish listing; exact-id resolution, suffix-id resolution with explicit "ambiguous" error when multiple records match a suffix; directory-walk session discovery bounded by a git-repo-root (or explicit boundary).
6. Prune: age-based and/or agent-command-scoped pruning with dry-run support and byte-freed reporting.
7. Import/export: port `session/export.ts` (250 lines) and `session/import.ts` (330 lines) preserving the `importedFrom` provenance fields acpx tracks (source record id, original cwd, exporter identity/timestamp).

## Architecture

```
crates/acp/src/session/
├── model_state.rs        # SessionModelState (currentModelId/availableModelIds) — small
├── live_checkpoint.rs     # LiveSessionCheckpoint — small
├── config_options.rs      # applyConfigOptionsTo{Record,State} — small
├── event_log.rs           # SessionEventLog defaults — small
├── model_application.rs   # applyRequestedModelIfAdvertised, currentModelIdFromSetModelResponse
├── conversation_model/
│   ├── mod.rs             # public API: create/clone/trim/record functions
│   ├── limits.rs          # MAX_RUNTIME_MESSAGES=200 + per-field char-limit constants
│   └── trim.rs             # trimConversationForRuntime equivalent
├── export.rs
├── import.rs
├── mode_preference.rs      # desired mode/model/config-option get/set/sync helpers
├── events.rs               # session/events.ts — event-log-adjacent record events (distinct from
│                             # runtime/public/events.rs's live prompt events — verify naming doesn't
│                             # collide; consider `session_event_log_entries.rs` if it does)
├── persisted_key_policy.rs # snake_case validation + tag-key allowlist + opaque-path skip-list
└── persistence/
    ├── repository.rs       # write/resolve/list/find/prune/close — the 461-line primary source
    ├── parse.rs             # forward/backward-compat parse — the 878-line primary source
    ├── index.rs             # session index load/rebuild/write
    └── serialize.rs         # serializeSessionRecordForDisk equivalent (small, 51 lines)
```

## ADR Rationale

### ADR-5: Session persistence format & versioning strategy

- **Context:** acpx's on-disk format is a hand-validated JSON convention: a `schema` string tag (`"acpx.session.v1"`), a `persisted-key-policy.ts` runtime assertion enforcing snake_case keys (catching accidental camelCase drift from TS's default JSON.stringify of a camelCase object graph — the policy exists specifically because TS objects default to camelCase and the persisted format is deliberately snake_case, so the assertion is a lint-at-runtime safety net against future contributors forgetting to convert a new field), and `parse.ts`'s manual field-by-field forward/backward-compat logic (878 lines of it).
- **Decision:** Rust's `serde` gives most of this "for free" if used correctly, but the equivalent guarantees must be *deliberately* engineered, not assumed:
  - **Schema tag:** a `schema: SessionSchemaVersion` field (a Rust enum, not a bare string, so an unrecognized future version fails to deserialize *explicitly* rather than silently coercing) — e.g. `#[serde(rename = "boltz-acp.session.v1")]` tag on a v1 variant, ready for a v2 variant later via a manual `Deserialize` impl or a version-sniffing pre-pass (read as `serde_json::Value` first, branch on the `schema` field, then deserialize into the matching versioned struct) — mirrors acpx's `SESSION_RECORD_SCHEMA` constant but makes version dispatch a compile-time-checked enum match instead of a runtime string compare.
  - **Forward-compat (unknown fields):** every persisted struct carries `#[serde(flatten)] extra: serde_json::Map<String, serde_json::Value>` to capture and round-trip fields this version of the Rust struct doesn't know about — this is the direct structural analog of acpx's parse.ts "ignore unknown fields but don't drop them on next write" behavior, except serde does the capture/round-trip automatically once `flatten` is in place, versus acpx's manual per-field logic.
  - **Backward-compat (missing optional fields):** every field that acpx's `parse.ts` treats as optional-with-default gets `#[serde(default)]` in the Rust struct — again structural, not manual per-field `if (x == null) x = default` code.
  - **Snake_case enforcement:** since Rust struct fields are naturally snake_case and `serde` serializes field names as-written (no camelCase-by-default surprise the way TS/JS has), the *entire class of bug* `persisted-key-policy.ts` guards against does not exist in the Rust port by construction for ordinary struct fields. The policy is **not dead**, though: acpx's internally-tagged enum variants (`User`/`Agent`/`Resume`/`Text`/`Mention`/etc.) are deliberately PascalCase tag names inside an otherwise-snake_case document — port this as a `#[serde(tag = "type")]`-style internally-tagged Rust enum with explicit `#[serde(rename = "User")]` etc. per variant, and keep a **debug-only** `assert_persisted_key_policy`-equivalent test helper (not a runtime check in release builds — the guarantee is now structural, so this becomes a regression test against accidental future `#[serde(rename_all = "camelCase")]` additions, not a hot-path assertion).
- **Why this over alternatives:** (a) Porting `parse.ts`'s 878 lines of manual per-field default-filling line-by-line would be a straight DRY violation of what `serde`'s `#[serde(default)]`/`#[serde(flatten)]` already give the language — Rust's type system makes most of that file's *purpose* structurally redundant, though its *test cases* (what should default to what) remain essential input for writing the struct definitions correctly. (b) A bare `String` schema tag (matching acpx exactly) would let an unrecognized future schema version silently attempt to deserialize into the current struct and fail with a generic serde error instead of a clear "unsupported schema version" error — the enum-tag approach fails loudly and specifically instead.

## Related code files

- `others/acpx/src/persisted-key-policy.ts` (117 lines, read in full) — validation policy this ADR responds to.
- `others/acpx/src/session/persistence/repository.ts` (461 lines, read in full) — primary source.
- `others/acpx/src/session/persistence/parse.ts` (878 lines), `serialize.ts` (51 lines), `index.ts` (190 lines).
- `others/acpx/src/session/conversation-model.ts` (941 lines) — largest single source file in this phase.
- `others/acpx/src/session/{model-state,live-checkpoint,config-options,event-log,model-application,export,session,runtime-session-id,events,persistence,import,mode-preference}.ts`.
- `others/acpx/src/types.ts` — `SessionRecord`, `SessionConversation`, `SessionMessage`, `SessionUserContent`, `SessionAgentContent`, `SessionToolUse`, `SessionToolResult`, `SessionTokenUsage`, `SessionUsageCost`, `SessionAcpxState`, `SessionImportedFrom` type definitions (already read in full during planning) — direct struct-field source of truth for this phase.
- Consumed by: `crates/acp/src/runtime/public/contract.rs`'s `AcpSessionStore` trait (Phase 4).

## Implementation Steps

1. Transcribe every field of acpx's `SessionRecord` and its nested types (`SessionConversation`, `SessionMessage` variants, `SessionTokenUsage`, `SessionAcpxState`, `SessionImportedFrom`) from `types.ts` into Rust structs/enums first, before writing any repository logic — this is the schema Phase 4 and the repository both build on.
2. Apply ADR-5: add `#[serde(default)]` to every optional field, `#[serde(flatten)] extra: serde_json::Map<...>` on the top-level record and any nested struct acpx's `parse.ts` treats as independently-evolvable, internally-tagged enum for `SessionMessage`/`SessionUserContent`/`SessionAgentContent`/`SessionToolResultContent` matching the PascalCase tag names acpx's persisted-key-policy allowlists.
3. Port `conversation_model/limits.rs` (constants) and `conversation_model/trim.rs` (the actual truncation algorithm — get the exact truncation order right: which limit applies first when multiple are exceeded simultaneously, matching acpx's behavior, not an arbitrary reordering).
4. Port `conversation_model/mod.rs` (clone/create/record functions).
5. Port `mode_preference.rs`, `model_state.rs`, `model_application.rs`, `config_options.rs`, `live_checkpoint.rs`, `event_log.rs` (all small, mechanical).
6. Port `persistence/serialize.rs`, then `persistence/parse.rs` (write parse tests *first* against a handful of representative acpx JSON fixtures — export a few real session files from acpx if available, or hand-construct them from `types.ts`'s shape — to lock in exact field-name/casing expectations before the repository depends on parse being correct).
7. Port `persistence/index.rs`.
8. Port `persistence/repository.rs`: atomic write, resolve (exact/suffix/ambiguous), list/find/find-by-directory-walk, prune, close.
9. Port `export.rs`, `import.rs` preserving `importedFrom` provenance.
10. Write the debug-only persisted-key-policy regression test per ADR-5 (not a runtime assertion).
11. Decide the Rust port's session-storage root directory (Unresolved Question below) and implement it as a configurable `AcpFileSessionStoreOptions`-equivalent (`state_dir: PathBuf`), matching acpx's `AcpFileSessionStoreOptions.stateDir` shape rather than hardcoding a path — this makes the open question a runtime configuration point, not a blocking design decision, so Phase 5 can proceed without the answer being final.
12. Integration tests: round-trip a record through write→read and confirm byte-for-semantic equality (accounting for the `extra` flatten field being empty on a fresh round-trip); write a record with an injected unknown field (simulating a "future version wrote this"), confirm it survives a read-then-write cycle unchanged; write a record missing an optional field (simulating "older version wrote this"), confirm it loads with the documented default; test suffix-id ambiguity error; test prune dry-run vs. real.
13. `cargo fmt`, `cargo check -p boltz-acp`, `make check-all`.

## Todo list

- [x] Transcribe `SessionRecord` and nested types from `types.ts` into Rust structs/enums.
- [x] Apply `#[serde(default)]`/`#[serde(flatten)]`/internally-tagged-enum pattern per ADR-5.
- [x] Port `conversation_model/{limits,trim,mod}.rs`.
- [x] Port `mode_preference.rs`, `model_state.rs`, `model_application.rs`, `config_options.rs`, `live_checkpoint.rs`, `event_log.rs`.
- [x] Port `persistence/{serialize,parse,index,repository}.rs`.
- [x] Port `export.rs`, `import.rs`.
- [x] Write debug-only persisted-key-policy regression test.
- [x] Decide + implement configurable session-storage root (`state_dir`).
- [x] Round-trip / forward-compat / backward-compat / ambiguity / prune integration tests.
- [x] All new files < 200 lines (`conversation-model.ts` 941 lines and `persistence/parse.ts` 878 lines needed aggressive splitting — verified: largest file post-port is 198 lines; see the "Implementation notes" section below for the final file layout, which splits several files further than the Architecture section's original suggestion).
- [x] `cargo check -p boltz-acp`, `make check-all`, `cargo fmt --all -- --check` green (the workspace-wide `fmt-check` currently fails only in `crates/ui/...palette.rs`, owned by a concurrent, unrelated session — not this phase's files; `cargo fmt -p boltz-acp -- --check` is clean).

## Implementation notes / deviations

- **`conversation_model`/`persistence::repository`/`export`/`import`/`events` split further than the Architecture doc's suggested layout** to keep every file under this crate's 200-line convention (the doc's layout under-estimated post-port line counts once doc comments + tests were added). New submodules not named in the original Architecture section: `conversation_model::{agent_content, tool_use}`, `persistence::repository::{find, resolve, write, prune, close}` (a directory, not a single file), `export::{lookup, archive}` (a directory), `import::{archive_parse, agent_match, build}` (a directory), `events::{lock, rotate, writer}` (a directory).
- **Live ACP-protocol types intentionally not referenced** in `conversation_model`'s record/session-update functions or `model_application.rs`'s `applyRequestedModelIfAdvertised` half, per the plan's explicit "Phase 5 only depends on Phase 1" constraint. `InboundContent`/`ToolCallUpdateInput`/`SessionUpdateInput` are protocol-crate-agnostic stand-ins for Phase 4 to populate from the real `agent_client_protocol` types. `current_model_id_from_set_model_response` (the pure half) is ported; the live-`AcpClient`-calling half is deferred to Phase 4.
- **`appendLegacyHistory`/`LegacyHistoryEntry`** (acpx's pre-acpx-format migration helper) not ported — no predecessor on-disk format exists for this port to migrate from (YAGNI).
- **`session/session.ts`** (a barrel re-exporting acpx's CLI-only `cli/session/*` surface) not ported — this crate has no CLI.
- **JSON-RPC history entries** in `export.rs`/`import.rs`/`events.rs` are opaque `serde_json::Value`s rather than acpx's typed `AcpJsonRpcMessage` (`isAcpJsonRpcMessage` lives in Phase 2's `acp/jsonrpc.ts` equivalent) — avoids a dependency this phase shouldn't have while preserving forward-compat.
- **`import.rs`'s agent-identity matching** uses this crate's own `agent_command::normalize_agent_name` instead of acpx's per-agent npm-package-name regexes (`@agentclientprotocol/codex-acp`, etc.), which don't apply to this port's command-resolution conventions.
- **`SessionAcpxState.config_options`/`SessionRecord.agent_capabilities`** use the real `agent_client_protocol::schema::v1::{SessionConfigOption, AgentCapabilities}` types directly (matching acpx's own `types.ts`, which imports these from the ACP SDK) rather than a locally re-derived shape.
- **`persisted_key_policy.rs`** is wired into `write_session_record` only under `#[cfg(debug_assertions)]` (panics on violation), plus a dedicated regression test — per ADR-5, the guarantee is structural in Rust, so this is a regression test against an accidental future `#[serde(rename_all = "camelCase")]`, not a release-mode hot-path assertion.
- **State-dir default:** `AcpFileSessionStoreOptions::default()` uses `dirs::state_dir().or_else(dirs::data_dir).unwrap_or(temp_dir)` joined with `"boltz-acp"` (sessions live under `<state_dir>/sessions`, matching the shape Phase 4's `file-session-store.ts` equivalent expects) — a reasonable, distinctly-named default per the phase's Unresolved Question #7, not a final decision.
- **Repository/events I/O is synchronous `std::fs`**, not threaded through `smol`'s async I/O — this class of operation (small, infrequent, local-disk metadata/log writes) doesn't need async, and keeps this phase self-contained without requiring Phase 2's async transport conventions. `live_checkpoint.rs` (the one genuinely async, debounced piece) does use `smol::spawn`/`smol::Timer` per ADR-2.

## Success Criteria

- A record written by this phase's code, with a hand-injected unknown top-level field (simulating a future schema addition), survives a read→write round-trip with that field intact and unchanged.
- A record missing an optional field acpx treats as defaultable loads without error and matches acpx's documented default value.
- Suffix-id resolution against 2 records sharing a suffix returns an explicit ambiguity error, never a silently-wrong match.
- Conversation trimming applied to a 250-message synthetic conversation caps at 200 messages and matches acpx's char-limit truncation on at least one oversized agent-text message, verified by test.

## Risk Assessment

- **`conversation-model.ts` (941 lines) truncation-order bugs:** if multiple limits are exceeded simultaneously (e.g. both message count and per-message char count), truncation order matters for which content survives. Get this from the TS source exactly, don't infer an order.
- **Schema-version dispatch cost:** the "sniff schema field first, then deserialize into the matching versioned struct" approach (ADR-5) means every load does a two-pass parse (once as generic `Value`, once as the typed struct). For a session-file-sized document this is negligible; flag only in case a future perf-sensitive path (e.g. listing hundreds of sessions) needs the index (`persistence/index.rs`) to avoid this by design — which it already does, since the index stores summary fields separately from full records.
- **State-dir decision deferred:** shipping with a wrong-but-configurable default is low-risk (it's a config point, not baked into the wire format), but picking a *bad* default (e.g. colliding with acpx's own `~/.acpx/sessions` if a user has both installed) could cause confusing cross-tool interference. Pick a distinctly-named default (not `.acpx`).

## Security Considerations

- Atomic write (temp-file + rename) must use a temp file in the *same directory* as the final destination (not a shared system temp dir) to guarantee the rename is atomic (cross-filesystem renames are not atomic) — verify acpx's `${file}.${pid}.${timestamp}.tmp` pattern (same-directory sibling file) is preserved exactly.
- Session record files may contain conversation content with sensitive user data (prompts, tool outputs, auth-adjacent env values if a tool call echoed them) — file permissions on the session storage directory should be user-only (0700/0600-equivalent on Unix; confirm Windows ACL equivalent isn't silently more permissive) — acpx doesn't appear to set explicit permissions beyond OS defaults; decide whether this port tightens that or matches acpx's current (looser) behavior, and document the choice.
- Import (`import.rs`) accepts a record from an external source (potentially another machine/user) — validate the imported record against the same schema/parse path as a locally-written one, never trust import-time fields (like `cwd`) without re-validating them against the importing machine's actual filesystem state.

## Next steps

- Hand off `AcpSessionStore` trait implementation to [Phase 4](./phase-04-runtime-engine-public-contract.md) once `persistence::repository` is stable.
- Unresolved question carried forward: **session state directory** — final location/name for the Rust port's session storage root (e.g. `dirs::state_dir()`-based path vs. app-provided path via `AcpFileSessionStoreOptions`). Get user input; Step 11 makes this a runtime config point so the answer doesn't block implementation, only the shipped default.
