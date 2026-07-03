# Phase 4 — Full-Page Composed Examples

## Context links

- `./plan.md` Key Codebase Facts
- `./phase-02-fix-interactive-bugs.md`, `./phase-03-enrich-showcase-variants.md`
- `plans/20260703-0001-tailwind-app-ui-complete/phase-07-layout-shells.md` (AppShell component
  details — reused here, not rebuilt)

## Overview

Current gallery pages show components in isolation (one `section()` per component). This phase
adds a small number of realistic, composed page examples — a dashboard, a settings form, a
table-with-toolbar view, and an app-shell-wrapped screen — so components are exercised together
(layout interplay, real data flow, nested state) the way Tailwind Plus's "Example Applications"
category demonstrates.

## Key Insights

- `AppShell` component already exists (`crates/ui/src/components/app_shell.rs` per prior plan's
  Phase 7) — reuse it as the composed examples' outer frame, don't rebuild shell chrome.
- These are NEW pages/sections, additive to the existing 7-page sidebar — either as a new 8th
  `GalleryPage::Examples` entry, or as extra sections appended to `Layout` page. Prefer a new
  `GalleryPage::Examples` variant (mirrors the existing `PAGES` array pattern in
  `gallery_app.rs:33-41`) since composed examples are conceptually distinct from single-component
  showcases.
- Composed examples need their OWN local state (e.g. dashboard filter, settings form fields,
  table sort/page) — follow the same "persist Entity/state on GalleryApp, never inside render"
  rule from Phase 2, or scope the state to a small dedicated sub-struct field on `GalleryApp` if
  it's example-specific (e.g. `examples_dashboard_range: DateRange`-style plain field).

## Requirements

1. **Dashboard example**: StatsCard row + a Table/List + an Alert — composed with `AppShell` or
   plain `v_flex`, using Phase 3's enriched Table/StatsCard variants.
2. **Settings form example**: FormField/InputGroup/Switch/Select grouped into realistic sections
   (Account/Notifications) with an ActionPanel (Save/Cancel) at the bottom — real state, Save
   button visibly does something (e.g. shows a toast via existing `ToastItem`/toast-stack state
   already on `GalleryApp`).
3. **Table + toolbar example**: Table with a SearchInput/Combobox filter row above it and
   Pagination below — demonstrates the fixed interactive components from Phase 2 working
   together, with the Table content actually filtering by the SearchInput query (real behavior,
   not decorative).
4. **App shell example**: full `AppShell::preview()`-based screen showing Navbar + Sidebar +
   content together (this may already substantially exist per Layout page — extend, don't
   duplicate, if `AppShell::preview()` already composes these).

## Architecture

Low-risk, no ADR — new page/section composition using only existing components; one line: adds
a `GalleryPage::Examples` variant (additive enum member) plus a `pages/examples.rs` file
following the existing `pages/*.rs` module convention.

## Related code files

- `examples/ui_gallery/src/gallery_app.rs` (add `GalleryPage::Examples` variant + `PAGES` entry
  + dispatch arm + any new state fields for the composed examples)
- `examples/ui_gallery/src/pages/examples.rs` (new file, following existing page module shape)
- `examples/ui_gallery/src/pages/mod.rs` (add `pub mod examples;`)
- `crates/ui/src/components/app_shell.rs`, `data_table.rs`/`table_row.rs`, `search_input.rs`,
  `combobox.rs` (reused, read-only unless a real gap is found — if so, fix non-breakingly)

## Implementation Steps

1. Add `GalleryPage::Examples` to the enum + `PAGES` array + `title()` match arm in
   `gallery_app.rs`, and a dispatch arm routing to `self.render_examples(window, cx)` (method,
   since it needs `&mut self` for real interactive state).
2. Create `pages/examples.rs`, implement `render_examples` as a `GalleryApp` impl block
   (mirroring `forms.rs`/`overlays.rs` module shape: `use crate::gallery_app::GalleryApp; impl
   GalleryApp { pub(crate) fn render_examples(...) }`).
3. Build the Dashboard section first (lowest state complexity — mostly Phase 3's enriched
   static variants composed together).
4. Build the Settings form section — wire Save button to push a toast via the existing
   `GalleryApp::toasts`/`next_toast_id` fields (already present, see `gallery_app.rs:61-62`).
5. Build the Table + toolbar section — wire the SearchInput's `query(cx)` to filter the Table's
   rows in `render_examples` (real filter logic, small and local, not a new component).
6. Build/confirm the App shell section reusing `AppShell::preview()` or a real composition.
7. Add a harness test: open gallery, navigate to Examples page (simulate sidebar click), assert
   it renders (screenshot or entity read), then type into the toolbar SearchInput and assert
   the Table's visible row count changed.

## Todo

- [ ] `GalleryPage::Examples` variant + routing wired
- [ ] `pages/examples.rs` created, `mod.rs` updated
- [ ] Dashboard section built
- [ ] Settings form section built + Save→toast wired
- [ ] Table+toolbar section built + real filter wired
- [ ] App shell section built/confirmed
- [ ] Harness test: navigate to Examples + filter table, assert row count change
- [ ] `make check` + `cargo fmt --all --check` green

## Success Criteria

- New "Examples" sidebar entry renders 4 composed sections without panic.
- Table+toolbar filter is REAL (typing narrows visible rows), not decorative.
- Settings form Save button produces a visible toast via existing toast-stack state.
- Harness test for the Examples page passes.

## Risk Assessment

- Composed pages risk becoming a second source of truth for component usage patterns that
  drifts from per-component `preview()`s — mitigate by reusing `preview()`-level building
  blocks (e.g. call the same Table/StatsCard construction helpers) rather than hand-rolling
  parallel markup.
- Adding a `GalleryPage` enum variant is additive but touches `gallery_app.rs`'s match
  statements in multiple places — grep for `match current` / `match self.page` before editing
  to catch every exhaustive match site.

## Security Considerations

N/A — local demo state and static/filtered display data only, no network/file I/O added.

## Next steps

Phase 5 closes any remaining Tailwind catalog gaps found while building these composed
examples (e.g. if Table sort-by-column UI turns out to be missing, that's a Phase 5 item).
