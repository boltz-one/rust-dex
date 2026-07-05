# Goal: Zed UI Component Enrichment — Phase A (Visual Chrome)

## Mission
Implement Phase A only: add chrome-only GPUI components to `crates/ui` (picker/tab-switcher, breadcrumbs, minimal pane-group/dock, title bar) and upgrade `code_editor.rs`'s gutter — zero new dependencies, zero backing-state complexity. Phase B/C (real syntax highlighting, real terminal PTY) are gated by unresolved decisions below; do NOT start them without explicit user go-ahead.

## Context & Key Files
- Full plan: `plans/20260705-1722-zed-ui-component-enrichment/plan.md`
- This phase: `plans/20260705-1722-zed-ui-component-enrichment/phase-01-visual-chrome-components.md` (full detail: ADRs, step order, exact file list)
- Later phases (read before continuing, do not implement yet): `phase-02-real-syntax-highlighting-and-terminal-chrome.md`, `phase-03-real-terminal-pty-and-text-buffer.md`
- Existing code to reuse/audit first: `crates/ui/src/components/{code_editor,command_palette/,tab,tab_bar,resizable,sidebar,breadcrumb}.rs`
- Zed reference (read-only, gitignored, NOT a workspace member — never add as dependency): `others/zed/crates/{picker,tab_switcher,breadcrumbs,title_bar,platform_title_bar}/`

## Requirements
**Must do:**
- Audit `breadcrumb.rs` and `command_palette/` BEFORE creating new files — do not duplicate existing functionality (see phase-01 ADR §1).
- Add `TabSwitcher` (Cmd+Tab-style overlay), `PaneGroup`/N-panel chrome (extend existing `ResizablePanelGroup`, do not port Zed's 48kLOC `workspace` crate), chrome-only `TitleBar` (no real window-API calls — callbacks only).
- Upgrade `code_editor.rs`: dynamic gutter width (currently hardcoded `px(44.)`, breaks on 4+ digit line counts) + current-line highlight.
- Register every new component in `crates/ui/src/components.rs` + add a preview in `examples/ui_gallery`.
- Follow existing conventions: files <200 lines, no `#[cfg(target_os)]` outside platform crates, no new workspace dependency.

**Must not:**
- Do not vendor/copy Zed source directly — its `picker`/`tab_switcher`/`breadcrumbs`/`title_bar` all depend on Zed's `workspace`/`db`/`project`/`client` crates and will not compile standalone. Rewrite from GPUI primitives only.
- Do not implement tab-drag-to-split, cross-pane drag, or item persistence (no `crates/db` in scope).
- Do not touch Phase B/C scope (rope, tree-sitter, terminal PTY, alacritty_terminal).

## Success Criteria
- `cargo check --workspace --all-targets` passes.
- `cargo fmt --all -- --check` passes.
- All new components implement `Render`/`RenderOnce` only — no direct platform API calls.
- No new entries added to `[workspace.dependencies]`.
- Every new component renders without panic in `examples/ui_gallery`.

## Out of Scope
- Real syntax highlighting, real terminal PTY (Phase B/C — blocked on unresolved questions in `plan.md`, especially: accept `alacritty_terminal`/tree-sitter as new deps? refactor `command_palette` into generic `Picker<T>`?).
- Full LSP, workspace persistence/DB, extension host, AI/agent panel, vim mode, collab/remote dev, Zed's `project_panel`/`sidebar`/full `workspace` system (see plan.md "Ngoài phạm vi").

## Verification
```bash
make fmt-check
make check-all
cargo run -p ui_gallery  # manually confirm new component previews render
```
