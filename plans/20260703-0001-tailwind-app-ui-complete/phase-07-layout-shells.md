# Phase 07 — Layout/Shells & Headings (App Shell, Page/Section Heading, Container, Card Polish, Feed)

## Context Links

- Research: `researcher-01-tailwind-appui-catalog.md` (Application Shells row: Stacked/Sidebar/Multi-column layouts; Headings row: Page/Card/Section Heading; Layout row: Containers, Cards, List Containers, Media Objects, Dividers)
- Research: `researcher-02-codebase-audit.md` (Navbar/Sidebar ✅ done; Card ✅ done)
- Plan: `./plan.md` (Cross-Cutting Requirements)

## Overview

- Date: 2026-07-03
- Description: Compose an Application Shell from the already-done Navbar+Sidebar; build Page Heading, Section Heading, Container; polish Card (minor gap-fill only); note Feed here if not already covered by Phase 4 (Feed is Phase 4's deliverable — this phase does NOT duplicate it, see Key Insights).
- Priority: P2
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- `Navbar` and `Sidebar` are ✅ DONE (prior plan) — this phase does NOT restyle them, only COMPOSES a new `app_shell.rs` wrapper that lays them out together with a content slot (header + sidebar + main, per researcher-01's "Stacked Layout"/"Sidebar + Content" rows). Zero edits to `navbar.rs`/`sidebar.rs` themselves — avoids file-ownership overlap with anything else.
- "Feeds" is Phase 4's deliverable (`components/feed.rs`, Data Display & Lists group per researcher-01's own table placement) — do NOT recreate it here; if Phase 4 hasn't landed yet when this phase starts, just skip referencing it in this phase's own file list (no dependency, they're independent files).
- `Card` is ✅ done (prior plan: default/elevated/outlined) — "Card polish" here means closing small gaps only (e.g. hover-state shadow bump if trivial, header/footer padding consistency check against Tailwind spec) — NOT a rebuild.
- "Multi-column Grid" from researcher-01 is a caller-composed layout (flex/grid wrap of existing components) — document as covered, no new component, same treatment as Phase 4's "Grid List" finding.

## Requirements

### Reuse Map

| Tailwind category | GPUI base | Action |
|---|---|---|
| Application Shells (stacked/sidebar+content) | `components/navbar.rs` + `components/sidebar.rs` (both done) | NEW `components/app_shell.rs` (compose only) |
| Multi-column Grid | caller-composed | NONE — document as covered |
| Page Heading | none | NEW `components/page_heading.rs` |
| Section Heading | none | NEW `components/section_heading.rs` |
| Card Heading | `components/card.rs` (header slot) | covered by Card's existing header builder — verify, extend only if gap found |
| Containers | none | NEW `components/container.rs` |
| Cards | `components/card.rs` | POLISH (minor gaps only) |
| List Containers | `components/list.rs` (Phase 4) | covered by Phase 4, no duplicate work here |
| Media Objects | `components/media_object.rs` (Phase 4) | covered by Phase 4, no duplicate work here |
| Dividers | `components/divider.rs` (Phase 2) | covered by Phase 2, no duplicate work here |

### Functional

- **App Shell** (new): composes `Navbar` (top) + `Sidebar` (left) + a `.content(impl IntoElement)` main area, using `h_flex()`/`v_flex()` layout only — no new styling logic, pure composition of two already-restyled components.
- **Page Heading** (new): title (largest text size, e.g. `text_ui_3xl` equivalent per `typography.rs`) + optional subtitle (`text_muted`) + right-aligned actions slot (`Button`s) — mb-6, flex justify-between items-center.
- **Section Heading** (new): mid-size title (`text_ui_2xl`-ish) + optional grouped content below — mb-4.
- **Container** (new): fixed max-width centering wrapper (`max_w()` + `mx_auto()` if GPUI supports auto-margin centering — check `spacing.rs`/`styled.rs` first; if no auto-margin primitive exists, compute centering via fixed width + parent flex `justify_center()`) + `px()` horizontal padding.
- **Card polish**: read current `card.rs`, diff its header/body/footer padding against Tailwind's p-6 spec; fix only genuine gaps found (do not restructure working code).

### Non-functional

- `app_shell.rs`, `page_heading.rs`, `section_heading.rs`, `container.rs` each under 100 lines (thin composition components).

## Architecture

```
crates/ui/src/components/
├── app_shell.rs         (NEW — composes existing Navbar + Sidebar)
├── page_heading.rs        (NEW)
├── section_heading.rs      (NEW)
├── container.rs             (NEW)
└── card.rs                   (MODIFY — polish only, if gaps found)
```

## Related Code Files

**Read first:** `navbar.rs`, `sidebar.rs` (public builder API, to compose without modifying), `card.rs` (current header/body/footer spec).

**Modify:** `card.rs` (only if gap found), `crates/ui/src/components.rs`, `crates/ui/src/prelude.rs`.

**Create:** `app_shell.rs`, `page_heading.rs`, `section_heading.rs`, `container.rs`.

## Implementation Steps

1. Read `navbar.rs` + `sidebar.rs` public APIs (builder methods, no modification).
2. Build `AppShell` (top navbar + left sidebar + content slot, pure composition).
3. Build `PageHeading` (title+subtitle+actions).
4. Build `SectionHeading` (title+content).
5. Build `Container` (max-width centering).
6. Read `card.rs`; diff against Tailwind p-6/header/footer spec; fix only genuine gaps.
7. Update/add `preview()` for all 5 deliverables.
8. `cargo check -p ui` clean.
9. `cargo run -p ui_gallery` — confirm `AppShell` renders Navbar+Sidebar+content correctly (no double-borders or layout overlap).

## Todo List

- [ ] Read navbar.rs + sidebar.rs public APIs
- [ ] Build AppShell (compose only)
- [ ] Build PageHeading
- [ ] Build SectionHeading
- [ ] Build Container
- [ ] Card polish (diff + fix genuine gaps only)
- [ ] preview() for all 5
- [ ] cargo check -p ui clean
- [ ] Manual layout check: AppShell renders without overlap/double-border

## Success Criteria

- `make check` + `make check-all` + `cargo fmt --all --check` green.
- `AppShell` renders Navbar+Sidebar+content with no layout overlap, using ONLY the existing public APIs of `Navbar`/`Sidebar` (zero edits to those two files — verify via `git diff --stat`).
- Container correctly centers content at a fixed max-width (manual check in gallery).
- Card polish changes (if any) documented with a one-line "before → after" note; if no gap found, note "no changes needed" explicitly (do not silently skip without confirming).

## Risk Assessment

- **Risk:** `AppShell` composition might reveal a layout API gap in `Navbar`/`Sidebar` (e.g. no way to set sidebar width consistently) that tempts editing those files, breaking the "zero edits" constraint. **Mitigation:** if a genuine gap is found, solve it via `AppShell`'s own wrapper styling (e.g. wrap `Sidebar` in a fixed-width `div()`), not by modifying `Sidebar` itself, unless the gap is a real bug — then flag it as a note in Next Steps rather than silently fixing inline.
- **Risk:** GPUI may lack a true CSS `margin: auto`-style centering primitive for `Container`. **Mitigation:** step 5 explicitly checks `spacing.rs`/`styled.rs` first; fallback to flex `justify_center()` wrapper, documented as the chosen approach.

## Security Considerations

None — presentational layout components.

## Next Steps

- Phase 9 gallery's own shell could optionally adopt `AppShell` for a more realistic showcase, but this is optional polish, not required (gallery's existing `gallery_app.rs` layout already works).
- Confirm Phase 4's `feed.rs` and Phase 2's `divider.rs` are not duplicated by this phase (checked in Reuse Map above).
