# Research: Rust ACP Ecosystem & Non-Tokio Subprocess I/O

**Date:** 2026-07-05 | **Budget:** 5 searches | **Methodology:** Parallel web searches + GitHub/crates.io verification

---

## Executive Summary

**Finding:** Official Rust ACP SDK exists and is mature. **Recommendation: Reuse `agent-client-protocol` crate.** For non-tokio subprocess I/O, use **`async-task` + `async-executor` + custom stdio framing** (matches your futures/boltz-scheduler stack) or **`smol` runtime** as a lighter alternative to tokio.

---

## Q1: Official Rust ACP Implementation?

**YES** — Mature, official implementation exists.

- **Primary crate:** [`agent-client-protocol`](https://crates.io/crates/agent-client-protocol) ([GitHub](https://github.com/agentclientprotocol/rust-sdk)) — maintained by agentclientprotocol org
- **Ecosystem:** 
  - `agent-client-protocol-schema` — Protocol types/JSON schema
  - `agent-client-protocol-tokio` — Tokio utilities (spawning agents as subprocesses)
  - `agent-client-protocol-conductor` — Proxy/router for ACP chains
  - `acpx` ([crates.io](https://crates.io/crates/acpx)) — Thin subprocess launcher already using the SDK

**Maturity:** Referenced by Zed (the reference editor implementation) and multiple production adopters. [ACP registry live](https://zed.dev/blog/acp-registry).

---

## Q2: RPC Coverage vs. Your Feature List

From [`agent-client-protocol` trait docs](https://docs.rs/agent-client-protocol/latest/agent_client_protocol/):

**Agent trait (server-side):** ✅ `initialize` ✅ `authenticate` ✅ `session/new` ✅ `session/prompt` ✅ `session/cancel`

**Client trait (client-side, what agent calls back):** ✅ `fs/read_text_file` ✅ `fs/write_text_file` ✅ `terminal/create` ✅ `terminal/output` ✅ `terminal/kill` ✅ `request_permission`

**GAPS:** 
- `session/set_mode`, `session/set_config_option`, `terminal/release` not explicitly listed in SDK trait docs (may exist in schema but not yet bound to Rust types)
- Check [`agent_client_protocol_schema`](https://docs.rs/agent-client-protocol-schema/latest/agent_client_protocol_schema/) for full capability matrix

---

## Q3: Non-Tokio Subprocess + Stdio JSON-RPC

### Problem: ACP Uses JSON-RPC 2.0 Over Stdio
[Zed's architecture](https://zed.dev/acp): Editor spawns agent subprocess, exchanges newline-delimited JSON via stdin/stdout. No built-in tokio requirement — just async I/O + process spawning.

### Ecosystem Analysis

| Crate | Stack | Fit | Notes |
|-------|-------|-----|-------|
| **`async-task` + `async-executor`** | futures-compatible | ⭐⭐⭐ | Lightweight executor on top of your existing futures. Pair with stdlib `std::process` or `std::fs` async wrappers. Same pattern as `boltz-scheduler`. |
| **`smol`** | standalone runtime | ⭐⭐ | Full runtime (reactor + executor). Lighter than tokio. No tokio ecosystem integration, but futures-compatible. Requires runtime block_on. |
| **`async-process`** | futures wrapper | ? | [Crate exists](https://crates.io/crates/async-process) but ecosystem unclear; may require tokio-compat. |
| **`async-std`** | standalone runtime | ⭐ | Heavy for your needs; designed as tokio alternative, ecosystem fragmented. |

**Reference:** [Async Ecosystem Comparison](https://rust-lang.github.io/async-book/08_ecosystem/00_chapter.html).

### Recommended Approach for Your Workspace

**Option A (RECOMMENDED for your stack):** 
- Reuse `agent-client-protocol` (core trait/types)
- Spawn subprocess with `async-task` spawned on your `boltz-scheduler` executor
- Manually wrap subprocess stdin/stdout with `tokio::io::AsyncRead/Write` adapters or futures-compatible async I/O (e.g., `async-fs` pattern you already use)
- Frame JSON-RPC messages as newline-delimited JSON on wire

**Option B (Lighter, more isolated):**
- Use `smol` as a lightweight subprocess reactor
- Keep main app on `boltz-scheduler`; delegate ACP agents to smol runtime island
- Trade: Harder to integrate, but compartmentalizes async runtimes

**Precedent:** Zed's own implementation spawns subprocesses natively without forcing a specific async runtime on agents (agents can be any language/async runtime). Use JSON-RPC framing as the contract.

---

## Implementation Recommendation

**1. Reuse `agent-client-protocol` crate** (don't hand-roll ACP framing)
**2. For subprocess spawning:**
   - Async-spawn: `async-task::Task::spawn()` on `boltz-scheduler` executor
   - Stdio I/O: Wrap `std::process::Command` child stdin/stdout with futures-compatible async wrappers (similar to your `async-fs` pattern)
   - JSON-RPC framing: Hand-rolled (newline-delimited JSON, request/response ID matching)

**3. Skip `agent-client-protocol-tokio`** (it hardcodes tokio); write minimal subprocess adapter for your executor.

---

## Trade-Offs

| Choice | Pro | Con |
|--------|-----|-----|
| Reuse SDK + hand-roll subprocess | Official types, easy JSON-RPC, minimal deps | ~500–800 LOC for subprocess I/O + framing |
| Reuse SDK + smol island | Proven runtime, subprocess isolated | Runtime boundary overhead, harder debugging |
| Hand-roll ACP | Full control | Reimplements JSON-RPC, duplicates npm SDK logic (loses YAGNI) |

---

## Unresolved Questions

1. Does `agent-client-protocol-schema` expose `session/set_mode`, `session/set_config_option`, `terminal/release` as Rust types, or are they missing from the SDK? *→ Check schema crate docs.*
2. What is the exact breaking change if agents spawned with `async-task` miss backpressure signals from the editor (e.g., if stdio buffer fills)? *→ Test with real ACP agents.*
3. Is there a boltz-scheduler example for wrapping subprocess stdio? *→ Check `crates/scheduler` in your repo.*

---

## References

- [Official ACP Spec](https://agentclientprotocol.com/protocol/v1/overview)
- [Rust SDK GitHub](https://github.com/agentclientprotocol/rust-sdk)
- [agent-client-protocol crate](https://crates.io/crates/agent-client-protocol)
- [Zed ACP Integration](https://zed.dev/acp)
- [Async Rust Ecosystem](https://rust-lang.github.io/async-book/08_ecosystem/00_chapter.html)
- [async-task](https://crates.io/crates/async-task)
- [async-executor](https://crates.io/crates/async-executor)
- [smol](https://github.com/smol-rs/smol)
