# Phase 7: Windows Batch-Shell Agent Spawn + Claude Executable Resolution

## Context links

- Plan: [plan.md](./plan.md)
- Research: dedicated research pass (Windows spawn, Qoder, misc LOW gaps group)
- Original port plan phase to correct after this lands: [phase-02-protocol-transport-lifecycle.md](../20260705-1718-acpx-to-acp-crate-port/phase-02-protocol-transport-lifecycle.md) (gaps 21,27)
- Parallel (once HIGH tier is merged): [Phase 4](./phase-04-session-lifecycle-reconnect-model-state.md), [Phase 5](./phase-05-runtime-contract-dynamism.md), [Phase 6](./phase-06-conversation-fidelity-client-operations.md) (file-disjoint)

## Scope boundary

Only touch: `crates/acp/src/client/spawn.rs`, `crates/acp/src/agent_command/spawn_options.rs`. No other files. (No `windows_quirks.rs` this pass — see ADR-10; the wrapper-script-sniffing work it would have held is deferred.)

## Overview

- **Priority:** P2 (MEDIUM — Windows is a genuinely supported target platform for this workspace, confirmed via `docs/system-architecture.md`, `docs/project-overview-pdr.md`, and the real `crates/gpui_windows` workspace member; this is a real functional gap on a supported platform, not a "Windows out of scope" case)
- **Status:** pending
- **Description:** The real agent-process spawn path (`client/spawn.rs`) has zero Windows `.cmd`/`.bat` batch-shell wrapping logic — an npx-based agent (codex/claude/mux/pi/kilocode/opencode) resolving to a `.cmd` shim will fail to spawn correctly on Windows without a shell wrapper. `agent_command/spawn_options.rs`'s module doc comment **falsely claims** this logic (`ShellWrap`/`wrap_for_windows_batch_shell`) already exists — it does not (confirmed via grep: neither symbol exists anywhere in the crate). The two pure helpers that DO exist (`resolve_windows_command`, `should_use_windows_batch_shell`) are wired only into the terminal-tool spawn path (`terminal/spawn.rs`), not the agent-process spawn path.
- **Scope decision (locked in, see ADR-10):** this phase ports (a) the batch-shell wrapping of the real agent-spawn path only. (b) acpx's wrapper-script-content-sniffing half (`resolveWindowsWrapperExecutable`) is explicitly deferred — see ADR-10 for why — and left as a documented `// TODO(gap-21b)` rather than implemented speculatively without a way to test it for real.

## Key Insights (from research)

- `client/spawn.rs::spawn_agent_process` builds a `util::command::Command` and calls `Child::spawn` directly — no `.cmd`/`.bat` detection or shell-wrapping of any kind.
- `spawn_options.rs`'s existing functions: `resolve_windows_command(command, env, exists) -> Option<PathBuf>` (PATHEXT-based candidate resolution — the *simpler* half of acpx's `resolveWindowsExecutablePath`, missing the wrapper-script-sniffing half entirely), `should_use_windows_batch_shell(command, env, exists) -> bool`, `build_terminal_shell_spawn_command(command, is_windows) -> (String, Vec<String>, bool)` — this last one IS wired into `terminal/spawn.rs`, proving the wiring *pattern* already exists somewhere in the crate; it's just never applied to the agent-spawn path.
- acpx's actual wiring point: `spawn-command-options.ts`'s `buildSpawnCommandOptions(command, options)` — `if (!shouldUseWindowsBatchShell(command, platform, env)) return options; return {...options, shell: true};` — called from `agent-command.ts:205` inside the real agent `spawn(...)` call. This is a **small, targeted** change (wrap the options object passed to `spawn`), not a rewrite of the spawn path.
- `resolveWindowsExecutablePath` (`agent-command.ts:110-132`) — the fuller version — resolves `.cmd`/`.bat`/`.ps1` shims to either a sibling `.exe` or by parsing the wrapper script's content for an embedded `.exe` target (`resolveWindowsWrapperExecutable`/`resolveWindowsWrapperToken`, L60-88). This wrapper-script-content-sniffing half is genuinely more complex (reads and parses a script file) and is entirely unported — flagged as a scope question in plan.md (Unresolved Questions #3).
- `resolveClaudeCodeExecutable` (`agent-command.ts:372-383`, gap 27) is a small, Windows-only function layered directly on top of `resolveWindowsExecutablePath`: returns `undefined` on non-Windows or when `CLAUDE_CODE_EXECUTABLE` env override is set, otherwise resolves `"claude"` via the same executable-path resolution this phase is already porting — natural to bundle since it's a 10-line function with no independent design surface once the resolver exists.

## Requirements

1. `agent_command/spawn_options.rs`'s module doc comment is corrected to accurately describe current state (no longer claims `ShellWrap`/`wrap_for_windows_batch_shell` exist) — done regardless of the rest of this phase's scope.
2. `client/spawn.rs`'s real agent-spawn path calls `should_use_windows_batch_shell` (already exists) on the resolved command; when true, wraps the spawn in a shell invocation (Windows `cmd.exe /c`, matching acpx's `shell: true` semantics translated to Rust's `std::process::Command`/`util::command::Command` API — confirm exact mechanism, e.g. setting `.shell(true)` if `util::command::Command` supports it, or manually prefixing `cmd`/`/C` args if not).
3. **Deferred, locked in (ADR-10):** `resolveWindowsExecutablePath`'s wrapper-script-sniffing half and `resolveClaudeCodeExecutable` (gap 27, which depends on it) are NOT implemented in this pass. Add a `// TODO(gap-21b, gap-27): port others/acpx/src/spawn-command-options.ts's resolveWindowsWrapperExecutable/resolveWindowsWrapperToken and others/acpx/src/acp/agent-command.ts's resolveClaudeCodeExecutable once real Windows test infrastructure is available` comment at the top of `spawn_options.rs`, next to the corrected module doc. Do not create `windows_quirks.rs` — there is nothing to put in it this pass; leaving it as a documented TODO instead of an empty file avoids YAGNI-violating scaffolding.
4. All new logic (`should_use_windows_batch_shell`'s call site in `spawn.rs`) is `#[cfg(windows)]`-appropriate where genuinely Windows-only, but must remain unit-testable on non-Windows CI (acpx's own functions take an explicit `platform`/`env` parameter rather than reading `process.platform` directly, specifically to make them testable cross-platform — mirror this: the call site takes an explicit `is_windows: bool` or similar parameter, matching the existing `should_use_windows_batch_shell`/`resolve_windows_command` pattern which already does this).

## Architecture

```
crates/acp/src/
├── client/spawn.rs
│   └── spawn_agent_process: + should_use_windows_batch_shell check before building the
│         util::command::Command; when true, wrap in a shell invocation
└── agent_command/
    └── spawn_options.rs   # doc comment corrected; existing resolve_windows_command /
                            # should_use_windows_batch_shell / build_terminal_shell_spawn_command
                            # unchanged (still used by terminal/spawn.rs too); + TODO comment
                            # for the deferred wrapper-script-sniffing half (gap 21b/27)
```

## ADR Rationale

### ADR-10: Windows batch-shell wrapping ported into the real spawn path; wrapper-script sniffing explicitly deferred (CONFIRMED)

- **Context:** Two genuinely different pieces of work are bundled under "gap 21": (a) actually wrapping the resolved agent command in a shell when spawning on Windows (small, mechanical, directly testable via the existing `should_use_windows_batch_shell` predicate), and (b) parsing a `.cmd`/`.bat` wrapper script's *content* to find its real `.exe` target when a naive PATHEXT lookup doesn't find one (acpx's `resolveWindowsWrapperExecutable`) — this second piece requires reading and pattern-matching an arbitrary shell script's text, a meaningfully larger and more speculative-to-test piece of logic (hard to validate without a real Windows environment and real npm-installed `.cmd` shims to test against).
- **Decision (confirmed):** (a) is in scope for this phase — it's the actual bug (agent spawn fails on Windows without it) and is directly unit-testable via the existing `is_windows`-parameterized helpers. (b), and gap 27 (`resolveClaudeCodeExecutable`, which depends on it), are explicitly deferred — not implemented this pass. Left as a `// TODO(gap-21b, gap-27)` comment referencing the exact TS source functions to port later, once real Windows test infrastructure exists to validate against.
- **Why defer (b)/27 rather than attempt them speculatively:** this crate's own testing convention (established across every earlier phase) is "no mocks, real subprocess/real behavior tests." (b) requires reading and pattern-matching real `.cmd`/`.bat` wrapper-script content produced by real npm installs on a real Windows machine — there is no way to write a genuine, non-speculative test for this without Windows CI access, which this environment doesn't have. Shipping untested, Windows-specific script-parsing logic carries more risk (silently wrong on real installs) than shipping nothing and leaving a clear TODO. (a) alone is still a strict improvement over today (agent spawn currently fails outright on Windows for any `.cmd`-shimmed agent; (a) fixes the common case, `resolve_windows_command`'s existing PATHEXT-only resolution remains the fallback for the narrower case (b) would have covered).

## Related code files

- `crates/acp/src/client/spawn.rs` (`spawn_agent_process`, L28-43).
- `crates/acp/src/agent_command/spawn_options.rs` (module doc L1-16 — false claim to correct; `resolve_windows_command` L54-81, `should_use_windows_batch_shell` L85-98, `build_terminal_shell_spawn_command` L112-134 — read-only reference for the wiring pattern).
- `crates/acp/src/terminal/spawn.rs` (L23-25, L64-68 — existing terminal-path wiring, read-only reference).
- Reference (read-only, for the deferred TODO's benefit, not implemented this pass): `others/acpx/src/spawn-command-options.ts` (`resolveWindowsWrapperExecutable`/`resolveWindowsWrapperToken` L60-88), `others/acpx/src/acp/agent-command.ts` (`resolveClaudeCodeExecutable` L372-383).
- Reference (read-only, implemented this pass): `others/acpx/src/spawn-command-options.ts` (`buildSpawnCommandOptions` L147-160), `others/acpx/src/acp/agent-command.ts` (real agent-spawn call site L205).

## Implementation Steps

1. Fix `spawn_options.rs`'s false module doc comment (Requirement 1) — do first, independent of the rest.
2. In `client/spawn.rs::spawn_agent_process`, call `should_use_windows_batch_shell(command, env, exists_fn)` on the resolved command before building the `util::command::Command`; when true, adjust the spawn to route through a shell (confirm `util::command::Command`'s exact API for this — check if it exposes a `.shell(bool)`-equivalent option, matching `build_terminal_shell_spawn_command`'s already-working approach for the terminal path, and reuse the same mechanism rather than inventing a second one).
3. Unit test: a fake `exists_fn`/`env` simulating a `.cmd`-resolving command on `is_windows=true` produces a shell-wrapped spawn config; the same inputs on `is_windows=false` (or a `.exe`-resolving command on Windows) produce an unwrapped spawn config — mirrors the existing test pattern already used for `should_use_windows_batch_shell`'s own unit tests.
4. **Deferred (ADR-10, confirmed) — do not implement:** add the `// TODO(gap-21b, gap-27)` comment to `spawn_options.rs` referencing `others/acpx/src/spawn-command-options.ts`'s `resolveWindowsWrapperExecutable`/`resolveWindowsWrapperToken` and `others/acpx/src/acp/agent-command.ts`'s `resolveClaudeCodeExecutable` as the functions to port once real Windows test infrastructure is available. No `windows_quirks.rs` file this pass.
5. Real call-path check: since this crate's CI likely runs on macOS/Linux (confirm), a true end-to-end Windows spawn integration test isn't feasible in this environment — the "real call path" requirement here is satisfied by `spawn_agent_process`'s actual code path being exercised with `is_windows` forced true in a test (dependency-injected, not `cfg(windows)`-gated at the call site) against a fake `.cmd`-shaped command, confirming the resulting spawn configuration is shell-wrapped. Note in this phase's Implementation status that true Windows-native validation of the wrapping added here (Steps 1-3) is deferred to whoever has Windows CI access, with a clear TODO, separate from the (b)/gap-27 deferral above.
6. `cargo fmt -p boltz-acp`, `cargo check -p boltz-acp --all-targets --features test-support`, `cargo test -p boltz-acp --features test-support`, `make check-all`.
7. Update `plans/20260705-1718-acpx-to-acp-crate-port/phase-02-protocol-transport-lifecycle.md` per plan.md's housekeeping (gap 21 as fixed; gap 27 as explicitly deferred, not silently dropped).

## Todo list

- [ ] Fix `spawn_options.rs`'s false doc comment.
- [ ] `spawn_agent_process` calls `should_use_windows_batch_shell`, wraps spawn when true.
- [ ] Unit tests: shell-wrap decision on Windows/.cmd vs. non-Windows/.exe.
- [ ] Add `// TODO(gap-21b, gap-27)` deferral comment to `spawn_options.rs` (confirmed deferred, not implemented).
- [ ] Dependency-injected `is_windows=true` test proves `spawn_agent_process`'s real code path shell-wraps a `.cmd`-shaped command.
- [ ] `cargo fmt`, `cargo check`, `cargo test`, `make check-all` green.
- [ ] Correct original plan's Phase 2 status text (gap 21 fixed; gap 27 explicitly deferred).

## Success Criteria

- A test forcing `is_windows=true` against a command that `should_use_windows_batch_shell` classifies as needing wrapping proves `spawn_agent_process`'s actual spawn configuration is shell-wrapped (not just that the pure predicate function returns true in isolation).
- `spawn_options.rs`'s doc comment no longer makes a false claim (grep for `ShellWrap`/`wrap_for_windows_batch_shell` in the doc comment returns nothing, or the comment is rewritten to reference the actual function names) and instead documents the (b)/gap-27 deferral with a `TODO(gap-21b, gap-27)`.

## Risk Assessment

- **No real Windows CI to validate against** — all testing in this phase is dependency-injected (`is_windows: bool` parameters, fake `exists`/env closures), which proves the logic is *correct given accurate inputs* but cannot catch a real-world Windows-specific `util::command::Command` API surprise (e.g. shell-wrapping interacting unexpectedly with argument quoting on real `cmd.exe`). Document this residual risk explicitly rather than claiming full confidence.
- **Deferred scope (gap 21b/27)** means agents whose `.cmd`/`.bat` shim can't be found via simple PATHEXT lookup (i.e. needs wrapper-script-content sniffing to locate the real `.exe`) will still fail to spawn correctly on Windows after this phase — an accepted, documented residual gap, not a silent one.

## Security Considerations

- Shell-wrapping introduces a real injection surface if the wrapped command's arguments aren't quoted correctly for `cmd.exe`'s parsing rules (distinct from POSIX shell quoting, which this crate's `agent_command::command_args` already handles carefully per the original Phase 2's Security Considerations) — verify `build_terminal_shell_spawn_command`'s existing quoting approach (already used for the terminal path) is reused, not reinvented, for the agent-spawn path to avoid introducing a second, potentially inconsistent quoting implementation.

## Next steps

- Proceed to [Phase 8](./phase-08-agent-quirks-shutdown-persistence.md) (LOW tier) once all MEDIUM-tier phases merge.
- No unresolved questions — ADR-10's scope (batch-shell wrapping in, wrapper-script sniffing deferred with a TODO) is confirmed.
