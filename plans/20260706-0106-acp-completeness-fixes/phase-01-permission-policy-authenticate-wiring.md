# Phase 1: Permission Policy, Escalation Audit Trail, Authenticate RPC, Auth Credential Map, Permission Stats

## Context links

- Plan: [plan.md](./plan.md)
- Research: [researcher-01-high-priority-verification.md](./research/researcher-01-high-priority-verification.md), [researcher-02-secondary-verification.md](./research/researcher-02-secondary-verification.md)
- Original port plan phase to correct after this lands: [phase-03-permissions-filesystem-terminal.md](../20260705-1718-acpx-to-acp-crate-port/phase-03-permissions-filesystem-terminal.md) (gaps 1,2,25), [phase-02-protocol-transport-lifecycle.md](../20260705-1718-acpx-to-acp-crate-port/phase-02-protocol-transport-lifecycle.md) (gap 3, gap 24)
- Next: [Phase 2](./phase-02-reconnect-hardening-claude-timeout.md), [Phase 3](./phase-03-conversation-trim-import-security.md) (parallel, file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/client/{handshake.rs,handlers.rs,state.rs}`, `crates/acp/src/runtime/public/contract/options.rs`, `crates/acp/src/permissions/*.rs`, `crates/acp/src/runtime/engine/manager_spawn.rs` (thread new fields only, no other edits), `crates/acp/src/auth_env.rs` (call sites only, no signature change), `crates/acp/src/runtime/public/events/types.rs` (only if ADR-8 needs a variant — it does not, per this phase's decision). No other files. No `crates/acp/Cargo.toml` changes needed (no new deps).

## Overview

- **Priority:** P1 (HIGH — authorization + audit-trail gaps)
- **Status:** pending
- **Description:** Fix 5 gaps, all rooted in the same finding: `client/handshake.rs:216` hardcodes `policy: None` when calling permission resolution, using the response-only wrapper instead of the full decision-tree function — silently dropping policy overrides, escalation events, and any place a `PermissionStats` counter could be incremented. Also wires the never-called `authenticate` RPC and an always-`None` auth-credential map, both orphaned in `auth_env.rs`/nowhere threaded from `AcpRuntimeOptions`.

## Key Insights (from verification research)

- `resolve_permission_request_with_details(params, mode, non_interactive_policy, policy, handler)` (`permissions/resolve.rs:106`) already implements the full policy→mode→prompt decision tree AND returns `ResolvedPermissionRequest.escalation: Option<PermissionEscalationEvent>`. `handshake.rs:211-219` calls the thin wrapper `resolve_permission_request` instead, which (a) hardcodes `None` for `policy` and (b) discards `.escalation` at `resolve.rs:178`. **The fix for gaps 1 and 2 is the same call-site change**: switch to the `_with_details` function and pass through both new pieces of data.
- `PermissionPolicy` (`permissions/policy.rs:23-30`, fields `auto_approve/auto_deny/escalate: Vec<String>`, `default_action: Option<PermissionPolicyAction>`) already exists and is fully implemented+tested inside `permissions/`. It has zero references from `client/` or `runtime/` — this is a **pure threading gap**, not a missing-implementation gap.
- `PermissionRequestWiring` (`client/handlers.rs:28-32`) is constructed in exactly one place: `runtime/engine/manager_spawn.rs:89`. This is the only call site that needs the new field threaded through.
- `AcpRuntimeOptions` (`runtime/public/contract/options.rs:29-49`) is the natural home for `permission_policy: Option<PermissionPolicy>`, `on_permission_escalation: Option<Arc<dyn Fn(PermissionEscalationEvent) + Send + Sync>>`, and `auth_credentials: Option<HashMap<String, String>>` — it already carries the analogous `on_permission_request` handler and `non_interactive_permissions` policy.
- acpx's `authenticateIfRequired` (`others/acpx/src/acp/client.ts:1634-1661`) runs immediately after `initialize` resolves, inside `initializeProtocolConnection` (`client.ts:806-823`) — i.e. it belongs inside `handshake.rs::spawn_and_initialize`, which already has the `InitializeResponse` in scope, not in `client/mod.rs::spawn` (that function only wraps the Gemini *initialize*-timeout race, a different concern). This keeps gap 3's fix file-disjoint from gap 4's fix (Phase 2), which touches `session/new`, not `initialize`.
- `auth_env::read_env_credential`/`resolve_configured_auth_credential` are pure functions, already correct, zero call sites outside their own tests. `build_agent_environment` (`auth_env.rs:81-86`) already accepts `auth_credentials: Option<&HashMap<String, String>>` but its 2 non-test callers (`probe.rs:69`, `manager_spawn.rs:61`) always pass `None` — this is gap 24, and it shares the exact same missing `AcpRuntimeOptions` field this phase is already adding for auth-method selection (gap 3), so it's folded into this phase per the plan's ADR guidance.
- `PermissionStats` has zero Rust equivalent (net-new port). acpx's `recordPermissionDecision`/`recordPermissionError` (`client.ts:1943-1964`) increment 4 counters (`requested/approved/denied/cancelled`) from inside the permission RPC handler — the natural Rust home is `client/state.rs`'s `ClientState` (the file created in the original Phase 2 specifically for "non-queue client state"), read back via a new accessor.
- **Whether `agent-client-protocol`'s Rust SDK exposes a typed `authenticate` method was not verified during research** — this must be checked as Implementation Step 1, before any other code in this phase, since it determines whether gap 3 uses the SDK's method directly or needs a hand-rolled `jsonrpc_gap.rs`-style fallback (see ADR-1 in the original port plan for the precedent).

## Requirements

1. `AcpRuntimeOptions` gains 3 new optional fields: `permission_policy: Option<PermissionPolicy>`, `on_permission_escalation: Option<Arc<dyn Fn(PermissionEscalationEvent) + Send + Sync>>`, `auth_credentials: Option<HashMap<String, String>>`. All default to `None` (no behavior change for existing callers that don't set them).
2. `PermissionRequestWiring` gains `policy: Option<PermissionPolicy>`, threaded from `AcpRuntimeOptions.permission_policy` at `manager_spawn.rs:89`.
3. `handshake.rs`'s permission-request RPC closure calls `resolve_permission_request_with_details` (not the response-only wrapper), passes `permission.policy.as_ref()` instead of `None`, and — when `.escalation` is `Some(event)` — invokes the caller-supplied `on_permission_escalation` callback if present (fire-and-forget, must not block or fail the RPC response).
4. After `initialize` succeeds inside `spawn_and_initialize`, if `init_response.auth_methods` (or SDK-equivalent field name — confirm exact name in Step 1) is non-empty and no credential resolves via `read_env_credential`/`resolve_configured_auth_credential(auth_credentials)`, behavior must match acpx's `authPolicy` semantics: fail loudly if configured to `"fail"`, otherwise proceed without authenticating (log only) — mirror `AcpClientOptions.authPolicy` if the Rust port has an equivalent option, otherwise default to "proceed without auth, let the agent reject if it requires it" (least-surprise default matching what happens today by omission) and flag the policy-configurability question as a follow-up if no equivalent option field exists yet.
5. `build_agent_environment`'s 2 production call sites (`probe.rs:69`, `manager_spawn.rs:61`) pass `options.auth_credentials.as_ref()` instead of `None`.
6. `PermissionStats { requested: u64, approved: u64, denied: u64, cancelled: u64 }` struct added to `client/state.rs`, incremented from the same RPC closure in `handshake.rs` that resolves each permission request (approved/denied/cancelled derived from the resolved decision; `requested` incremented unconditionally once per request). Exposed via a new accessor on `ClientState` (or `AcpClient`, whichever already exposes `state()` — confirm exact existing pattern before adding).

## Architecture

```
crates/acp/src/
├── runtime/public/contract/options.rs
│   └── AcpRuntimeOptions { ..., permission_policy: Option<PermissionPolicy>,
│         on_permission_escalation: Option<Arc<dyn Fn(PermissionEscalationEvent) + Send + Sync>>,
│         auth_credentials: Option<HashMap<String, String>> }
├── client/
│   ├── handlers.rs   # PermissionRequestWiring { ..., policy: Option<PermissionPolicy> }
│   ├── handshake.rs  # permission RPC closure calls resolve_permission_request_with_details;
│   │                 # forwards .escalation to on_permission_escalation; increments PermissionStats;
│   │                 # spawn_and_initialize calls authenticate_if_required after initialize succeeds
│   └── state.rs      # + PermissionStats struct, ClientState gains a stats field + accessor
├── permissions/*.rs  # NO logic changes — already correct, this phase only threads inputs to it
└── runtime/engine/manager_spawn.rs  # threads AcpRuntimeOptions.{permission_policy,auth_credentials}
                                      # into PermissionRequestWiring / build_agent_environment calls
```

## ADR Rationale

### ADR-7: `PermissionPolicy` threading shape — programmatic field, no CLI/config loader

- **Context:** acpx's `PermissionPolicy` reaches the client via `AcpClientOptions.permissionPolicy`, itself populated by a CLI-only loader (`permission-policy.ts`'s `parsePermissionPolicy`/`loadPermissionPolicySpec`, driven by a `--permission-policy <json-or-path>` flag).
- **Decision:** Add `permission_policy: Option<PermissionPolicy>` directly to `AcpRuntimeOptions` as a programmatic (Rust struct) value the embedding host (GPUI app) constructs and passes in. Do **not** port `permission-policy.ts`'s CLI-arg/file-loading logic.
- **Why:** `crates/acp` has no CLI layer (confirmed — the whole crate is a library embedded by a GPUI app). A JSON/file-based config loader is YAGNI here: the host application already has its own settings/config system and can construct a `PermissionPolicy` value directly, more idiomatically than round-tripping through JSON. Porting the loader would introduce a config-file convention this crate has no other precedent for.

### ADR-8: `PermissionEscalationEvent` surfaced via callback field, not an event-stream variant (CONFIRMED)

- **Context:** acpx's `onPermissionEscalation?: (event) => void` is a synchronous, fire-and-forget callback on `AcpClientOptions`, invoked once per escalation regardless of which (if any) prompt turn is in flight — permission requests can occur outside an active `session/prompt` (e.g. during a `terminal/create` call triggered by a tool), so tying escalation events to a specific `AcpRuntimeTurn`'s event stream (`Requirement 3` of the original Phase 4) would either drop escalations that happen outside a turn or force an awkward "which turn does this belong to" lookup.
- **Decision:** `AcpRuntimeOptions.on_permission_escalation: Option<Arc<dyn Fn(PermissionEscalationEvent) + Send + Sync>>` — a synchronous, non-async callback (escalation is a notification, not a decision the caller must respond to, unlike `on_permission_request`'s async handler). Called from `handshake.rs`'s RPC closure immediately after `resolve_permission_request_with_details` returns, best-effort (a panic inside the callback must not be allowed to poison the RPC response path — wrap in `std::panic::catch_unwind` or document the caller's obligation not to panic, decide at implementation time).
- **Why over a new `AcpRuntimeEvent` variant:** matches acpx's actual placement (client-level, not turn-level) and requires zero changes to `AcpRuntimeTurn`'s already-stable event-stream contract (Phase 4 of the original plan explicitly flagged public-contract stability as a risk). **Confirmed** (plan.md Unresolved Questions #1) — the callback shape is locked in; an event-stream variant is not planned unless a future embedding need for interleaving escalations with a turn's `Stream<Item = AcpRuntimeEvent>` arises.

## Related code files

- `crates/acp/src/client/handshake.rs` (permission RPC closure ~L200-230, `spawn_and_initialize` ~L131+).
- `crates/acp/src/client/handlers.rs` (`PermissionRequestWiring`, L28-32).
- `crates/acp/src/client/state.rs` (`ClientState`, exact fields TBD at read time — read in full before editing).
- `crates/acp/src/runtime/public/contract/options.rs` (`AcpRuntimeOptions`, L29-49).
- `crates/acp/src/permissions/{policy.rs,resolve.rs,escalation.rs,response.rs}.rs` — read only, no logic edits.
- `crates/acp/src/runtime/engine/manager_spawn.rs` (L89 `PermissionRequestWiring` construction, L61 `build_agent_environment` call).
- `crates/acp/src/runtime/public/probe.rs` (L69 `build_agent_environment` call).
- `crates/acp/src/auth_env.rs` (`read_env_credential` L48, `resolve_configured_auth_credential` L56, `build_agent_environment` L81-86 — call-site changes only).
- Reference (read-only): `others/acpx/src/permission-policy.ts`, `others/acpx/src/permissions.ts` (`resolvePermissionRequestFromMode`, `emitPermissionEscalation` ~L1742-1765), `others/acpx/src/acp/client.ts` (`authenticateIfRequired`/`selectAuthMethod` ~L1562-1661, `recordPermissionDecision`/`recordPermissionError` ~L1943-1964), `others/acpx/src/types.ts` (`PermissionStats` L123-127).

## Implementation Steps

1. **Confirm SDK `authenticate` support**: read `agent-client-protocol`/`agent-client-protocol-schema`'s docs.rs page (or vendored source) for an `authenticate`/`AuthenticateRequest` type and the exact field name for advertised auth methods on `InitializeResponse` (acpx calls it `authMethods`). Document the actual finding in this file's Implementation status once known. If absent, hand-roll a `jsonrpc_gap`-style request/response pair following the pattern already established for `session/set_mode` etc. (see original Phase 2's ADR-1).
2. Add `permission_policy`, `on_permission_escalation`, `auth_credentials` fields to `AcpRuntimeOptions` (`options.rs`), all `Option<..>`, all default `None` — confirm the struct's existing `Default`/builder pattern and extend consistently.
3. Add `policy: Option<PermissionPolicy>` to `PermissionRequestWiring` (`handlers.rs`); update its `Default` impl to `None`.
4. In `manager_spawn.rs:89`, thread `options.permission_policy.clone()` into the new field. At `manager_spawn.rs:61` and `probe.rs:69`, pass `options.auth_credentials.as_ref()` instead of `None`.
5. In `handshake.rs`'s permission RPC closure, replace the `resolve_permission_request(...)` call with `resolve_permission_request_with_details(&req, permission.mode, permission.non_interactive_policy, permission.policy.as_ref(), permission.handler.as_deref())`; extract `.response` for the RPC reply (unchanged behavior) and `.escalation` for the new callback + stats hook.
6. Add `PermissionStats` struct to `client/state.rs`; add a field on `ClientState` (or wherever `AcpClient`'s per-client mutable state already lives — confirm exact home during Step 1's file read) + an accessor method (e.g. `AcpClient::permission_stats() -> PermissionStats`, mirroring acpx's `getPermissionStats()`). Increment `requested` once per resolved request; derive approved/denied/cancelled from the resolved decision's outcome (map `PermissionDecision`/`ResolvedPermissionRequest.response`'s outcome — read `permissions/response.rs` to find the exact enum to match on).
7. Wire the `on_permission_escalation` callback: after step 5's resolution, if `resolved.escalation.is_some()` and `options.on_permission_escalation` is `Some(cb)`, call `cb(escalation_event)`. Guard against a panicking callback poisoning the transport task (wrap the call or document the non-panic contract in the field's rustdoc).
8. Implement `authenticate_if_required` inside `handshake.rs` (new private fn, called from `spawn_and_initialize` right after `init_response` is captured, before returning `Ok(RunningConnection{..})`): iterate `init_response`'s advertised auth methods, try `read_env_credential` then `resolve_configured_auth_credential(method_id, auth_credentials)` per method (mirror acpx's `selectAuthMethod` order), call the SDK's `authenticate` (or the Step-1 fallback) on first match, no-op if no methods advertised. Decide and document the "no credential found" behavior per Requirement 4 (this crate currently has no `authPolicy`-equivalent option — adding one is out of scope unless trivial; document the chosen default behavior clearly in the function's rustdoc either way).
9. Write unit tests for `PermissionStats` counting (all 4 outcomes) and for the escalation-callback firing (fake handler asserting it was called with the expected event) directly in `permissions/`-adjacent or `client/`-adjacent test modules — these can be pure unit tests, no fake-agent subprocess needed since they exercise the resolution function + callback wiring, not RPC framing.
10. **Real call-path integration test** (required per plan.md's testing rule — dead-code-fix gaps need a test that exercises the REAL path): extend `crates/acp/tests/fixtures/fake_agent/main.rs` if needed so its `initialize` response can be configured (env var) to advertise at least one auth method, and its permission-request flow can trigger an escalation. Add an integration test in `tests/` (new or existing file) that: spawns the fake agent with an `AcpRuntimeOptions.permission_policy` configured to force an escalation, asserts `on_permission_escalation` fires with the expected event; and a second test that configures `auth_credentials` + a fake-agent-advertised auth method, asserts `authenticate` was actually sent (fake agent logs/echoes the received `methodId` so the test can assert on it).
11. `cargo fmt -p boltz-acpx`, `cargo check -p boltz-acpx --all-targets --features test-support`, `cargo test -p boltz-acpx --features test-support`, `make check-all`.
12. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-03-permissions-filesystem-terminal.md` and `phase-02-protocol-transport-lifecycle.md` per this plan's housekeeping (already done as part of this planning pass — see plan.md's original-phase-file corrections; re-verify status text matches the real post-fix state once implemented).

## Todo list

- [ ] Confirm `agent-client-protocol` SDK's `authenticate` support (Step 1).
- [ ] Add 3 new `AcpRuntimeOptions` fields.
- [ ] Add `policy` field to `PermissionRequestWiring`.
- [ ] Thread `permission_policy`/`auth_credentials` through `manager_spawn.rs`/`probe.rs`.
- [ ] Switch `handshake.rs`'s RPC closure to `resolve_permission_request_with_details`.
- [ ] Add `PermissionStats` to `client/state.rs`, wire increments.
- [ ] Wire `on_permission_escalation` callback.
- [ ] Implement `authenticate_if_required` in `handshake.rs`, call from `spawn_and_initialize`.
- [ ] Unit tests: `PermissionStats` all 4 outcomes, escalation callback firing.
- [ ] Integration tests against real fake-agent binary: policy-driven escalation end-to-end, authenticate RPC actually sent.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green; test count grows past 274.
- [ ] Correct original plan's Phase 2/Phase 3 status text (see plan.md housekeeping).

## Success Criteria

- A test configures `AcpRuntimeOptions.permission_policy` with an `escalate` rule matching a specific tool, drives a real permission request through the real fake-agent binary, and asserts `on_permission_escalation` fired with a `PermissionEscalationEvent` whose `matched_rule` reflects the configured rule — not just that the pure `resolve_permission_request_with_details` function returns the right value in isolation.
- A test configures `auth_credentials` with a credential for a fake-agent-advertised auth method, and asserts (via the fake agent's own log/echo) that `authenticate` was actually sent over the wire with the matching `method_id`.
- `AcpClient::permission_stats()` (or equivalent accessor) returns correct counts after a sequence of approved/denied/cancelled requests in a test.
- `cargo test -p boltz-acpx --features test-support` count is >= 274 + new tests, all green.

## Risk Assessment

- **Callback panic safety**: `on_permission_escalation` runs inside the same async task that must return the permission RPC's response — a panicking callback could poison that task and break the in-flight `request_permission` response, causing the agent's RPC to hang or error unexpectedly. Mitigate: catch_unwind or document strictly as the caller's responsibility, decide explicitly (don't leave implicit).
- **`authenticate_if_required`'s "no credential, no policy" default**: acpx has an explicit `authPolicy: "fail" | ...` option this crate doesn't yet have. Silently proceeding without authenticating could surface a confusing downstream error from the agent instead of a clear one from this crate. Document the chosen behavior prominently; this is an accepted narrower-scope decision (adding a full `authPolicy` option is out of scope for this phase) unless implementation reveals it's trivial to add.
- **SDK method mismatch (Step 1)**: if `agent-client-protocol` doesn't expose `authenticate`, the fallback hand-rolled RPC adds scope; budget for this in the 6h estimate being a soft floor, not a hard ceiling.

## Security Considerations

- This phase directly touches the permission-enforcement decision path (`handshake.rs`'s RPC closure) — any regression here is a security regression. The switch from `resolve_permission_request` to `resolve_permission_request_with_details` must preserve the exact same `.response` value for the RPC reply in the non-escalation case (verify via existing `permissions/resolve_tests.rs` unit tests still passing unchanged, plus a diff-review of the two functions' logic to confirm `_with_details` truly is a superset, not a behavioral change).
- `auth_credentials: Option<HashMap<String, String>>` on `AcpRuntimeOptions` carries secrets (API keys/tokens) — ensure no `Debug`/`log::debug!` derive or call site logs this field's contents (audit `AcpRuntimeOptions`'s existing `Debug` impl, if any, and exclude this field or redact it).
- `PermissionStats` counters are aggregate numbers only (no tool names/content) — confirmed no new sensitive-data exposure surface.

## Next steps

- Proceed to [Phase 2](./phase-02-reconnect-hardening-claude-timeout.md) and [Phase 3](./phase-03-conversation-trim-import-security.md) in parallel (file-disjoint, verified in plan.md's matrix).
- ADR-8's escalation-callback shape is confirmed (plan.md Unresolved Questions #1) — no further sign-off needed.
- The `authPolicy`-equivalent option is confirmed deferred (plan.md Unresolved Questions #6): proceed without it this phase; document the "proceed without auth, let the agent reject if required" default per Requirement 4 and revisit only if a real embedding need surfaces.
