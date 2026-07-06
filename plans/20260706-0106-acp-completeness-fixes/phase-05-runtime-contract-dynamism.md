# Phase 5: Runtime Contract Dynamism — Per-Session Capabilities, Self-Describing Handle, Session Validation

## Context links

- Plan: [plan.md](./plan.md)
- Research: dedicated research pass (session/close, model application, handle_state, reconnect replay dead code group)
- Original port plan phase to correct after this lands: [phase-04-runtime-engine-public-contract.md](../20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md) (gaps 13,14,34)
- Parallel (once HIGH tier is merged): [Phase 4](./phase-04-session-lifecycle-reconnect-model-state.md), [Phase 6](./phase-06-conversation-fidelity-client-operations.md), [Phase 7](./phase-07-windows-batch-shell-spawn.md) (file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/runtime/engine/manager/mod.rs`, `crates/acp/src/runtime/engine/manager/status.rs`, `crates/acp/src/runtime/public/handle_state.rs`, `crates/acp/src/runtime/public/shared.rs`. No other files.

## Overview

- **Priority:** P2 (MEDIUM)
- **Status:** pending
- **Description:** Three small, related gaps in the runtime's public contract surface: `get_capabilities()` always returns a static list instead of reflecting a specific session's advertised config options (gap 13); `AcpRuntimeHandle`'s `runtime_session_name` is a raw session-key copy instead of the versioned opaque-encoded string the already-implemented `handle_state.rs` machinery exists to produce (gap 14); `ensure_session` doesn't validate blank `session_key`/`agent` before use (gap 34).

## Key Insights (from research)

- **Gap 13**: `manager/status.rs::get_capabilities(&self) -> AcpRuntimeCapabilities` takes no params — acpx's version (`runtime.ts:224-247`) takes `input?: {handle?: AcpRuntimeHandle}`, and when a handle is given, loads the session record and reads `record.acpx.config_options` to populate `configOptionKeys`. Rust's `ConnectedSession` (held in-memory by the manager) already has the record locked and available — a design decision (see ADR below) is whether to read from that live in-memory record (zero extra I/O, but could theoretically diverge from an externally-mutated on-disk record) or replicate acpx's re-load-from-store behavior exactly.
- **Gap 14**: `encode_runtime_handle_state`/`decode_runtime_handle_state`/`write_handle_state` (`handle_state.rs`) and `derive_agent_from_session_key` (`shared.rs`) are fully implemented and unit-tested but have zero production call sites — `manager/mod.rs::handle_for` (L58-74) sets `runtime_session_name: session_key.to_string()` directly, a raw copy, never running it through the encode/write helpers. This means the "opaque handle" round-trip (a caller reconstructing agent/cwd/mode from just the `runtime_session_name` string, without side-channel access to the rest of the handle) is currently impossible even though the machinery for it exists. There is also **no existing Rust entry point that decodes a bare `runtime_session_name` back into a handle** — the reverse direction (`decode_runtime_handle_state` actually being called by something) doesn't exist either; acpx's analog is `resolveHandleState`/`resolveManagerHandle` in `runtime.ts`.
- **Gap 34**: acpx's `ensureSession` (`runtime.ts:132-141`) trims and validates both `sessionKey`/`agent` before use, throwing `AcpRuntimeError("ACP_SESSION_INIT_FAILED", ...)` on either being blank — Rust's `ensure_session` (`manager/mod.rs:76-88`) has no equivalent check; a blank session key or agent flows straight into `record_id_for`/`agent_registry.resolve` unvalidated. `AcpRuntimeErrorCode::SessionInitFailed` already exists in the Rust error enum (confirmed used elsewhere in `manager_spawn.rs`) — this is a pure validation-gap fix, no new error type needed.

## Requirements

1. `get_capabilities` accepts `Option<&AcpRuntimeHandle>` (or equivalent — match whatever the public `AcpRuntime` trait/struct's existing method-signature convention is, e.g. an `input: GetCapabilitiesInput { handle: Option<AcpRuntimeHandle> }` struct if other methods use an input-struct pattern). When a handle is given and a matching session's config options are non-empty, populate `config_option_keys` from them; otherwise return the static base capability set unchanged (backward-compatible for no-handle callers).
2. `handle_for` (or wherever `AcpRuntimeHandle` is constructed) calls `write_handle_state`/`encode_runtime_handle_state` to populate `runtime_session_name` as the versioned opaque-encoded string, instead of a raw `session_key` copy.
3. A new public entry point resolves a bare `runtime_session_name` string back into enough state to re-attach to the right session (mirroring acpx's `resolveHandleState`) — needed for the encode/decode round-trip to be genuinely useful, not just internally consistent. Scope this narrowly: only what's needed to make `decode_runtime_handle_state`'s existing output actionable (see Architecture).
4. `ensure_session` validates `input.session_key.trim()` and `input.agent.trim()` are both non-empty before any other work, returning `AcpRuntimeError::new(AcpRuntimeErrorCode::SessionInitFailed, "ACP session key is required.")` / `"ACP agent id is required."` on failure — matching acpx's exact messages. Downstream code uses the trimmed values (matching acpx, which uses `sessionName`/`agent` post-trim throughout the rest of the function), not the raw input.

## Architecture

```
crates/acp/src/runtime/
├── engine/manager/
│   ├── mod.rs      # ensure_session: + blank session_key/agent validation (Requirement 4);
│   │                 # handle_for: + write_handle_state/encode_runtime_handle_state call
│   │                 #   (Requirement 2); + a new resolve-handle-from-name entry point
│   │                 #   (Requirement 3, exact home TBD — likely here alongside handle_for)
│   └── status.rs   # get_capabilities: + Option<&AcpRuntimeHandle> param, reads live
│                     #   connected.record.acpx.config_options when a handle resolves
└── public/
    ├── handle_state.rs  # no logic changes — already correct, wired in by this phase
    └── shared.rs         # derive_agent_from_session_key — wired in as a fallback inside the
                            #   new resolve-handle-from-name entry point when decoding fails
                            #   (e.g. an older, non-versioned session_key string)
```

## ADR Rationale

### Phase-local ADR: `get_capabilities`'s config-options source — live in-memory record vs. re-load from store (CONFIRMED)

- **Context:** acpx re-loads the record from `sessionStore` every time `getCapabilities({handle})` is called, even if a live connection to that session already exists. Rust's manager already holds a live `Arc<Mutex<SessionRecord>>` inside `ConnectedSession` for any currently-connected session.
- **Decision:** Read from the live in-memory record when the session is currently connected (zero extra I/O, always-fresh within this process); only fall back to `session_store.load()` when no live connection exists for the given handle (matching what a cold `get_capabilities` call on a never-connected-this-process session must do anyway).
- **Why:** avoids a redundant disk read for the common case (checking capabilities of a session you're actively driving), while still supporting the acpx-parity case (checking a session you haven't connected to yet in this process). This is a deliberate, documented improvement over acpx's always-reload behavior, not an accidental divergence — **confirmed** (plan.md Unresolved Questions #5). The theoretical live/store divergence if something external mutates the on-disk record mid-session is an accepted edge case (no other users of this crate today).

## Related code files

- `crates/acp/src/runtime/engine/manager/status.rs` (`get_capabilities`, L20-30).
- `crates/acp/src/runtime/engine/manager/mod.rs` (`ensure_session` L76-88, `handle_for` L58-74).
- `crates/acp/src/runtime/public/handle_state.rs` (`encode_runtime_handle_state` L28-34, `decode_runtime_handle_state` L37-42, `write_handle_state` L45-51 — read only, no logic changes).
- `crates/acp/src/runtime/public/shared.rs` (`derive_agent_from_session_key` L34-47 — read only).
- Reference (read-only): `others/acpx/src/runtime.ts` (`getCapabilities` L224-247, `ensureSession` L132-141, `resolveHandleState`/`resolveManagerHandle` — locate and read before implementing Requirement 3).

## Implementation Steps

1. Read `others/acpx/src/runtime.ts`'s `resolveHandleState`/`resolveManagerHandle` in full to confirm the exact contract Requirement 3 needs to match (what inputs it accepts, what it returns, how it falls back when decoding fails).
2. Add blank-check validation to `ensure_session` (Requirement 4) as the very first statements in the function body; use the trimmed values for every subsequent use of `session_key`/`agent` in the function.
3. Change `handle_for` to build an `AcpxHandleState` (check its exact field names in `handle_state.rs`) from the already-available `record`/`session_key`/`agent`/`cwd` data, call `write_handle_state`, use its output for `runtime_session_name` instead of the raw `session_key.to_string()`.
4. Add the new resolve-handle-from-name entry point (exact name TBD, e.g. `resolve_handle_from_runtime_session_name`) to `manager/mod.rs`: try `decode_runtime_handle_state` first; on `None` (e.g. an older non-versioned name, or a name from before this phase shipped), fall back to `derive_agent_from_session_key` for a best-effort reconstruction, matching acpx's tolerance for pre-existing handles.
5. Change `get_capabilities`'s signature to accept an optional handle (match the codebase's existing method-signature idiom — check whether other `AcpRuntime` methods take a bare param or an input-struct, for consistency). When a handle resolves to a connected session, read `config_options` from the live record (per this phase's ADR); when it resolves to a not-currently-connected session, fall back to `session_store.load()`; when no handle is given or nothing resolves, return the static capability list unchanged.
6. Unit tests: `ensure_session` rejects blank session_key, blank agent, and both blank, with the exact acpx-matching error messages; accepts and trims a session_key/agent with leading/trailing whitespace. `handle_for` produces a `runtime_session_name` that `decode_runtime_handle_state` can successfully decode back into the expected agent/cwd. `get_capabilities` returns the static list with no handle, and a handle-specific `config_option_keys` list when a handle with known config options is given.
7. Real call-path integration test: spawn a fake agent, create a session with specific config options advertised, call `get_capabilities` with that session's handle, confirm the returned `config_option_keys` match — not just that the pure logic works in isolation but that the manager's live-record wiring actually surfaces it.
8. `cargo fmt -p boltz-acpx`, `cargo check -p boltz-acpx --all-targets --features test-support`, `cargo test -p boltz-acpx --features test-support`, `make check-all`.
9. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-04-runtime-engine-public-contract.md` per plan.md's housekeeping (gaps 13, 14, 34).

## Todo list

- [ ] `ensure_session` blank session_key/agent validation.
- [ ] `handle_for` uses `write_handle_state`/`encode_runtime_handle_state`.
- [ ] New resolve-handle-from-name entry point using `decode_runtime_handle_state` + `derive_agent_from_session_key` fallback.
- [ ] `get_capabilities` accepts optional handle, reads live record's config options.
- [ ] Unit tests: validation, handle encode/decode round-trip, capabilities with/without handle.
- [ ] Integration test: real session's config options surfaced via `get_capabilities`.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green.
- [ ] Correct original plan's Phase 4 status text (gaps 13, 14, 34).

## Success Criteria

- `ensure_session("", "claude", ...)` and `ensure_session("key", "", ...)` both return `AcpRuntimeErrorCode::SessionInitFailed` with the exact acpx-matching message text.
- A handle produced by `handle_for` round-trips through `decode_runtime_handle_state` to recover the original agent/cwd.
- `get_capabilities` called with a real, connected session's handle returns `config_option_keys` matching that session's actually-advertised config options (integration-tested against the real fake-agent binary, not just a unit test of the pure logic).

## Risk Assessment

- **`runtime_session_name` format change**: switching from a raw session-key copy to an opaque encoded string is technically a public-API behavior change for any external caller relying on `runtime_session_name == session_key` — since this crate has no external consumers yet (still pre-integration into the GPUI app), this is safe now but should be called out as a one-time breaking change in this phase's own notes for whoever integrates the GPUI app next.
- **Live-record vs. store divergence** (see ADR) — low risk given single-process usage today, but document clearly so a future multi-process extension doesn't silently inherit a subtle bug.

## Security Considerations

- No new untrusted-input surface. `ensure_session`'s validation is a pure input-sanity gate, not a security boundary in itself, but prevents blank-key session confusion (two different blank-key requests could otherwise collide on the same internal record).

## Next steps

- Proceed to [Phase 8](./phase-08-agent-quirks-shutdown-persistence.md) (LOW tier) once all MEDIUM-tier phases merge.
- No unresolved questions for this phase — the ADR's live-record-vs-store choice is confirmed (plan.md Unresolved Questions #5).
