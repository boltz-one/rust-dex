# Zed Workspace Chrome Components — Portability Analysis

**Scope:** Survey Zed's workspace chrome crates to identify reusable UI components for base project enrichment.  
**Methodology:** LOC count, file structure inspection, type dependency analysis.

---

## Summary of Findings

| Zed Source | Role | Coupling | Phase | Destination @ base |
|---|---|---|---|---|
| **picker** | Fuzzy search overlay (file/command/goto) | MEDIUM | Phase A | `crates/ui/src/components/picker/` |
| **tab_switcher** | Cmd+Tab window switcher | MEDIUM | Phase A | `crates/ui/src/components/tab_switcher/` |
| **breadcrumbs** | File navigation breadcrumbs | MEDIUM | Phase A | `crates/ui/src/components/breadcrumbs/` |
| **title_bar** | MacOS/Win/Linux custom title bars | MEDIUM-HIGH | Phase B | `crates/ui/src/components/title_bar/` + config |
| **platform_title_bar** | Platform-specific title bar logic | MEDIUM | Phase B | `crates/platform_title_bar/` (separate) |
| **outline_panel** | Code outline tree navigator | MEDIUM-HIGH | Phase B | Needs `crates/outline_panel/` + editor adapter |
| **project_panel** | File tree navigator + git status | HIGH | Phase C | Skip (too Zed-specific: Project/Worktree/Git) |
| **sidebar** | Multi-thread/agent sidebar | HIGH | Phase C | Skip (Zed-AI only: agent_ui, ThreadStore) |
| **workspace** | Dock/Pane/Item system | HIGH | Skip | Skip (core Zed, ties to Client/Workspace/collab) |
| **search** | Buffer + project search UI | MEDIUM-HIGH | Phase B | Skip (requires deep editor integration) |

---

## Detailed Assessments

### 1. Picker (3.8 kLOC) — **HIGHEST PRIORITY**
**Role:** Generic fuzzy-search overlay modal (underpins file finder, command palette, go-to-line).  
**Key files:** `picker/src/picker.rs:1-60` (state machine), `render.rs` (layout), `preview.rs` (preview pane), `footer.rs` (action buttons).  
**Coupling:** MEDIUM. Imports `workspace::ModalView` (fixable via trait), `ErasedEditor` (optional, for preview only), GPUI primitives.  
**Porting path:** Extract `picker.rs` + `render.rs` as generic component. Modal view abstraction can accept any `View` for preview. No language/editor dependency required for basic picker.  
**Phase:** Phase A. Pure GPUI component, modal-agnostic.

### 2. Tab Switcher (1.5 kLOC) — **HIGH PRIORITY**
**Role:** Cmd+Tab style floating window switcher between open items/windows.  
**Key files:** `tab_switcher/src/tab_switcher.rs` (~30 kLOC shared with tests).  
**Coupling:** MEDIUM. Depends on action dispatch, GPUI window/focus APIs.  
**Porting path:** Extract as pure GPUI component; caller provides list of items + callbacks. No Zed-specific types needed.  
**Phase:** Phase A.

### 3. Breadcrumbs (127 LOC) — **HIGH PRIORITY**  
**Role:** Clickable file path breadcrumbs for quick navigation.  
**Key files:** `breadcrumbs/src/breadcrumbs.rs` (single file, minimal).  
**Coupling:** MEDIUM. Simple GPUI elements + action dispatch.  
**Porting path:** Direct port. Caller provides path segments + on_click callbacks.  
**Phase:** Phase A.

### 4. Title Bar (2.9 kLOC) — **MEDIUM PRIORITY**
**Role:** Custom native-style title bar + app menu + collab indicators + onboarding banner.  
**Key files:** `title_bar.rs` (57 kLOC), `application_menu.rs` (12 kLOC), `collab.rs` (35 kLOC), `onboarding_banner.rs` (6 kLOC).  
**Coupling:** MEDIUM-HIGH. Heavy dependency on Zed settings, application state (workspace, multi-workspace, collab state). `collab.rs` = heavy Zed-specific UI.  
**Porting path:** Extract title bar chrome (border, buttons, text) as generic component. Deprecate collab/onboarding subcomponents for now; integrate only when base has equivalent state types.  
**Phase:** Phase B. Visual-only port first; stateful integrations later.

### 5. Platform Title Bar (1.2 kLOC) — **MEDIUM PRIORITY**  
**Role:** Platform-specific title bar OS integration (macOS safe area, Win/Linux custom chrome).  
**Coupling:** MEDIUM. Window/platform API calls, minimal Zed types.  
**Porting path:** New crate `crates/platform_title_bar/` with platform module structure. Extract platform detection + safe-area logic.  
**Phase:** Phase B. Utility crate, can be standalone.

### 6. Outline Panel (8.2 kLOC) — **MEDIUM PRIORITY**  
**Role:** Code outline tree (functions, classes, etc.) extracted from current buffer.  
**Key files:** 2 Rust files (settings + main outline logic). Imports: `language::OutlineItem`, `BufferSnapshot`, GPUI list rendering.  
**Coupling:** MEDIUM-HIGH. Tightly bound to `language::OutlineItem` (Zed language abstraction) and `Editor` context.  
**Porting path:** Separate crate `crates/outline_panel/`. Requires abstraction over outline item source (e.g., LSP-based outline if base uses language server). Port visual component + tree state, decouple from Zed's language layer.  
**Phase:** Phase B. Visual component itself is reusable; coupling is data-source (editor), not UI.

### 7. Project Panel (20 kLOC) — **LOW PRIORITY, HIGH EFFORT**  
**Role:** File tree navigator + git diff decorator + file icons + diagnostic overlays.  
**Coupling:** HIGH. Core dependencies: `project::{Project, Worktree, Entry, GitTraversal}`, `editor::Editor`, `git::GitTraversal`, `file_icons::FileIcons`, `markdown_preview::MarkdownPreviewView`.  
**Why skip:** Port requires porting Zed's entire Project/Worktree/Git abstraction. Not feasible for base (which may use different filesystem/VCS layers).  
**Phase:** Phase C+ (defer indefinitely unless base defines equivalent Project model).

### 8. Sidebar (23 kLOC) — **SKIP**  
**Role:** Multi-thread AI sidebar (agent UI for Zed's agentic features).  
**Coupling:** VERY HIGH. Depends on `agent_ui::*`, `ThreadStore`, `ThreadMetadata`, `AcpThreadImportOnboarding`, `agent_client_protocol`.  
**Why skip:** Zed-specific feature; not generic chrome. Base project is orthogonal.

### 9. Workspace System (48 kLOC: item, dock, pane, pane_group) — **SKIP**  
**Role:** Core dockable split-pane system + item persistence.  
**Coupling:** VERY HIGH. `item.rs` ties to `client::Client`, `project::Project`, `workspace::Workspace` (collab-aware), `language::{LanguageServer, Capability}`.  
**Why skip:** Foundation of Zed; porting requires porting entire Zed workspace model (unfeasible). Base should design own dock/pane system from GPUI primitives.

### 10. Search (12 kLOC) — **LOW PRIORITY**  
**Role:** Buffer search + project-wide search UI.  
**Coupling:** MEDIUM-HIGH. Depends on `editor::Editor`, `project::Project`, `BufferSnapshot`, regex/matching state.  
**Why defer:** Requires deep editor/project integration. Can be ported later once base editor architecture stabilizes.

---

## Phasing Recommendation

**Phase A (UI-heavy, pure GPUI):**
- ✅ `picker` → `crates/ui/src/components/picker/`
- ✅ `tab_switcher` → `crates/ui/src/components/tab_switcher/`
- ✅ `breadcrumbs` → `crates/ui/src/components/breadcrumbs/`

Effort: ~3–4 weeks (extract, adapt ModalView, validate with command palette reuse).

**Phase B (UI + minimal state):**
- Title bar chrome (extract visual, defer collab/onboarding).
- Platform title bar (separate crate, generic).
- Outline panel (visual tree component; defer editor integration).

Effort: ~2–3 weeks (visual extraction + adapter traits).

**Phase C (defer):**
- project_panel (requires Project/Worktree abstraction).
- search (requires editor integration).
- workspace chrome (out of scope; design from GPUI).

---

## Key Dependencies to Inject

1. **ModalView trait** (for picker): Replace `workspace::ModalView`. Define trait with `render()` + `is_modal()`.
2. **Editor abstraction** (for outline): Define `Outline` trait + `OutlineProvider` for language-agnostic code navigation.
3. **File icon registry** (for breadcrumbs, future panel): Reuse existing `crates/ui/src/components/icon.rs` or port `file_icons` subset.

---

## Unresolved Questions

- Does base project plan to support split-pane layout? If yes, design independently; don't port workspace system.
- Will base support file-tree navigation (project_panel-like)? Requires design of Project model first.
- Are language servers (for outline) in scope? Affects outline_panel timing.
