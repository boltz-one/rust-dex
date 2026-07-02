# Phase 03 — Form Controls

## Context Links

- Research: `plans/20260702-1417-tailwind-ui-gallery-and-uikit/research/researcher-01-tailwind-spec.md` (§2.2)
- Phase 01: `./phase-01-design-tokens.md` (focus_ring, palette)
- Phase 02: `./phase-02-core-components.md` (badge color-variant pattern for error states)
- Existing: `crates/ui/src/components/toggle.rs` (`Checkbox`, `Switch` already implemented)

## Overview

- Date: 2026-07-02
- Description: Restyle existing `Checkbox`/`Switch`; build net-new `TextInput`, `Textarea`, `Select`, `RadioButton`; standardize `Label` + help/error text styling for form fields.
- Priority: P1
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- **Cross-cutting (see plan.md):** input borders/bg/text/placeholder neutrals come from `semantic` (dark+light), accents (checked blue-600, error red-500/600) from `palette`. Focus + error use the gapped `focus_ring()`/`focus_ring_error()` wrappers from Phase 01. Checkbox/select icons use Heroicon `IconName`s (`check`, `chevron-down`). Visual-verify in BOTH modes.
- `Checkbox` and `Switch` already exist in `toggle.rs` (lines 43 and 338) — this is a RESTYLE task for these two, not new components. `SwitchColor`, `SwitchLabelPosition`, `SwitchField` also exist — check if `SwitchField` already bundles label+switch (Tailwind's toggle pattern includes an inline label).
- `TextInput`, `Textarea`, `Select`, `RadioButton` do NOT exist anywhere in `crates/ui/src/components/` (confirmed via grep) — no ready-made TextInput/Editor component in `ui` OR `gpui` (Zed's actual `editor` crate isn't vendored here). BUT the hard 20% (platform IME/key plumbing) IS already vendored and working:
  - `crates/gpui/src/input.rs` — `EntityInputHandler` trait (`text_for_range`, `selected_text_range`, `marked_text_range`, `replace_text_in_range`, `replace_and_mark_text_in_range` for IME composition, `bounds_for_range`, `character_index_for_point`, `accepts_text_input`) + `ElementInputHandler<V>` adapter (lines 82-195).
  - `crates/gpui/src/platform.rs:1275-1373` — platform `InputHandler` trait, a 1:1 port of NSTextInputClient; `crates/gpui/src/window.rs:4058-4078` — `Window::use_input_handler` registration; key dispatch at `window.rs:4511-4566` consults `accepts_text_input`.
  - macOS (`gpui_macos/src/window.rs:236,1693`) and Linux Wayland (`gpui_linux/.../wayland/client.rs`, `zwp_text_input_v3`, preedit/composing state at `window.rs:442,911-928`) both implement this already. X11/Windows have stubs wired.
  - `crates/gpui/src/key_dispatch.rs:845,1045` has example `InputHandler` impls usable as a template.
  - **Verdict: medium effort, not from-scratch.** Missing piece is application-level only: a struct holding `String` buffer + `Range<usize>` selection, caret render/blink, click-to-position + drag-to-select mouse handling, and implementing the ~8 `EntityInputHandler` methods against that buffer. A single-line subset (no wrapping/multi-cursor) is a well-scoped few-hundred-line component.
- GPUI has no focus-ring auto-behavior (confirmed research). Every focusable form control needs explicit `.track_focus(&focus_handle)` + Phase 01's `focus_ring()` helper.
- Radio buttons need group behavior (only one selected) — needs either a shared `Entity<T>` state pattern (per gpui-ui-design skill's views-and-state reference) or a simple parent-owned `selected_value` field with each `RadioButton` taking `selected: bool` + `on_click`.

## Requirements

### Functional

**Text Input** (new `crates/ui/src/components/text_input.rs`):
- Build a `TextInputState` (or similar) struct: `content: String`, `selected_range: Range<usize>`, `focus_handle: FocusHandle`, implementing `EntityInputHandler` (`crates/gpui/src/input.rs`) — the 8 methods (`text_for_range`, `selected_text_range`, `marked_text_range`, `unmark_text`, `replace_text_in_range`, `replace_and_mark_text_in_range`, `bounds_for_range`, `character_index_for_point`). Use `crates/gpui/src/key_dispatch.rs:845,1045` example impls as a template.
- Register via `Window::use_input_handler` (`window.rs:4058-4078`) so platform IME (macOS NSTextInputClient / Linux Wayland zwp_text_input_v3) drives it for real typing, composition, and paste — this plumbing already works, do not reimplement it.
- Render: caret as a thin `div()` positioned at `selected_range` end, blinking via existing `AnimationDuration`/`Animated` trait; click-to-position and drag-to-select via mouse event handlers on the text container.
- Style: border gray-300, rounded-md, px-3 py-2, `focus_ring()` (Phase 01) on focus, placeholder gray-400 text when `content.is_empty()`.
- Scope: single-line only, no wrapping, no multi-cursor — matches Tailwind's basic text input, not a full editor.

**Textarea** (new `crates/ui/src/components/textarea.rs`):
- Same input primitive as Text Input but multi-line, min-height 6rem (96px), resize handled by fixed sizing initially (no drag-resize — desktop-native drag-resize is a stretch goal, not required for parity).

**Select** (new `crates/ui/src/components/select.rs`):
- Reuse existing `DropdownMenu` (`crates/ui/src/components/dropdown_menu.rs`) as the popover mechanism — Select is a trigger button (styled like text input: border gray-300, rounded-md, chevron-down icon) + `DropdownMenu` popover listing options. Do NOT build a new overlay system; compose existing `DropdownMenu`.

**Radio Button** (new `crates/ui/src/components/radio.rs`):
- 16px (4×4 in Tailwind's rem = 16px) circle, border gray-300, checked: border blue-600 + filled blue-600 inner dot.
- Follow `Checkbox`'s existing pattern in `toggle.rs` as a template (same file structure: state via `checked: bool` prop + `on_click` callback) — group/exclusivity is caller's responsibility (parent view holds `selected: RadioValue`, passes `checked: selected == this_value` to each Radio).

**Checkbox/Switch restyle** (`crates/ui/src/components/toggle.rs`):
- Checkbox: 16px, border gray-300, checked bg-blue-600 + white check icon, `focus_ring()` wired.
- Switch: width 44px height 24px, bg gray-200 (off)/blue-600 (on), thumb white circle, smooth position transition via existing `AnimationDuration::Fast` (150ms) + `Animated`/`AnimationElement` (from `animation.rs`) — reuse existing transition mechanism, do not invent new animation code.

**Label + Help/Error text** (extend `crates/ui/src/components/label/` or add small `form_field.rs` wrapper):
- Label: text-sm font-medium gray-700 equivalent, mb-1 (4px bottom margin).
- Help text: text-xs gray-500, mt-1.
- Error text: text-xs red-600, mt-1; error state also sets input border to red-500 + focus ring red-500 instead of blue-500 (parameterize `focus_ring()` to accept a color, or add `focus_ring_error()` variant).

### Non-functional

- Files under 200 lines each; if TextInput requires substantial key-handling logic, split into `text_input.rs` (public API) + `text_input_state.rs` (internal editing state) if needed.
- No `unwrap()` on any text/clipboard operation.

## Architecture

```
crates/ui/src/components/
├── toggle.rs           (MODIFY — restyle Checkbox + Switch)
├── text_input.rs        (NEW — investigate primitive first)
├── textarea.rs           (NEW — built on text_input primitive)
├── select.rs             (NEW — composes DropdownMenu)
├── radio.rs              (NEW — mirrors Checkbox pattern)
└── form_field.rs         (NEW, optional — Label + help/error wrapper, only if not cleanly fitting into label/ dir)
```

## Related Code Files

**Reference (read before implementing, do not modify):**
- `crates/gpui/src/input.rs` — `EntityInputHandler` trait, `ElementInputHandler<V>` adapter.
- `crates/gpui/src/platform.rs:1275-1373` — platform `InputHandler` trait.
- `crates/gpui/src/window.rs:4058-4078,4511-4566` — `use_input_handler` registration + key dispatch.
- `crates/gpui/src/key_dispatch.rs:845,1045` — example `InputHandler` impls (template).

**Modify:**
- `crates/ui/src/components/toggle.rs`
- `crates/ui/src/components/dropdown_menu.rs` (only if Select needs a new builder method on it, e.g. `.as_select_trigger()`)
- `crates/ui/src/prelude.rs` (export new components)

**Create:**
- `crates/ui/src/components/text_input.rs`
- `crates/ui/src/components/textarea.rs`
- `crates/ui/src/components/select.rs`
- `crates/ui/src/components/radio.rs`

## Implementation Steps

1. Read `crates/gpui/src/input.rs` (`EntityInputHandler`) and the example impls at `crates/gpui/src/key_dispatch.rs:845,1045`; read `window.rs:4058-4078` for registration pattern. Confirm the 8-method contract before writing `text_input.rs`.
2. Build `TextInput`: state struct (`content: String`, `selected_range: Range<usize>`, `FocusHandle`) implementing `EntityInputHandler`; register via `use_input_handler`; render caret + selection highlight; wire mouse click/drag for cursor positioning; style per Tailwind spec + `focus_ring()`.
3. Build `Textarea` reusing step 2's `EntityInputHandler` impl for multi-line (newline handling on Enter, min-height 96px, no drag-resize in v1).
4. Build `Select` composing `DropdownMenu` — trigger styled as input, popover lists options, selection updates trigger label.
5. Build `RadioButton` mirroring `Checkbox` structure from `toggle.rs`.
6. Restyle `Checkbox` colors/size/focus-ring in `toggle.rs`.
7. Restyle `Switch` colors/size, wire `AnimationDuration::Fast` transition for thumb position.
8. Add error-state color parameter to `focus_ring()` (Phase 01 file) or add `focus_ring_error()` sibling function.
9. Build/extend Label + help/error text styling.
10. `cargo check -p ui` clean.
11. Visual verify: Playwright screenshot `tailwindui.com/components/application-ui/forms/*` pages (inputs, checkboxes, radios, toggles, select menus); render each new/restyled component's `preview()` via `VisualTestAppContext`; compare and iterate.

## Todo List

- [ ] Read EntityInputHandler + key_dispatch.rs example impls + use_input_handler registration
- [ ] Build TextInput (real editable, EntityInputHandler-backed, not mock)
- [ ] Build Textarea
- [ ] Build Select (composes DropdownMenu)
- [ ] Build RadioButton
- [ ] Restyle Checkbox (toggle.rs)
- [ ] Restyle Switch (toggle.rs) with transition
- [ ] Add error-state focus ring variant
- [ ] Label + help/error text styling
- [ ] cargo check -p ui clean
- [ ] Visual verify vs Tailwind UI forms pages

## Success Criteria

- TextInput accepts real keyboard input (typed characters appear, backspace deletes) — verified by running gallery app manually, not just compiling.
- Textarea supports multi-line input (Enter creates newline).
- Select opens `DropdownMenu` popover and updates displayed value on selection.
- RadioButton group exclusivity works when composed by a parent view (demonstrated in `preview()`).
- Checkbox/Switch visually match Tailwind spec (colors, sizes) and retain existing functional behavior (no regression for existing callers).
- `make check-all` green.
- Visual comparison documented for each new/restyled component.

## Risk Assessment

- **Risk (MEDIUM):** No ready-made TextInput exists, but the hard 20% (platform IME/key plumbing via `EntityInputHandler`, working on macOS + Linux Wayland) is already vendored and functional — confirmed by direct code read. Remaining work (buffer + selection state, caret render, mouse-to-cursor mapping, ~8 trait methods) is a well-scoped few-hundred-line component, comparable to a minimal single-line text field, not a full editor. **Mitigation:** if actual implementation time materially exceeds the 4h phase budget once started, split TextInput/Textarea into their own sub-phase rather than cutting scope (real typing is non-negotiable per no-mocks rule) — flag early, don't discover late.
- **Risk:** Select's chevron icon and popover z-index/overlay behavior depends on `DropdownMenu` internals not fully explored yet — read `dropdown_menu.rs` fully before composing.
- **Risk:** Radio group state pattern (no built-in "radio group" concept) could be inconsistently implemented by different callers. **Mitigation:** document the recommended pattern clearly in `radio.rs` doc comments + gallery's radio showcase page as the canonical example.

## Security Considerations

- Text input: if used for anything beyond gallery demo (future password fields etc.), note that no masking/secure-entry variant is built in this phase — out of scope, flag for future work if needed.

## Next Steps

- If TextInput/Textarea implementation time runs materially over budget, split into a dedicated sub-phase (`phase-03b`) rather than cutting functional scope.
- Phase 04 (composite/overlay) does not depend on form controls directly but Phase 05 gallery's "Forms" showcase page depends on this phase's components existing.
