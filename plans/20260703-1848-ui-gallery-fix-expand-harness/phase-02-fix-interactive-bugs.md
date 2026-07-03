# Phase 2 — Fix Interactive Bugs

## Context links

- `plans/20260703-1848-ui-gallery-fix-expand-harness/research/researcher-01-bug-rootcause.md`
  (bug matrix, file:line, fix direction for every bug below)
- `./plan.md` Key Codebase Facts
- `./phase-01-investigation-harness-foundation.md` (harness must exist first)

## Overview

Fix all 6 confirmed bugs. Each fix ships with a harness test (Phase 1's scaffold) that fails
before the fix and passes after — this is the "computer-use" proof the plan requires, not a
manual screenshot check.

## Key Insights

- Bug #4 (TextInput focus) is the only **component-level** bug — everything else is
  gallery-wiring (fix in `examples/ui_gallery/src/**`, not `crates/ui`).
- Bugs #3/#4/#5 in the research doc (MultiSelect/Combobox/SearchInput) are the SAME root cause
  (`cx.new(...)` inside a free `*_preview` fn, recreated every `GalleryApp::render()`) — same
  fix pattern (persist `Entity<T>` on `GalleryApp`), 3 separate call sites.
- Reference pattern already correct in the codebase: `gallery_app.rs:57-59` — `text_input`,
  `textarea`, `select` fields, created once in `GalleryApp::new`, cloned into render. Copy this
  exactly for the 3 new fields.
- SegmentedControl is `RenderOnce`, not an `Entity` — its "state" is just a plain `usize` field
  on `GalleryApp` (like `modal_open: bool` already is), not a new `Entity`.
- Scroll fix is a container-level wrap, not a per-page rewrite — touches
  `gallery_app.rs:141`'s content wrapper (or make it per-page if content height varies wildly
  page to page — decide during implementation, default to the single shared wrapper first).

## Requirements — the 6 bugs

1. **[component-bug] TextInput never focuses on click**
   `crates/ui/src/components/text_input.rs` render fn (~line 111-132): add
   `.on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| { window.focus(&this.focus_handle); cx.notify(); }))`
   to the `field` div. Non-breaking (additive handler, no signature change). Fixes typing for
   every `TextInput` consumer including the inner inputs of `Combobox`/`SearchInput`.
2. **[gallery-wiring] Scroll doesn't work on any page** (RESOLVED approach)
   Store a `ScrollHandle` field on `GalleryApp` (`scroll: ScrollHandle`, init `ScrollHandle::new()`
   in `new()`) so scroll offset persists across frames. `gallery_app.rs:141` — change
   `v_flex().flex_1().p_6().gap_8().child(content)` to
   `.id("gallery-content").overflow_y_scroll().track_scroll(&self.scroll)` (both verified on
   `div`: `div.rs:1198` / `:1204`). This makes the harness scroll test deterministic: dispatch a
   `ScrollWheelEvent { position, delta: ScrollDelta::Pixels(point(px(0.), px(-200.))), modifiers,
   touch_phase }` (shape at `crates/gpui/src/interactive.rs:428`), then assert `self.scroll`'s
   offset moved — no pixel/screenshot diff needed. Add a visible `Scrollbar` thumb only if the
   showcase needs it (optional polish, not required for the fix).
3. **[gallery-wiring] SegmentedControl (reported as "Tabs") doesn't change on click**
   Add a `forms_segment: usize` field to `GalleryApp` (default `0`), initialize in `new()`.
   In `forms.rs`, replace the static `SegmentedControl::preview(window, cx)` call with a
   hand-built `SegmentedControl::new("segmented-demo", [...]).active(self.forms_segment)
   .on_change(cx.listener(|this, i, _, cx| { this.forms_segment = *i; cx.notify(); }))` (adjust
   exact closure signature to match `on_change`'s `Fn(usize, &mut Window, &mut App)` — this is
   a `RenderOnce` builder consumed in a `&self`/`&mut self` method, so use `cx.listener` only if
   `render_forms` takes `&mut self`; check current `&self` signature and change to `&mut self`
   if needed, since it must call `cx.notify()`).
4. **[gallery-wiring] MultiSelect selection resets**
   Add `multi_select: Entity<MultiSelect>` field to `GalleryApp`, init in `new()` with
   `cx.new(|_| MultiSelect::new([...]).selected_indices([0, 2]))`. In `forms.rs`, replace
   `multi_select_preview(window, cx)` call with `self.multi_select.clone().into_any_element()`.
5. **[gallery-wiring] Combobox typed filter / selection resets**
   Add `combobox: Entity<Combobox>` field, init with `cx.new(|cx| Combobox::new(cx, [...]))`.
   Replace `combobox_preview(window, cx)` with `self.combobox.clone().into_any_element()`.
6. **[gallery-wiring] SearchInput query resets**
   Add `search_input: Entity<SearchInput>` field, init with
   `cx.new(|cx| SearchInput::new(cx, "Search…"))`. Replace `search_input_preview(window, cx)`
   with `self.search_input.clone().into_any_element()`.

## Architecture

Low-risk, no ADR — all fixes are additive (new struct fields + one new handler), no public API
signature changes in `crates/ui`. One line: components stay non-breaking because every change
either adds an optional-effect handler or moves an already-public constructor call from a free
function into the existing `GalleryApp::new()`.

## Related code files

- `crates/ui/src/components/text_input.rs` (bug #1 fix)
- `examples/ui_gallery/src/gallery_app.rs` (struct fields, `new()`, content wrapper — bugs #2-6)
- `examples/ui_gallery/src/pages/forms.rs` (call-site swaps — bugs #3, #4, #5, #6)
- `crates/ui/src/components/scrollbar.rs` (reference pattern for bug #2)
- `crates/ui/src/components/multi_select.rs`, `combobox.rs`, `search_input.rs`,
  `segmented_control.rs` (constructors used, no changes needed to these files themselves)
- `examples/ui_gallery/tests/visual_harness.rs` (add 6 test fns, one per bug)

## Implementation Steps

1. Fix bug #1 (TextInput focus) in `text_input.rs`. Write/run a harness test: open gallery,
   `simulate_click` on the email input's known position (or use `read_window` to locate via
   layout — simplest: click then `simulate_input("hello")`, assert
   `text_input.read(cx).text() == "hello"`).
2. Fix bug #2 (scroll) in `gallery_app.rs`. Harness test: `simulate_event` with a
   `ScrollWheelEvent` (shape from Phase 1's notes) on the content area, assert scroll offset
   changed (read via whatever state `.overflow_y_scroll()` + `id()` exposes — check
   `scrollbar.rs` for how offset is read back, e.g. via `ScrollHandle`).
3. Fix bug #3 (SegmentedControl) — add `forms_segment` field + wire `.on_change`. Harness test:
   click the second segment, assert `forms_segment == 1`.
4. Fix bug #4 (MultiSelect) — add `multi_select` Entity field, swap call site. Harness test:
   click to open, click an option, trigger an unrelated re-render (e.g. click sidebar page and
   back, or toggle theme), assert selection persisted.
5. Fix bug #5 (Combobox) — same pattern. Harness test: type a filter substring, assert filtered
   list narrows; select an option, assert `combobox.read(cx).value()` persists after re-render.
6. Fix bug #6 (SearchInput) — same pattern. Harness test: type a query, assert
   `search_input.read(cx).query(cx)` persists after re-render.
7. Run full `examples/ui_gallery/tests/visual_harness.rs` suite (`--ignored`), all 6 new tests
   green plus Phase 1's smoke test.

## Todo

- [ ] Bug #1 TextInput focus fix + harness test
- [ ] Bug #2 scroll fix + harness test
- [ ] Bug #3 SegmentedControl wiring + harness test
- [ ] Bug #4 MultiSelect entity persistence + harness test
- [ ] Bug #5 Combobox entity persistence + harness test
- [ ] Bug #6 SearchInput entity persistence + harness test
- [ ] `make check` + `cargo fmt --all --check` green
- [ ] `cargo run -p ui_gallery` manual smoke (type, click segment, toggle multiselect, scroll)

## Success Criteria

- All 6 harness tests pass (`cargo test -p ui_gallery -- --ignored`), each provably failing
  before its corresponding fix (verify by running the test against pre-fix code once).
- No `crates/ui` public signature changed; `cargo check --workspace` unaffected outside
  `text_input.rs`'s additive handler.
- Manual `cargo run -p ui_gallery`: typing works in every text field, SegmentedControl click
  changes active segment, MultiSelect/Combobox/SearchInput retain state across page switches,
  every page scrolls when content overflows.

## Risk Assessment

- `.overflow_y_scroll()` added to the single shared content wrapper could clip a page whose
  content is meant to be taller than viewport intentionally (unlikely here, but check Data/
  Layout pages after the change — those have the most content).
- `on_mouse_down` added to `TextInput` must not double-fire with the existing `on_key_down` or
  break `Combobox`/`SearchInput`'s embedded-input hit-testing (embedded inputs are `div()`s
  inside a bigger clickable trigger row — verify the mouse-down doesn't get swallowed by the
  trigger's own `on_click` in `combobox.rs`/`search_input.rs`).

## Security Considerations

N/A — UI state wiring only, no new input parsing beyond existing `TextInput` key handling.

## Next steps

Phase 3 enriches each of these now-correctly-stateful showcases with more variants; the
underlying state model must be correct first (this phase) or the added variants would just
multiply the same reset bug.
