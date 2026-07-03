# Phase 09 — Gallery Consolidation + Final Verify

## Context Links

- All Phase 01-08 files (this phase wires every deliverable from all prior phases into the gallery)
- Research: `researcher-02-codebase-audit.md` (current gallery state: 4 pages, `.preview()` pattern)
- Plan: `./plan.md` (Cross-Cutting Requirements, verify commands)

## Overview

- Date: 2026-07-03
- Description: Extend `examples/ui_gallery`'s `GalleryPage` enum + pages to showcase every new/restyled component from Phases 2-8 (whichever ran); run the final full-workspace verify (`make check`, `make check-all`, `cargo fmt --all --check`) confirming no regression across the whole 48-component catalog effort.
- Priority: P1 (this is the acceptance gate for the whole plan)
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- Current `examples/ui_gallery/src/gallery_app.rs` (184 lines) has 4 pages: `Elements, Forms, Feedback, Navigation`. This phase EXTENDS the `GalleryPage` enum (add `Data, Overlays, Layout`, keep existing 4) rather than rewriting the gallery from scratch.
- Pattern to follow (already established): each page calls the relevant components' `.preview()` (or hand-composes a small showcase using their builders) inside a scrollable `v_flex()` with section headers — reuse, do not duplicate showcase code that already lives in each component's `preview()`.
- If `gallery_app.rs` would exceed ~200 lines after adding 3 more pages, split into `examples/ui_gallery/src/pages/{elements,forms,feedback,navigation,data,overlays,layout}.rs` (one module per page) — this was anticipated as an option in the prior plan's Phase 5.
- This is the ONLY phase touching `examples/ui_gallery/*` — no file-ownership conflict with Phases 2-8 (they only touch `crates/ui`/`crates/icons`).
- Depends on Phases 2-7 being merged (Phase 8 optional — gallery gracefully has fewer pages if Phase 8 didn't run, note this, don't block on it).

## Requirements

### Functional

- Extend `GalleryPage` enum: add `Data` (Phase 4: DataTable/List/DescriptionList/StatsCard/MediaObject/EmptyState/Feed), `Overlays` (Phase 5: Modal/AlertModal/Drawer/Dropdown-family/Popover/Tooltip/Toast/ToastStack), `Layout` (Phase 7: AppShell/PageHeading/SectionHeading/Container/Card). Extend existing `Elements` page with Phase 2's ButtonGroup/Facepile/Chip/Divider; extend `Forms` page with Phase 3's InputGroup/SearchInput/Combobox/MultiSelect/SegmentedControl/FormField/ActionPanel/FileInput; extend `Navigation` page with Phase 6's Breadcrumb/Pagination/VerticalNav/Stepper (Tabs/Progress restyle shows via existing `preview()`). If Phase 8 ran, add an `Advanced` page for its 7 deliverables.
- Every showcase entry must actually render (no panics) and, where interactive (Checkbox, Switch, Combobox, MultiSelect, Modal, Dropdown, Toast, Tabs, Pagination), must be genuinely clickable in the running app — this is the manual acceptance step, not just a compile check.
- Update sidebar page list (in `gallery_app.rs`) to include new page names.

### Non-functional

- Split into `pages/*.rs` modules if single-file size limit is hit (see Key Insights).
- No changes to `crates/app`, `default-members`, or any `crates/ui`/`crates/icons` file (this phase is gallery-only wiring).

## Architecture

```
examples/ui_gallery/src/
├── gallery_app.rs        (MODIFY — extend GalleryPage enum + page dispatch)
└── pages/                  (NEW, only if file-size forces split)
    ├── data.rs
    ├── overlays.rs
    ├── layout.rs
    └── advanced.rs          (only if Phase 8 ran)
```

## Related Code Files

**Modify:** `examples/ui_gallery/src/gallery_app.rs`.

**Create (conditional):** `examples/ui_gallery/src/pages/*.rs`.

**Read (no modify, source of `preview()` calls):** every component file touched in Phases 2-8.

## Implementation Steps

1. Confirm which of Phases 2-8 actually landed (git log / branch merge status) — build the page list accordingly (Phase 8's `Advanced` page is conditional).
2. Extend `GalleryPage` enum + `PAGES` const + sidebar labels.
3. Extend `Elements`/`Forms`/`Navigation` page render functions with the new components from Phases 2/3/6.
4. Add `Data` page (Phase 4 deliverables).
5. Add `Overlays` page (Phase 5 deliverables) — needs interactive triggers (buttons that open Modal/Dropdown/Toast) since these are overlay components, not static renders.
6. Add `Layout` page (Phase 7 deliverables) — `AppShell` demo may need its own sub-view or a scaled-down preview (full shell inside a gallery page is a nested-shell situation, handle via a bounded-height container).
7. If Phase 8 ran, add `Advanced` page.
8. Split into `pages/*.rs` if `gallery_app.rs` exceeds ~200 lines.
9. `cargo check -p ui_gallery` clean.
10. `cargo run -p ui_gallery` — manually click through EVERY page, EVERY interactive component (Checkbox, Switch, Combobox, MultiSelect open/select, Modal open/close, Dropdown open/select, Toast trigger+auto-dismiss, Tabs switch, Pagination bounds) — this is the real acceptance test.
11. Toggle light/dark (existing navbar toggle) and spot-check every new page renders legibly in both modes (no invisible/washed-out elements) — no screenshot required, just visual eyeballing while running.
12. Run full-workspace verify: `make check`, `make check-all` (if present), `cargo fmt --all --check` — all green.
13. Diff-check `crates/app/`, `Cargo.toml` `default-members` unchanged (`git diff Cargo.toml` shows only `members` list line untouched from this plan's start — should already be present from before this plan).

## Todo List

- [ ] Confirm which Phases 2-8 landed
- [ ] Extend GalleryPage enum + sidebar
- [ ] Extend Elements/Forms/Navigation pages
- [ ] Add Data page
- [ ] Add Overlays page (interactive triggers)
- [ ] Add Layout page (AppShell nested-demo handling)
- [ ] Add Advanced page (if Phase 8 ran)
- [ ] Split into pages/*.rs if size limit hit
- [ ] cargo check -p ui_gallery clean
- [ ] Manual click-through of every page + every interactive component
- [ ] Light/dark toggle spot-check on every new page
- [ ] make check + make check-all + cargo fmt --all --check green
- [ ] Confirm crates/app + default-members untouched

## Success Criteria

- `cargo run -p ui_gallery` opens; every one of the 48 component deliverables (fewer if Phase 8 skipped) is reachable and renders without panic.
- Every interactive component (list above) demonstrably works via real clicks during manual testing, not just static render.
- Both light and dark modes legible on every page (manual check).
- `make check`, `make check-all`, `cargo fmt --all --check` all green at the FULL workspace level (final acceptance gate for the entire plan).
- `crates/app` and `default-members = ["crates/app"]` unchanged (git diff confirms).
- Cross-check: every category in Phase 01's Locked Scope Matrix is now either showcased in the gallery (IN-SCOPE, done) or explicitly marked OUT/deferred (Phase 8) — no silent gaps.

## Risk Assessment

- **Risk:** `AppShell` (Phase 7) nested inside a gallery page could create a confusing "shell within a shell" visual — the gallery itself is already a shell-like layout. **Mitigation:** bound `AppShell`'s demo to a fixed small height/width container within the Layout page, clearly labeled as a demo, not full-window.
- **Risk:** Manual click-through of ~48 components is time-consuming and easy to shortcut. **Mitigation:** this IS the explicit, user-approved acceptance method (no screenshot verify per plan.md) — do not skip it; treat the Todo checklist as mandatory, not optional.
- **Risk:** If Phase 8 was cancelled mid-plan, stale references to its components in this phase's page list would break the build. **Mitigation:** step 1 explicitly confirms landed phases before wiring pages.

## Security Considerations

None — internal dev-facing example app, not shipped.

## Next Steps

- This phase is the final deliverable acceptance gate. After it's green, the plan is complete (Phase 8 status permitting per user's go/no-go).
- Follow-up (out of scope): consider `/vk:learn` to distill recurring restyle patterns (e.g. "composing DropdownMenu into new trigger components") into a reusable `agent-learned` skill if this pattern recurs in future UI work.
