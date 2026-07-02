# Phase 05 — Navigation Components + Gallery App

## Context Links

- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-01-tailwind-spec.md` (§2.10)
- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-02-gpui-codebase.md` (§4, §5 — screenshot + workspace wiring)
- Skill: `.claude/skills/gpui-ui-design/references/app-bootstrap.md`, `views-and-state.md`
- Phases 01-04: all prior components this gallery showcases
- Existing: `crates/app/src/main.rs` (bootstrap template), `Cargo.toml` (workspace members)

## Overview

- Date: 2026-07-02
- Description: Build Navbar/Sidebar components (net-new — no existing nav container), then build `examples/ui_gallery` binary crate with sidebar-navigated showcase pages for every component from Phases 02-04, screenshot-verified end to end.
- Priority: P1 (delivers the actual "gallery" deliverable)
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- **Cross-cutting (see plan.md):** navbar/sidebar neutrals from `semantic` (dark+light); active-link accent from `palette`. **Gallery MUST include a light/dark toggle** (button in navbar) that flips theme appearance at runtime so both modes are demoable + screenshot-verifiable. Capstone visual-verify covers BOTH modes.
- No Navbar/Sidebar container component exists in `crates/ui/src/components/` (grep confirmed absent) — genuinely new, but simple: `div()`-based flex containers with nav-link styling, no complex state beyond "active route" tracking (owned by the gallery app, not the component itself).
- `examples/` directory already exists at repo root (empty) — just needs `examples/ui_gallery/` subdirectory with its own `Cargo.toml` + `src/main.rs`.
- Workspace wiring: add `"examples/ui_gallery"` to `Cargo.toml` `members` array. `default-members = ["crates/app"]` MUST stay unchanged — gallery only builds via explicit `cargo run -p ui_gallery`, never via `make dev`/`make check` (which target `crates/app` only per `Makefile`'s `PACKAGE := app`).
- Bootstrap: gallery's `main.rs` copies `crates/app/src/main.rs`'s exact pattern (`application().run(...)`, `theme::init(LoadThemes::JustBase, cx)`, `theme::set_theme_settings_provider(...)`, `cx.open_window`) — swap `HelloWorldApp` for a `GalleryApp` view with sidebar + content area.
- Gallery navigation model: `GalleryApp` holds `selected_page: GalleryPage` enum (Elements, Forms, DataDisplay, Overlays, Navigation — one per phase's component group); sidebar `Sidebar` component lists nav links, `on_click` updates `selected_page` + `cx.notify()`; content area renders the selected page's component showcase (reusing each component's `preview()` where possible, or a hand-built showcase layout).
- Visual verify for THIS phase = the capstone: screenshot the full gallery app (sidebar + content) via `VisualTestAppContext`, confirm layout matches a Tailwind UI "Application Shell" reference page (navbar/sidebar pattern), and spot-check a few showcase pages against their respective Tailwind UI component pages already verified in Phases 02-04.

## Requirements

### Functional

**Navbar** (new `crates/ui/src/components/navbar.rs`):
- bg surface, border-b gray-200, sticky top-0 (GPUI has no scroll-sticky in the CSS sense — desktop app likely doesn't need scroll-based navbar; treat as a fixed top bar, not a scrolling-page sticky), flex items-center px-6 py-4, shadow-sm.
- Builder: `.child(...)` for left content (logo/title), `.trailing(...)` for right-aligned actions.

**Sidebar** (new `crates/ui/src/components/sidebar.rs`):
- bg surface (light) — dark mode variant follows theme automatically via `cx.theme()`, width 256px, full height, border-r gray-200.
- Nav links: px-4 py-2, text-sm, rounded-md, hover:bg-gray-100; active: bg-gray-100, text-gray-900 (or theme-appropriate dark equivalents via `cx.theme().appearance()`).
- Builder: takes a `Vec<SidebarItem>` or accepts children directly (`ParentElement`), each item has label + optional icon + `is_active: bool` + `on_click`.

**Gallery app** (`examples/ui_gallery/`):
- `Cargo.toml`: `name = "ui_gallery"`, deps on `ui`, `theme`, `gpui`, `gpui_platform` (all `{ path = "../../crates/..." }`, matching workspace dependency versions).
- `src/main.rs`: bootstrap copy of `crates/app/src/main.rs` pattern.
- `src/gallery_app.rs` (or inline in `main.rs` if small enough <200 lines): `GalleryApp` struct with `selected_page: GalleryPage` state; renders `Sidebar` (left) + scrollable content area (right) showing selected page.
- `GalleryPage` enum variants, one per phase-02/03/04 component group: `Elements` (Button/Badge/Card/Alert), `Forms` (TextInput/Textarea/Select/Checkbox/Radio/Switch/Label), `DataDisplay` (Table), `Overlays` (Modal/DropdownMenu/Tooltip/Popover/Toast), `Navigation` (Tabs/Navbar/Sidebar self-demo).
- Each page renders using the relevant components' `preview()` functions (reuse, do not duplicate showcase code) composed into a scrollable `v_flex()` column with section headers.

### Non-functional

- Gallery crate itself stays modest — `main.rs` <150 lines, `gallery_app.rs` <200 lines, one file per page module if pages get complex (`pages/elements.rs`, `pages/forms.rs`, etc. — only split if a single file would exceed 200 lines).
- `cargo run -p ui_gallery` must open a real window showing the sidebar + first page by default.
- No changes to `crates/app` behavior — verify `make dev` and `make check` still work identically after this phase (they only ever touched `crates/app`, but workspace-level `Cargo.toml` edit must not regress them).

## Architecture

```
Cargo.toml                          (MODIFY — add "examples/ui_gallery" to members)
crates/ui/src/components/
├── navbar.rs                        (NEW)
└── sidebar.rs                       (NEW)
examples/ui_gallery/
├── Cargo.toml                       (NEW)
└── src/
    ├── main.rs                      (NEW — bootstrap, mirrors crates/app/src/main.rs)
    ├── gallery_app.rs                (NEW — GalleryApp view + GalleryPage enum)
    └── pages/                        (NEW, only if needed for file-size limits)
        ├── elements.rs
        ├── forms.rs
        ├── data_display.rs
        ├── overlays.rs
        └── navigation.rs
```

## Related Code Files

**Read first:**
- `crates/app/src/main.rs` (exact bootstrap template to mirror)
- `.claude/skills/gpui-ui-design/references/app-bootstrap.md`
- `.claude/skills/gpui-ui-design/references/views-and-state.md` (for `GalleryApp` state/render pattern)

**Modify:**
- `Cargo.toml` (workspace root) — add `examples/ui_gallery` to `members`; do NOT touch `default-members`.
- `crates/ui/src/prelude.rs` — export `Navbar`, `Sidebar`.

**Create:**
- `crates/ui/src/components/navbar.rs`
- `crates/ui/src/components/sidebar.rs`
- `examples/ui_gallery/Cargo.toml`
- `examples/ui_gallery/src/main.rs`
- `examples/ui_gallery/src/gallery_app.rs`
- `examples/ui_gallery/src/pages/*.rs` (as needed)

## Implementation Steps

1. Build `Navbar` component (simple flex container, no state).
2. Build `Sidebar` component (nav-link list, active-state styling, `on_click` callbacks).
3. Add `examples/ui_gallery` to workspace `members` in root `Cargo.toml`; confirm `default-members` untouched.
4. Scaffold `examples/ui_gallery/Cargo.toml` with correct path deps matching workspace dependency declarations.
5. Write `examples/ui_gallery/src/main.rs` mirroring `crates/app/src/main.rs` bootstrap exactly (theme init, settings provider, window open) but rendering `GalleryApp`.
6. Write `gallery_app.rs`: `GalleryPage` enum, `GalleryApp` struct (`selected_page` field), `Render` impl composing `Sidebar` (nav items = `GalleryPage` variants) + content area dispatching on `selected_page` to render the right showcase.
7. Wire each showcase page to call the relevant components' existing `fn preview()` (from Phases 02-04) inside a scrollable `v_flex()` with section headers — reuse, don't reinvent.
8. `cargo check -p ui_gallery` clean; `cargo run -p ui_gallery` opens a working window, sidebar navigation switches pages, `cx.notify()` triggers correctly.
9. Confirm `make dev` and `make check` (both scoped to `crates/app`) still work unchanged — run both, diff behavior against pre-phase baseline.
10. `make check-all` (workspace-wide) green, including the new `ui_gallery` crate.
11. Visual verify (capstone): screenshot full gallery app via `VisualTestAppContext` (sidebar + Elements page); compare layout proportions against a Tailwind UI "Application Shell" example page (Playwright screenshot); spot-check 2-3 other pages against their Phase 02-04 reference screenshots for consistency; iterate on any layout drift.

## Todo List

- [ ] Build Navbar component
- [ ] Build Sidebar component (nav links + active state)
- [ ] Add examples/ui_gallery to workspace members (keep default-members untouched)
- [ ] Scaffold ui_gallery Cargo.toml
- [ ] Write ui_gallery main.rs (bootstrap mirror)
- [ ] Write gallery_app.rs (GalleryPage enum + GalleryApp view)
- [ ] Wire showcase pages to existing preview() functions
- [ ] cargo run -p ui_gallery opens working window with navigation
- [ ] Confirm make dev / make check unchanged for crates/app
- [ ] make check-all green workspace-wide
- [ ] Visual verify: gallery screenshot vs Tailwind UI application shell reference

## Success Criteria

- `cargo run -p ui_gallery` opens a window with a working sidebar; clicking each nav item switches the content area to show that component group's showcase.
- Every component from Phases 01-04 appears somewhere in the gallery (audit: cross-check component list against showcase pages).
- `make dev` output/behavior identical to pre-phase baseline (still runs `crates/app` hello-world).
- `make check` and `make check-all` both green.
- `default-members = ["crates/app"]` unchanged in `Cargo.toml` (diff check).
- Visual verify screenshot comparison documented (gallery shell layout vs Tailwind UI reference).

## Risk Assessment

- **Risk:** Adding a new workspace member could slow down `cargo check --workspace` or introduce dependency resolution conflicts. **Mitigation:** path deps only, matching existing workspace dependency versions exactly (no new external crate versions introduced).
- **Risk:** Reusing `preview()` functions for showcase pages might produce a visually cluttered or inconsistent gallery layout (previews were designed for a component-doc tool, not necessarily a polished gallery). **Mitigation:** wrap each `preview()` call in a consistent section-header + padding layout at the gallery level; acceptable if previews look slightly utilitarian — polish is not the goal, coverage + Tailwind-accurate component styling (done in Phases 02-04) is.
- **Risk:** Sidebar active-state + dark/light theme interaction untested until this phase — first real integration test of Phase 01 tokens across light/dark.

## Security Considerations

None — internal dev-facing example app, not shipped to end users.

## Next Steps

- After this phase, the full UI kit + gallery deliverable is complete. Follow-up (out of scope for this plan): promote select components' visual QA learnings into a reusable `agent-learned` skill via `/vk:learn` if patterns recur (e.g. "how to approximate Tailwind shadows in GPUI").
- If TextInput/Textarea got split into a `phase-03b` (per Phase 03's escalation note), the Forms showcase page in this phase depends on that sub-phase being done first.
