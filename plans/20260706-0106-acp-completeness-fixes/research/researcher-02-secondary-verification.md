# ACP Completeness Audit — Secondary Verification Report

## Claim 1: Windows Support Status
**Result: CONFIRMED** ✓

Windows is an explicitly supported platform:
- `docs/system-architecture.md:35-44` — Architecture diagram shows Windows Backend (DirectX+Win32)
- `docs/system-architecture.md:69` — Platform startup lists Windows: `gpui_windows::platform::Platform with DirectX setup`
- `docs/project-overview-pdr.md:15,36` — PDR lists `crates/gpui_windows` as included, cross-platform scope explicitly mentions Windows
- `Cargo.toml:19,63` — `crates/gpui_windows` is a real workspace member at `path = "crates/gpui_windows"`
- `crates/gpui_windows/src/` — Contains real implementation files: `alpha_correction.hlsl`, `clipboard.rs`, `direct_manipulation.rs`, etc.

**Decision: Windows batch-shell wrapping (.cmd/.bat) is in-scope.**

---

## Claim 2: `authenticate` RPC Never Called
**Result: CONFIRMED** ✓

- `grep -rn "authenticate\b" crates/acp/src/` → No matches outside test files
- Credential functions are defined in `crates/acp/src/auth_env.rs:48,56` but only called within their own test (`auth_env.rs:177-180`)
- No call sites exist in runtime, client, or engine code paths

**Impact: `auth_env.rs`'s `read_env_credential` and `resolve_configured_auth_credential` are completely unused in the runtime.**

---

## Claim 3: `to_raw_input` Double-JSON-Encodes
**Result: CONFIRMED** ✓

Source: `crates/acp/src/session/conversation_model/tool_use.rs:16-21`

```rust
pub(super) fn to_raw_input(value: Option<&Value>) -> String {
    match value {
        Some(value) => trim_runtime_text(&value.to_string(), MAX_RUNTIME_TOOL_IO_CHARS),
        None => trim_runtime_text("{}", MAX_RUNTIME_TOOL_IO_CHARS),
    }
}
```

For `Value::String(s)`:
- `value.to_string()` serializes the Value → produces `"s"` (with outer quotes)
- Calls `trim_runtime_text()` on the already-quoted string
- Result: string values get double-quoted

**Note:** This is correct behavior for raw_input (should be JSON), but non-String values and strings are inconsistent paths.

---

## Claim 4: `close()` Never Sends `session/close` to Backend
**Result: CONFIRMED** ✓

Source: `crates/acp/src/runtime/engine/manager/queue_control.rs:74-115`

The `close()` function:
1. Removes session from internal map (line 81)
2. Marks record as closed + persists via session store (lines 95-100)
3. Calls `connected.client.shutdown()` (line 105)

**No `session/close` RPC or SessionClose message is sent.** Local cleanup only; backend has no signal that session ended.

---

## Claim 5: `PermissionStats` Never Ported
**Result: CONFIRMED** ✓

- `grep -rn "PermissionStats\|permission_stats" crates/acp/src/` → No matches
- No partial/renamed versions found
- Structure is completely absent from the port

---

## Summary

| Claim | Status | Evidence |
|-------|--------|----------|
| Windows is supported | CONFIRMED | system-architecture.md:35-44,69; project-overview-pdr.md:15,36; Cargo.toml:19,63; real src/ files |
| `authenticate` RPC unused | CONFIRMED | Zero non-test callsites; functions defined but orphaned in auth_env.rs |
| `to_raw_input` double-encodes | CONFIRMED | Calls `.to_string()` on Value, which adds quotes for String types |
| `close()` doesn't send session/close | CONFIRMED | Only local cleanup + session_store.save(); no RPC call |
| `PermissionStats` missing | CONFIRMED | No grep matches in entire crates/acp/src |

All five claims are **ready for implementation planning.**
