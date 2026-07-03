# Phase 03 — Form Controls (Checkbox/Switch restyle + Input Group, Search, Combobox, Multi-Select, Segmented Control, Form Layout, File Input)

## Context Links

- Research: `researcher-01-tailwind-appui-catalog.md` (Forms row: Input Groups, Select Menus, Comboboxes, Checkboxes, Toggles, Action Panels)
- Research: `researcher-02-codebase-audit.md` (Checkbox/Switch 🟡 restyle-pending; TextInput/Select/RadioButton ✅ done; Input Group/Search/File Input/Combobox/Multi-Select/Segmented Control ⬜ missing)
- Phase 01: `./phase-01-gap-analysis-icons.md`
- Plan: `./plan.md` (Cross-Cutting Requirements)

## Overview

- Date: 2026-07-03
- Description: Restyle Checkbox/Switch (`toggle.rs`) to token spec; build Input Group (prefix/suffix), Search Input, Combobox, Multi-Select, Segmented Control, Form Layout/Action Panel, File Input — all composing existing `TextInput`/`Select`/`DropdownMenu`/`RadioButton`/`Chip` bases, no new low-level input primitive.
- Priority: P1
- Implementation status: Pending
- Review status: Not reviewed

## Key Insights

- `TextInput` (`text_input.rs`) is ALREADY a real `EntityInputHandler`-backed editable input (confirmed done in prior plan) — every new form component in this phase composes it, none reimplement text editing.
- `Select` (`select.rs`) already composes `DropdownMenu` — Combobox/Multi-Select extend this same composition, do not build a new overlay primitive.
- Checkbox/Switch restyle is a token-value fix only (`toggle.rs`) — structure/behavior unchanged.
- Multi-Select's tag display reuses `Chip` (Phase 2) — Chip's public API (label/icon/dismiss) is stable regardless of Phase 2's restyle timing (file exists now), so no hard phase-ordering dependency, just consume `Chip::new(...)`.

## Requirements

### Reuse Map

| Tailwind category | GPUI base | Action |
|---|---|---|
| Checkboxes | `components/toggle.rs` (Checkbox) | RESTYLE |
| Toggles/Switches | `components/toggle.rs` (Switch) | RESTYLE |
| Input Groups | `components/text_input.rs` | NEW wrapper `components/input_group.rs` |
| Search Input | `components/text_input.rs` | NEW `components/search_input.rs` (TextInput + MagnifyingGlass + clear button) |
| Comboboxes | `components/select.rs` + `text_input.rs` | NEW `components/combobox.rs` (typeahead-filtered Select) |
| Multi-Select | `components/select.rs` + `components/chip.rs` | NEW `components/multi_select.rs` |
| Segmented Control | `components/radio.rs` pattern | NEW `components/segmented_control.rs` |
| Action Panels / Form Layouts | none | NEW `components/form_field.rs` (label+help/error) + `components/action_panel.rs` |
| File Input | none | NEW `components/file_input.rs` |

### Functional

- **Checkbox/Switch restyle** (`toggle.rs`): Checkbox 16px, `semantic::border`, checked → `palette::primary(600)` bg + white check icon (`IconName::Check`), `focus_ring()`. Switch 44×24px, off → `semantic::border_muted`/neutral bg, on → `palette::primary(600)`, white thumb, `AnimationDuration::Fast` transition (reuse existing mechanism, do not invent new animation code).
- **Input Group** (new): wraps `TextInput` with a leading and/or trailing slot (icon or button), border unified across the whole group (input's own border suppressed at the joined edge).
- **Search Input** (new): `TextInput` + `IconName::MagnifyingGlass` prefix + conditional clear (`x-mark`) button when non-empty, calling `TextInput`'s existing clear/set-content API.
- **Combobox** (new): `TextInput` (typed filter) + `DropdownMenu` popover showing filtered options; selecting an option sets the input's display text. Minimal typeahead — **case-insensitive substring match** only (user-confirmed 2026-07-03), no fuzzy-match / async / remote-data engine.
- **Multi-Select** (new): trigger area showing selected values as `Chip`s (dismissible) + `DropdownMenu` popover checklist; each option toggle adds/removes a Chip.
- **Segmented Control** (new): horizontal button-like group where exactly one option is active (mirrors `RadioButton`'s checked/on_click pattern per component, but rendered as a connected pill/segment row, similar visual family to Phase 2's `ButtonGroup` — reuse its connected-border approach if convenient, no hard dependency).
- **Form Layout / Action Panel** (new): `form_field.rs` = Label (text-sm font-medium) + child input + optional help text (text-xs `semantic::text_muted`) + optional error text (text-xs `palette::danger(600)`, also sets child focus ring to `focus_ring_error()`). `action_panel.rs` = bordered/bg-`semantic::elevated_surface` section with a fieldset area + `border-t` + right-aligned Save/Cancel `Button`s.
- **File Input** (new): styled trigger area (dashed border `semantic::border`, upload icon, "Click to upload" label) — file-picker OS integration only if a trivial existing hook exists in `gpui`/`gpui_platform` (grep first); otherwise this is a presentational trigger with an `on_click` callback the caller wires to their own file dialog (document this limitation in the file's doc comment — this is a real scope boundary, not scope-cutting: no OS file-dialog API exists in this codebase to call).

### Non-functional

- Files under 200 lines each; if Combobox/Multi-Select typeahead logic grows large, split state into a sibling `_state.rs` file.
- No `unwrap()` on any text/selection operation.

## Architecture

```
crates/ui/src/components/
├── toggle.rs              (MODIFY — restyle Checkbox + Switch)
├── input_group.rs          (NEW)
├── search_input.rs         (NEW)
├── combobox.rs              (NEW)
├── multi_select.rs          (NEW)
├── segmented_control.rs      (NEW)
├── form_field.rs             (NEW)
├── action_panel.rs           (NEW)
└── file_input.rs              (NEW)
```

## Related Code Files

**Read first (compose, don't duplicate):** `text_input.rs`, `select.rs`, `dropdown_menu.rs`, `radio.rs`, `chip.rs` (Phase 2's final API).

**Modify:** `toggle.rs`, `crates/ui/src/components.rs`, `crates/ui/src/prelude.rs`.

**Create:** the 7 new files listed above.

## Implementation Steps

1. Restyle Checkbox + Switch in `toggle.rs` (colors/sizes/focus-ring/transition).
2. Read `text_input.rs`, `select.rs`, `dropdown_menu.rs` public APIs fully before composing.
3. Build `InputGroup` (prefix/suffix wrapper).
4. Build `SearchInput` (icon + clear button).
5. Build `Combobox` (filtered TextInput + DropdownMenu).
6. Build `MultiSelect` (Chip tags + DropdownMenu checklist).
7. Build `SegmentedControl` (connected-segment single-select).
8. Build `FormField` (label+help/error wrapper) and `ActionPanel` (fieldset+save/cancel).
9. Build `FileInput` (styled trigger, document file-dialog limitation in doc comment).
10. Update/add `preview()` for all 9 deliverables.
11. `cargo check -p ui` clean.
12. `cargo run -p ui_gallery` — manually verify Checkbox/Switch behave correctly (real click toggles state, not just renders).

## Todo List

- [ ] Restyle Checkbox (toggle.rs)
- [ ] Restyle Switch (toggle.rs) with transition
- [ ] Build InputGroup
- [ ] Build SearchInput
- [ ] Build Combobox
- [ ] Build MultiSelect
- [ ] Build SegmentedControl
- [ ] Build FormField + ActionPanel
- [ ] Build FileInput (+ doc-comment the file-dialog limitation)
- [ ] preview() for all 9
- [ ] cargo check -p ui clean
- [ ] Manual click-test Checkbox/Switch/Combobox/MultiSelect in gallery

## Success Criteria

- `make check` + `make check-all` + `cargo fmt --all --check` green.
- Checkbox/Switch: real click toggles state (not mocked), visuals match token spec.
- Combobox/MultiSelect: typing filters options, selecting updates trigger display, real interaction verified in `cargo run -p ui_gallery`.
- FileInput: doc comment clearly states no OS file-dialog wiring (caller responsibility) — not silently pretending to be complete.
- No regression to existing `TextInput`/`Select`/`DropdownMenu`/`Chip`/`RadioButton` callers.

## Risk Assessment

- **Risk:** Combobox/MultiSelect typeahead could balloon into a full autocomplete engine. **Mitigation:** explicitly scope to substring filter only (documented in Requirements above); if more is needed later, that's a follow-up, not silently expanded here.
- **Risk:** FileInput without real OS file-dialog access could look "fake" if not documented. **Mitigation:** doc comment + gallery preview clearly labels it as a styled trigger, caller wires the callback — this is a genuine platform-capability boundary (no file dialog API in this codebase), not a cut corner.

## Security Considerations

- FileInput: no actual file I/O in this component (by design, see above) — zero attack surface here.

## Next Steps

- Phase 9 gallery Forms page must showcase all 9 new/restyled deliverables from this phase.
- Phase 4's Description List / Stats Card do not depend on this phase.
