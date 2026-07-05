# acpx TypeScript → `acp` Rust Port: Architecture Research

## Executive Summary
The acpx TypeScript CLI is a headless ACP (Agent Client Protocol) client managing subprocess agent communication via ndjson over stdio. To port specific components to Rust crate `crates/acp`, the Rust implementation must preserve:
- JSON-RPC 2.0 message framing (request/response/notification types)
- Subprocess lifecycle (spawn → initialize → session → close)
- Session reuse/reconnect policies on crash
- Permission approval flow (deny-all/approve-reads/approve-all modes)
- On-disk session persistence with versioning and backward compatibility

---

## 1. JSON-RPC & Transport Layer

**Message Format** ([jsonrpc.ts](file:others/acpx/src/acp/jsonrpc.ts), [client.ts](file:others/acpx/src/acp/client.ts:~250))

- **Protocol**: ndjson (newline-delimited JSON) over child process stdio (`stdin`/`stdout`)
- **JSON-RPC 2.0** conformance:
  - All messages require `{"jsonrpc": "2.0"}`
  - **Requests**: have `id` (string|number|null) + `method` + optional `params`
  - **Responses**: have `id` + either `result` or `error` object (mutually exclusive)
  - **Notifications**: have `method` but **no** `id`

**Error Code Mapping** ([jsonrpc-error.ts](file:others/acpx/src/acp/jsonrpc-error.ts:3-15))

```
NO_SESSION → -32002
TIMEOUT → -32070
PERMISSION_DENIED → -32071
PERMISSION_PROMPT_UNAVAILABLE → -32072
RUNTIME → -32603 (generic server error)
USAGE → -32602 (invalid params)
```

**Key Parsers** ([jsonrpc.ts](file:others/acpx/src/acp/jsonrpc.ts:63-85))
- `isAcpJsonRpcMessage()`: validates structure & `jsonrpc: "2.0"`
- `isSessionUpdateNotification()`: detects `method === "session/update"` (server → client)
- `isJsonRpcRequest/Response/Notification()`: type guards for routing

**Requirement for Rust port**: Build idiomatic serde-based message types; preserve error code mappings; implement incremental line-buffered parsing (stdio is streamed, not atomic).

---

## 2. Client Lifecycle

**Spawn & Handshake** ([client.ts](file:others/acpx/src/acp/client.ts:~400), [client-process.ts](file:others/acpx/src/acp/client-process.ts))

1. Spawn subprocess via `node:child_process.spawn(command, args, options)`
   - Stdio: `['pipe', 'pipe', 'pipe']` (stdin, stdout, stderr)
   - Working directory, env vars, auth credentials passed
2. Wait for spawn readiness
3. Wrap stdio in ndjson stream (TextDecoder buffering + line-split)
4. Create `ClientSideConnection` (ACP SDK) wrapping the stream
5. Send `initialize` request with client name, version, capabilities (fs, terminal)

**Capability Negotiation** ([client.ts](file:others/acpx/src/acp/client.ts:~180))
- Client advertises: fs read/write, terminal support
- Server responds with agent capabilities: `loadSession`, `resumeSession`, `closeSession`, `listSessions`
- Model state from response (`configOptions` or legacy `models` metadata)

**Session Creation** ([client.ts](file:others/acpx/src/acp/client.ts:~550))
- POST `sessions/create` → returns `sessionId`, optional `configOptions`
- Returns `SessionCreateResult` with model state info

**Shutdown** ([client.ts](file:others/acpx/src/acp/client.ts:~1000))
- Close stdio pipes
- SIGTERM agent (1.5s grace), then SIGKILL (1s grace)
- Record exit code, signal, reason (process_exit, process_close, pipe_close, connection_close)

**State Tracked** ([client.ts](file:others/acpx/src/acp/client.ts:~225-240))
- `activePrompt`: current prompt promise + sessionId (singular, not queue)
- `agentStartedAt`: ISO timestamp of spawn
- `lastAgentExit`: exit info (code, signal, reason, unexpectedDuringPrompt flag)
- `lastKnownPid`: for reporting

---

## 3. Runtime Engine & Public Contract

**Session Manager** ([runtime/engine/manager.ts](file:others/acpx/src/runtime/engine/manager.ts) preview)
- Orchestrates connected-session lifecycle
- Loads persisted `SessionRecord` via callback
- Creates/reuses `AcpClient` instances
- Applies `SessionResumePolicy` to decide reuse vs. new session
- Saves record after each prompt/state change

**Reconnect Logic** ([runtime/engine/reconnect.ts](file:others/acpx/src/runtime/engine/reconnect.ts) inferred)
- `connectAndLoadSession()`: resume on crash by calling `sessions/load` or `sessions/resume`
- Handles agent restart transparently
- Restores model state from config options

**Public API Contract** ([runtime/public/](file:others/acpx/src/runtime/public/) + [engine/manager.ts](file:others/acpx/src/runtime/engine/manager.ts:~20))
- `WithConnectedSessionOptions<T>`: host app provides:
  - `sessionRecordId`: string (unique key)
  - `loadRecord()`: async callback to retrieve persisted state
  - `saveRecord()`: async callback to persist after changes
  - `permissionMode`, `nonInteractivePermissions`, `onPermissionRequest` callbacks
  - `resumePolicy`: how aggressively to reuse sessions
- Returns methods: `setSessionModel()`, `setSessionConfigOption()`, prompt/close/etc.

**Model State** ([session/model-state.ts](file:others/acpx/src/session/model-state.ts) inferred)
- `configId` (string): selected model configuration ID
- `availableModels`: array of model specs from agent
- Built from `configOptions` (ACP standard) or legacy `models` metadata

---

## 4. Session Persistence

**On-Disk Format** ([session/persistence/](file:others/acpx/src/session/persistence/), [types.ts](file:others/acpx/src/types.ts))
- `SessionRecord` is JSON blob with:
  - `sessionId`: unique ACP session ID
  - `acpx`: acpx-specific metadata (model state, cwd, env)
  - `conversation`: message history (see below)
  - `agentMetadata`: server-provided capabilities snapshot
  - `lastActivity`: ISO timestamp

**Versioning & Backward Compatibility** ([persisted-key-policy.ts](file:others/acpx/src/persisted-key-policy.ts))
- Policy object defines which fields are essential, optional, deprecated
- `parse()`: ignores unknown fields; fills missing optional fields with defaults
- `serialize()`: preserves all fields for forward compatibility
- Corruption handling: graceful degradation (skip missing history, keep session alive)

**Conversation Model** ([session/conversation-model.ts](file:others/acpx/src/session/conversation-model.ts:~20-25))
- Max 200 runtime messages (MAX_RUNTIME_MESSAGES=200)
- Truncates old messages to fit within token/char limits:
  - Max agent text: 8,000 chars
  - Max thinking: 4,000 chars
  - Max tool I/O: 4,000 chars
  - Max request tokens logged: 100
- Preserves latest N messages + usage cost aggregates

**Repository** ([session/persistence/repository.ts](file:others/acpx/src/session/persistence/repository.ts) inferred)
- Reads/writes SessionRecord from `.acpx/sessions/{sessionRecordId}.json`
- Atomic writes (write-to-temp, rename)
- Index file for fast session list

---

## 5. Permissions & Terminal

**Permission Modes & Policy** ([permissions.ts](file:others/acpx/src/permissions.ts:~30), [permission-policy.ts](file:others/acpx/src/permission-policy.ts))

Modes: `"deny-all"` < `"approve-reads"` < `"approve-all"` (ranked)
- **deny-all**: reject all operations
- **approve-reads**: auto-approve read_file, but prompt for writes/terminal
- **approve-all**: auto-approve all

**Non-Interactive Policy** ([types.ts](file:others/acpx/src/types.ts) inferred)
- `{ fs?: { read?: boolean; write?: boolean }; terminal?: { ... } }`
- Used in scripts to define approval patterns without prompts

**Permission Request Flow** ([permissions.ts](file:others/acpx/src/permissions.ts:~50-150))
1. Agent sends `requestPermission` RPC with operation type, path, reason
2. Client checks: mode, policy rules, escalation history
3. If auto-approved: respond immediately
4. If needs prompt: call `promptForPermission()` (interactive stdin)
5. Track stats: requested, approved, denied, cancelled
6. Emit `PermissionEscalationEvent` for audit (including matched rule, reason)

**Terminal Manager** ([acp/terminal-manager.ts](file:others/acpx/src/acp/terminal-manager.ts) inferred)
- Handles `createTerminal`, `killTerminal`, `releaseTerminal`, `waitForTerminalExit`
- Cwd sandboxing: enforce `options.cwd` as root
- Streams `TerminalOutputRequest` for live stdout/stderr
- Permissioned like file operations (deny-all/approve-reads/approve-all)

**FileSystem Handlers** ([filesystem.ts](file:others/acpx/src/filesystem.ts) inferred)
- Wraps `readTextFile()`, `writeTextFile()` RPC handlers
- Enforces cwd sandboxing
- Permission checks before execution
- Returns structured errors (permission denied, file not found, etc.)

---

## 6. Prompt Queueing & Cancellation Semantics

**Current Design** ([client.ts](file:others/acpx/src/acp/client.ts:~240-250))
- **Single active prompt per client** (not per-session): `activePrompt` field holds one Promise
- `hasActivePrompt(sessionId?)`: query if any prompt active, or for specific session
- Cancellation: `cancellingSessionIds` Set tracks sessions being cancelled (not queued)
- **Session update chain**: `sessionUpdateChain` ensures serial processing of server updates

**Ordering Guarantees**:
1. Only one prompt at a time per client (serial execution)
2. Session updates process in order (queued on promise chain)
3. Permission requests may interleave with active prompt (separate RPC channel)
4. Cancellation cancels the active prompt immediately, not queued operations

**For Rust port**: In-process use case likely needs per-session queues, not global. Preserve serial session-update semantics but decouple prompt queue from permission requests.

---

## 7. Rust Idiom Reference

(http_client crate not found in this workspace; no direct pattern to cite.)

Infer from TypeScript style: favor **async/await with futures** (Tokio for async runtime), **builder pattern for clients** (fluent API for options), **typed Result<T, E> for errors**, **serde for JSON serde**. Avoid mocking; use real stdio/process APIs.

---

## Key Files to Port/Reference

| File | Purpose |
|------|---------|
| `src/acp/jsonrpc.ts` | Message parsing, error code defs |
| `src/acp/client.ts` | Main client lifecycle, session ops |
| `src/acp/client-process.ts` | Subprocess spawn, stdio wrapping |
| `src/runtime/engine/manager.ts` | Session reuse, reconnect logic |
| `src/session/persistence/repository.ts` | On-disk persistence |
| `src/permissions.ts` | Approval decision logic |
| `src/types.ts` | Core TypeScript types (map to Rust structs) |

---

## Unresolved Questions

1. **MCP Server Integration**: What's the scope of `mcpServers` in `WithConnectedSessionOptions`? Is it in scope for Rust port phase 1?
2. **Devin/Gemini/Copilot Variants**: Should Rust port support agent-specific command-line logic (buildQoderAcpCommandArgs, resolveGeminiCommandArgs), or defer to phase 2?
3. **Terminal Streaming**: Is `TerminalOutputRequest` (streaming model output during prompt) required, or is polling after completion sufficient?
4. **Legacy Model Metadata**: How long must backward compat with pre-configOptions model metadata be maintained in Rust?

## Trade-offs

- **Single global queue vs. per-session queues**: Current design uses single client prompt queue; in-process use more naturally requires per-session isolation. Decide scope boundary: does `acp` crate expose high-level session API or low-level RPC client?
- **Error handling**: TypeScript throws rich custom error classes (RequestError, etc.); Rust can use enums but will sacrifice some stack context—consider wrapping with `anyhow` for detailed error chains vs. structured error types for client error handling.
- **Permissions interactivity**: Prompting on stdin blocks the runtime; consider async permission request API (callback-based) to avoid blocking the main event loop in GPUI integration.
