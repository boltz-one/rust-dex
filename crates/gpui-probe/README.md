# gpui-probe

Shared element-tree core for a GPUI inspector overlay + an in-process UI test
driver, built on `boltz-gpui` only (no theme/ui/icons dependency).

- **`.probe("id")`** on any `IntoElement` records its real per-frame bounds into
  an app-wide `ElementRegistry` (via `gpui::canvas()`), with no `cfg`/feature
  gate — works in every build mode.
- **Test driver** (`test-support` feature): `TestHarness::new(build)` opens a
  headless TEST-platform window; `find(id).click()/.type_text()/.assert_visible()
  /.assert_not_present()` with Playwright-style actionability waits.
- **Tree-text snapshots** (`tree_text` + `insta` in your dev-deps).
- **Inspector overlay** (`inspector` feature): read-only hover-highlight + element
  list panel over a running app.

## Features

- `test-support` — the in-process test driver. Add as a **dev-dependency** with
  this feature; release builds (default features) never compile it, so no
  consumer links `gpui`'s `test-support`.
- `inspector` — the overlay/panel UI.
- `semantic` — reserved for `Role`/`Label` selectors (Phase 06 upstream work).

## Limitations (MVP)

- **Staleness needs a frame driver.** `get`/`all_visible` prune unmounted
  elements only when something calls `ElementRegistry::begin_frame()` each frame
  — the `TestHarness` does. A passive consumer (e.g. inspector-only) sees
  last-known bounds; unmounted elements are not auto-pruned. Keep probe ids a
  bounded, static set.
- **Bounds = wrapper box.** Tight in flex parents; may over-report width in block
  parents. Not a compositor hit-test.
- **Layout participation.** Probing an element wraps it; `flex_grow`/`flex_basis`
  set on the probed element itself stop influencing the parent. Probe leaf /
  intrinsically-sized elements.
- **Single App-wide registry**, not per-window: keep ids unique across windows.

Real occlusion, semantic selectors, and per-window scoping are Phase 06 work.

License: Apache-2.0
