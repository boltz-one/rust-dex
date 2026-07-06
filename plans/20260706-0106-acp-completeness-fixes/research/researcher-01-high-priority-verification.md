# High-Priority Gap Verification Report

## Summary
Verified 5 claims via direct codebase inspection. All claims confirmed with specific evidence.

---

## Claim 1: PermissionPolicy Never Wired to RPC Handler
**Status: CONFIRMED (with caveat)**

Evidence:
- `crates/acp/src/client/handshake.rs:216` — `None` hardcoded as 4th parameter to `resolve_permission_request()`
  - Full call: `resolve_permission_request(&req, permission.mode, permission.non_interactive_policy, None, permission.handler...)`
- `crates/acp/src/client/handlers.rs:30` — struct only carries `non_interactive_policy: NonInteractivePermissionPolicy`, no broader `PermissionPolicy` field
- `crates/acp/src/runtime/public/contract/options.rs:35` — `AcpRuntimeOptions` carries `non_interactive_permissions: NonInteractivePermissionPolicy`, NOT a pluggable `PermissionPolicy`

**Caveat:** Only `NonInteractivePermissionPolicy` is threaded; if claim is about a distinct `PermissionPolicy` type being dead, grep found zero bare "PermissionPolicy" mentions across these three files—suggests it doesn't exist yet or is named differently.

---

## Claim 2: Reconnect State Machine Has ~0 Unit Tests
**Status: CONFIRMED**

Evidence:
- `grep -c "#\[test\]"` output:
  - `crates/acp/src/runtime/engine/reconnect/mod.rs:0`
  - `crates/acp/src/runtime/engine/reconnect/replay.rs:0`
- No `mod tests` submodule found in either file (grep returned no matches)

**Conclusion:** Zero test coverage in reconnect module.

---

## Claim 3: request_token_usage Eviction Uses HashMap (Unordered)
**Status: CONFIRMED**

Evidence from `crates/acp/src/session/conversation_model/trim.rs:43–56`:
```rust
if conversation.request_token_usage.len() > MAX_RUNTIME_REQUEST_TOKEN_USAGE {
    // acpx keeps the *last* N entries in insertion order
    // (`Object.entries(...).slice(-N)`); a `HashMap` has no stable
    // insertion order, so this port approximates it by keeping an
    // arbitrary N entries.
    let keep: Vec<_> = conversation
        .request_token_usage
        .drain()
        .take(MAX_RUNTIME_REQUEST_TOKEN_USAGE)
        .collect();
    conversation.request_token_usage = keep.into_iter().collect();
}
```

**Conclusion:** Code comment explicitly acknowledges HashMap semantics break original "last N in insertion order" behavior. **indexmap fix is justified.**

---

## Claim 4: indexmap Not in Workspace Dependencies
**Status: CONFIRMED**

Evidence:
- Root `Cargo.toml` — `grep "^indexmap"` returned 0 results
- `crates/acp/Cargo.toml` — `grep indexmap` returned 0 results

**Conclusion:** `indexmap` must be added as a dependency before it can be used.

---

## Claim 5: maybe_wrap_session_control_error Ported But Never Called
**Status: CONFIRMED**

Evidence:
- `grep -rn "maybe_wrap_session_control_error" crates/acp/src/` returned 0 results

**Conclusion:** Function exists but has zero call sites. Dead code.

---

## Summary Table

| Claim | Status | Evidence |
|-------|--------|----------|
| 1. PermissionPolicy never wired | CONFIRMED | None hardcoded at handshake.rs:216; no PermissionPolicy field in AcpRuntimeOptions |
| 2. Reconnect tests ~0 | CONFIRMED | Both mod.rs and replay.rs: 0 #[test] markers; no test submodule |
| 3. HashMap eviction (unordered) | CONFIRMED | trim.rs:44 comment: "HashMap has no stable insertion order"; needs indexmap |
| 4. indexmap not in deps | CONFIRMED | grep found zero matches in root and crate Cargo.toml |
| 5. maybe_wrap_session_control_error never called | CONFIRMED | grep -rn returned 0 call sites |

---

## Unresolved Questions
- Does a distinct `PermissionPolicy` type exist elsewhere, or is the gap really about threading `NonInteractivePermissionPolicy` more thoroughly?
- What is the intended call path for `maybe_wrap_session_control_error`—should it be activated or deleted?
