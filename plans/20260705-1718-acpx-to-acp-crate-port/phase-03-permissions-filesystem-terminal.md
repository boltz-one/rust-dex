# Phase 3: Permissions + Filesystem + Terminal Manager

## Context links

- Plan: [plan.md](./plan.md)
- Previous: [Phase 2](./phase-02-protocol-transport-lifecycle.md) (needs `agent_command::spawn_options`, `client` transport)
- Next: [Phase 4](./phase-04-runtime-engine-public-contract.md)
- Research: [researcher-02-acpx-architecture.md](./research/researcher-02-acpx-architecture.md) §5

## Overview

- **Date:** 2026-07-05
- **Description:** Port the permission decision engine (mode + policy rules + escalation), filesystem RPC handlers (`fs/read_text_file`, `fs/write_text_file` with cwd sandboxing), and the terminal manager (`terminal/create|output|kill|release`, process-group tracking). The permission-prompt mechanism is **not** ported as-is (stdin TTY prompt) — replaced by an async/callback API per ADR-6, since this crate must never block the GPUI event loop.
- **Priority:** P1 (blocks Phase 4's runtime engine, which wires these handlers into `AcpRuntimeOptions`)
- **Implementation status:** Done (2026-07-05). `permissions/` (policy, escalation, response, decision, resolve, confirm, responder), `filesystem.rs`, `terminal/` (mod, spawn, tracking, kill, output) implemented with real cwd-sandboxed filesystem handlers, a `TerminalManager` driving real subprocesses, and the ADR-6 `PermissionRequestHandler`/`ChannelPermissionRequestHandler` async permission API. `cargo check -p boltz-acp`, `cargo test -p boltz-acp --features test-support` (205 unit + 3 integration tests), `cargo fmt --all -- --check`, and `make check-all` all green. See implementation report for deviations (terminal descendant-pid tracking simplified per ADR-3 reuse, symlink-escape policy = reject, `_meta` escalation-metadata embedding omitted in favor of the typed `escalation` field, terminal spawn's ENOENT-retry-to-shell replaced with an upfront heuristic).
- **Review status:** Not reviewed
- **Audit correction (2026-07-06):** a 4-way independent completeness audit found this phase's "done" status was inaccurate. Gap 1: `PermissionPolicy` (fully implemented in `permissions/policy.rs`) is never threaded to the RPC handler — `client/handshake.rs:216` hardcodes `policy: None` and calls the response-only `resolve_permission_request` wrapper instead of the full `resolve_permission_request_with_details` decision tree. Gap 2: consequently, `PermissionEscalationEvent` (the audit trail this phase's own Security Considerations section called out as important) never surfaces to any caller — the value is computed and immediately discarded. Gap 25: `PermissionStats` (requested/approved/denied/cancelled counters) was never ported at all — no Rust equivalent exists. Gap 32: `permissions/resolve_tests.rs` is missing direct test coverage for the `PermissionPolicyAction::Approve` arm and for policy-override behavior against `DenyAll`/`ApproveReads` modes (only `ApproveAll`-override and `Deny`/`Escalate` actions are tested). All fixes tracked in `plans/20260706-0106-acp-completeness-fixes/`: gaps 1/2/25 → [phase-01](../20260706-0106-acp-completeness-fixes/phase-01-permission-policy-authenticate-wiring.md), gap 32 → [phase-09](../20260706-0106-acp-completeness-fixes/phase-09-test-coverage-adr-cleanup.md). The underlying decision logic in `permissions/` itself is confirmed correct by the audit — only the wiring/test-coverage gaps above need fixing.

## Key Insights

- acpx's permission resolution (`permissions.ts`, 443 lines) is pure decision logic (mode rank comparison, policy rule matching, tool-kind inference from title heuristics) — this ports near 1:1, no I/O.
- The *only* I/O-touching part is `permission-prompt.ts` (33 lines, stdin TTY prompt) — this is explicitly a GUI-incompatible pattern (`process.stdin.isTTY` checks make no sense in a GPUI app) and is replaced wholesale by ADR-6's async API.
- `terminal-manager.ts` (884 lines) is the largest file in this phase's scope. It tracks descendant PIDs via a process-group snapshot for kill purposes, tracks output byte limits (64KB default) with truncation, and has separate kill-grace timing (1.5s default) similar to the client shutdown sequence from Phase 2 — reuse `util::process`/`control::with_timeout` rather than re-deriving grace-period logic a second time.
- `filesystem.ts` (236 lines) enforces `cwd` sandboxing before every read/write — this must be preserved exactly; a Rust path-traversal bug here (e.g. not canonicalizing `..` before the boundary check) would be a real security regression versus acpx.
- Permission mode ranking (`deny-all < approve-reads < approve-all`) and non-interactive policy (`"deny" | "fail"`) types already landed in Phase 1's `types.rs` — this phase adds the decision *logic*, not the type definitions.

## Requirements

1. `resolve_permission_request(params, mode, non_interactive_policy, policy) -> ResolvedPermissionRequest` ports acpx's `resolvePermissionRequestWithDetails` decision tree exactly (policy match → mode match → read-or-prompt fallback), returning an escalation event when applicable.
2. When a decision requires interactive input (the old `canPromptForPermission()` branch), the Rust port calls an injected async handler instead of reading stdin — see ADR-6.
3. `fs/read_text_file` / `fs/write_text_file` handlers enforce `cwd` sandboxing using canonicalized paths (reject any resolved path outside the session's cwd boundary, including via symlinks — acpx's Node `fs` behavior around symlinks should be checked and matched or deliberately tightened, flagged in Security Considerations).
4. Terminal manager: `terminal/create` spawns via `agent_command::spawn_options` (Phase 2) + `util::command`; `terminal/output` returns buffered output up to the byte limit with a `truncated` flag; `terminal/kill` / `terminal/release` follow the same grace-period escalation as Phase 2's client shutdown.
5. All permission/filesystem/terminal operations are gated by the same `PermissionMode`/`NonInteractivePermissionPolicy`/`PermissionPolicy` as acpx (deny-all/approve-reads/approve-all ranking preserved).

## Architecture

```
crates/acp/src/
├── permissions/
│   ├── mod.rs          # resolve_permission_request(), decision_to_response(), classify_decision()
│   ├── policy.rs        # PermissionPolicy matching (autoApprove/autoDeny/escalate rule lists),
│   │                     # ports permission-policy.ts's parse + permissions.ts's matchPermissionPolicy
│   └── responder.rs      # ADR-6: PermissionRequestHandler trait + PermissionResponder (oneshot-backed)
│                          # — replaces permission-prompt.ts entirely
├── filesystem.rs         # fs/read_text_file, fs/write_text_file + cwd sandbox canonicalization
└── terminal/
    ├── mod.rs            # TerminalManager public API: create/output/kill/release/wait_for_exit
    ├── spawn.rs           # uses agent_command::spawn_options + util::command (Phase 2 reuse)
    ├── tracking.rs        # descendant-pid / process-group tracking for kill
    └── output.rs          # buffered output + byte-limit truncation
```

## ADR Rationale

### ADR-6: Permission-request API is async/channel-based, never blocks the GPUI event loop

- **Context:** acpx's `permission-prompt.ts` calls `promptForPermission()`, which does a synchronous-feeling stdin read gated by `process.stdin.isTTY && process.stderr.isTTY`. A GUI has no stdin TTY and, more importantly, GPUI's render loop must never block on user input arriving from an arbitrary future — an interactive permission prompt in a GUI is *inherently* a "show a dialog, wait for the user to click, resume the agent's pending RPC response" flow that can take arbitrarily long (seconds to indefinite).
- **Decision:** Define `trait PermissionRequestHandler: Send + Sync { fn request(&self, req: PermissionRequestParams) -> BoxFuture<'static, PermissionDecision>; }` (or channel-equivalent). The runtime engine (Phase 4) is constructed with a `Arc<dyn PermissionRequestHandler>`. When a decision needs interactive input, `permissions::mod.rs` calls `handler.request(...)` and `.await`s it — this await point suspends only the specific in-flight `session/prompt` RPC's response, not the GPUI event loop, because it runs on the `smol`-driven task from Phase 2/ADR-2, not on GPUI's UI thread. The GPUI app implementation of `PermissionRequestHandler` sends the request over a channel to the UI, shows a dialog, and resolves a `futures::channel::oneshot::Sender` when the user responds; `request()`'s returned future awaits the paired `oneshot::Receiver`.
- **Why this over alternatives:** (a) A synchronous callback (`Fn(...) -> PermissionDecision` with no `async`) would force the *caller* (deep inside the transport read loop) to block a thread waiting for UI interaction — either freezes that smol worker thread or, if that worker also drives other sessions' I/O, stalls unrelated sessions too. (b) A polling API (`fn poll_decision() -> Option<PermissionDecision>`, called repeatedly) pushes complexity onto every caller and reintroduces stdin-prompt-style busy-waiting antipatterns. (c) The channel/future approach matches acpx's own `onPermissionRequest` callback shape (`Promise<AcpPermissionDecision | undefined>`) almost exactly — same conceptual API, just made non-blocking-safe for the host runtime it now lives in, minimizing behavioral surprise for anyone who knows the TS runtime contract.

## Related code files

- `others/acpx/src/permissions.ts` (443 lines) — primary decision-logic source.
- `others/acpx/src/permission-policy.ts` (103 lines) — policy parsing.
- `others/acpx/src/permission-prompt.ts` (33 lines) — reference only, replaced by ADR-6, not ported.
- `others/acpx/src/filesystem.ts` (236 lines).
- `others/acpx/src/acp/terminal-manager.ts` (884 lines) — primary source for `terminal/`.
- `others/acpx/src/spawn-command-options.ts` (183 lines, ported in Phase 2, reused here).
- `crates/util/src/command.rs`, `crates/util/src/process.rs` — reused for terminal subprocess spawn/kill (same as Phase 2's ADR-3).
- `others/acpx/test/permissions.test.ts`, `test/permission-prompt.test.ts`, `test/filesystem.test.ts`, `test/terminal.test.ts` — behavior-spec reference for what the Rust tests should check (not ported directly).

## Implementation Steps

1. Port `permissions::policy` (rule-list matching, tool-kind inference from title heuristics — the `TOOL_KIND_TITLE_MATCHERS` table) as a direct translation.
2. Port `permissions::mod` decision tree: policy match → mode match (`approve-all`/`deny-all` short-circuits) → read-or-prompt fallback (auto-approve `read`/`search` tool kinds, else defer to the async handler or fail per `NonInteractivePermissionPolicy`).
3. Define `PermissionRequestHandler` trait + `PermissionResponder`/oneshot plumbing per ADR-6 in `permissions/responder.rs`.
4. Port `filesystem.rs`: implement cwd sandboxing using `std::path::Path::canonicalize` (or a symlink-aware equivalent — see Security Considerations) before comparing against the session boundary; wire as ACP SDK `Client` trait method implementations (from Phase 2's transport).
5. Port `terminal/spawn.rs`: build the terminal's command via `agent_command::spawn_options::build_terminal_spawn_command` (ported in Phase 2) + `util::command`.
6. Port `terminal/tracking.rs`: process-group snapshotting for descendant-pid kill — reuse `util::process`'s POSIX `killpg` primitive; document the same inherited Windows job-object gap noted in Phase 2's Risk Assessment.
7. Port `terminal/output.rs`: buffered stdout/stderr capture with the 64KB default byte limit and `truncated` flag; decide (per Unresolved Question #3) whether output is captured eagerly (poll-style, matching acpx's `TerminalOutputRequest`) or streamed — implement poll-style first as the safe default, structure the code so streaming can be added without an API break (e.g. return type is `TerminalOutputSnapshot`, not a raw string, from day one).
8. Integration tests: real subprocess terminal (e.g. spawn `echo` / `sleep`), verify output truncation at the byte limit, verify kill grace-period escalation (reuse the fake-agent-adjacent test infra from Phase 2 or a plain `std::process` test fixture — terminal-manager doesn't need a fake ACP agent, just any real child process).
9. Filesystem sandbox test: attempt a `../` traversal read/write outside cwd, confirm rejection; attempt a symlink pointing outside cwd, confirm the chosen policy (reject or allow — pick one deliberately, document in Security Considerations, don't leave it as accidental behavior).
10. `cargo fmt`, `cargo check -p boltz-acp`, `make check-all`.

## Todo list

- [x] Port `permissions::policy`, `permissions::mod` decision tree (split into `policy.rs`/`escalation.rs`/`response.rs`/`decision.rs`/`resolve.rs`).
- [x] Define `PermissionRequestHandler`/`PermissionResponder` (ADR-6) — `permissions/responder.rs`, plus `ChannelPermissionRequestHandler` as a ready-to-use GPUI-agnostic implementation.
- [x] Port `filesystem.rs` with cwd sandboxing.
- [x] Port `terminal/{spawn,tracking,output}.rs` (plus `terminal/kill.rs` for the kill-escalation logic, split out for file-size hygiene).
- [x] Decide and document symlink-escape policy for filesystem sandbox — rejected (canonicalize-before-compare).
- [x] Integration tests: permission decision tree (all 3 modes x policy present/absent), filesystem sandbox escape attempts, terminal output truncation, terminal kill escalation.
- [x] All new files < 200 lines where possible (split `permissions/mod.rs` and `terminal/mod.rs`/`terminal-manager.ts`'s port into multiple focused files). A few files (`filesystem.rs` 203, `permissions/policy.rs` 211, `permissions/resolve_tests.rs` 216, `terminal/mod.rs` 226) land slightly over 200 after `cargo fmt` re-wrapping; further splitting was judged to hurt cohesion more than the overage hurts readability — see implementation report.
- [x] `cargo check -p boltz-acp`, `make check-all`, `cargo fmt --all -- --check` green.

## Success Criteria

- Permission decision tree unit tests cover all mode x policy-action combinations from acpx's `test/permissions.test.ts` behavior spec.
- Filesystem sandbox rejects a `../../etc/passwd`-style traversal attempt in a real (non-mocked) test.
- Terminal integration test spawns a real long-running child, confirms output truncates at the configured byte limit, confirms kill actually terminates the process group (checked via `is_process_alive` from Phase 1).
- `PermissionRequestHandler` is exercised end-to-end in a test using a fake handler that resolves after a delay, proving the call doesn't block other concurrent work (e.g. a second unrelated future on the same executor makes progress while the permission future is pending).

## Risk Assessment

- **Symlink-based sandbox escape:** if `canonicalize()` isn't applied consistently before the boundary check, a symlink inside the cwd pointing outside it could bypass sandboxing. This is a genuine security regression risk if not handled deliberately — see Security Considerations.
- **Terminal descendant tracking gap on Windows:** same inherited gap as Phase 2's ADR-3 (no job-object cleanup) — a killed terminal's grandchildren may survive on Windows.
- **Output buffering memory growth:** an agent-spawned process that produces output faster than it's drained (if a bug in `output.rs` fails to enforce the byte cap before buffering) could balloon memory. Enforce the cap at the point bytes are appended, not just at read time.

## Security Considerations

- **Path canonicalization ordering:** canonicalize the requested path *before* the boundary check, not after — canonicalizing after would resolve `..`/symlinks post-hoc and could report a false "safe" boundary based on a pre-resolution string compare. Get this ordering right; it's the single most important line of this phase.
- **Symlink escape decision:** explicitly decide whether a symlink inside cwd pointing outside cwd is permitted or rejected, and document the choice — don't let it be accidental based on whichever `canonicalize` behavior Rust's stdlib happens to produce.
- **Permission escalation audit trail:** port acpx's `PermissionEscalationEvent` emission (tool name/kind/matched-rule/timestamp) faithfully — this is the audit log a user might rely on to understand why an action was auto-approved or denied.
- **Terminal command injection:** same concern as Phase 2's `agent_command::command_args` — `spawn_options`'s terminal shell-spawn path (`buildTerminalShellSpawnCommand`) must preserve acpx's quoting/escaping, not naively concatenate strings into a shell invocation.

## Next steps

- Proceed to [Phase 4](./phase-04-runtime-engine-public-contract.md), which wires `PermissionRequestHandler`, `filesystem`, and `TerminalManager` into `AcpRuntimeOptions`.
- Unresolved question carried forward: **terminal output streaming vs polling** — Step 7 implements polling first; confirm with user whether live streaming is a hard requirement before Phase 4/6 lock in the public contract's event shape around it.
- Unresolved question carried forward: **terminal manager scope** — is full descendant-process-group tracking required for v1, or can Phase 3 ship with direct-child-only kill and defer descendant tracking? Affects `terminal/tracking.rs` scope and test surface.
