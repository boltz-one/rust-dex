---
title: "Phase 3 — Forms"
status: pending
effort: 10h
---

# Phase 3: Forms

[← plan.md](./plan.md) | Prev: [phase-02](./phase-02-core-elements.md) | Next: [phase-04](./phase-04-overlays.md)

## Context
shadcn "Forms" category, 13 catalog rows, 1 explicit skip (Native Select — web-only concept, N/A for a native desktop app). Mix of aligns and real new interactive components (Checkbox, Switch, Slider, Input OTP) needing dedicated state machines.

## Component Table

| Component | Codebase file | Action | Notes |
|---|---|---|---|
| Input | `text_input.rs` | Align | Verify sm/default/lg sizes + focus/disabled/invalid states |
| Input Group | `input_group.rs` | Align | Verify Group/Addon/Input anatomy |
| Textarea | `text_input.rs` (extend) | Align/Extend | Add multiline mode if not present — confirm via direct read (research flagged `[unverified]`) |
| Label | `label/` | Align | Verify disabled-via-peer behavior has a GPUI equivalent (no CSS `:has`/peer selector — likely explicit `disabled` prop passed down instead) |
| Checkbox | none dedicated (research flagged shared w/ `toggle.rs`, `[unverified]`) | New/Verify first | **Read `toggle.rs` fully first** to confirm whether Checkbox already exists under a different name; if genuinely absent, new `checkbox.rs`: checked/indeterminate/disabled, sm/default sizes |
| Radio Group | `radio.rs` | Align | Verify Root/Item anatomy, checked/disabled states |
| Switch | none dedicated (same unverified-shared status as Checkbox) | New/Verify first | Same verify-first step; if absent, new `switch.rs` built on `toggle.rs`'s interaction pattern, sm/default sizes |
| Select | `select.rs` | Align | Verify Trigger/Content/Item/Group/Label anatomy sub-parts |
| Native Select | — | **Skip (documented)** | Web-only `<select>` wrapper concept, no desktop-native equivalent — not built, note in gallery/docs why |
| Combobox | `combobox.rs` | Align + Extend | Add fuzzy-filter matching (research: no fuzzy-match crate wired yet — evaluate `nucleo` or `fuzzy-matcher` as a new `crates/ui` dependency, or hand-roll simple subsequence match if adding a crate is out of this phase's budget; document choice) |
| Slider | none | New | Root/Track/Range/Thumb; pointer-drag geometry: convert mouse-x delta within track bounds to a 0.0-1.0 value, clamp, support range (two-thumb) mode |
| Input OTP | none | New | Group/Slot/Separator; per-slot focus management (Tab/arrow-key moves focus between slots), paste-splits-across-slots handling |
| Form (layout+validation) | none as a component; may reuse `form_field.rs` | New | GPUI has no React-Hook-Form equivalent — build a **layout + validation-state struct only** (e.g. a `FormField` wrapper carrying an `Option<SharedString>` error + touched/dirty flags feeding `field.rs`'s Error slot). Do NOT attempt schema-validation-library parity — explicitly out of scope, note this in the component's doc comment so it's not mistaken for incomplete work later |

## Key Insights
- Two items ("Checkbox", "Switch") have **unverified current state** per research (open question #4: might already exist under `toggle.rs` sharing, or might be genuinely missing) — this phase's first concrete action must be reading `toggle.rs` end-to-end before writing any new file, to avoid duplicating existing functionality (DRY).
- Slider and Input OTP are the two real engineering items in this phase (pointer-drag math, per-slot focus state machine) — budget them first within the 10h.
- Combobox's fuzzy-filter is the one item in this phase with a genuine external-dependency decision (add a crate vs. hand-roll) — resolve and document the choice rather than silently picking one.

## Requirements
- Checkbox/Switch: confirm via `toggle.rs` read before deciding new-file vs. rename-surface; if shared, add clearly-named wrapper types (`Checkbox`, `Switch`) around the shared internals rather than exposing `Toggle` under two names with no distinction (shadcn API expects distinct semantic components).
- Slider: must support both single-value and two-thumb range mode (shadcn's `Slider` supports both via prop).
- Input OTP: paste handling must split pasted text across remaining empty slots starting at current focus, not just fill slot 1.
- Form: the validation-state struct must compose with `form_field.rs`'s existing Error slot — no parallel/competing error-display mechanism.

## Architecture
- `checkbox.rs`/`switch.rs` (if new): thin components rendering via the same click/keyboard-toggle interaction primitive `toggle.rs` already implements — extract that primitive into a shared internal fn/trait if duplicating it would violate DRY, but only if extraction doesn't require touching `toggle.rs`'s public API (additive-safe internal refactor only).
- `slider.rs`: standalone, drag state via GPUI's pointer-drag event handling (mirror whatever existing component in the codebase already does drag geometry, if any — check `scrollbar.rs` for its thumb-drag math as a starting reference pattern).
- `input_otp.rs`: standalone, `Vec<SharedString>` slot state + focus index, `Focusable` per slot.
- Combobox fuzzy-filter: if adding a crate, add to `crates/ui/Cargo.toml` under `[dependencies]`, gated to just this feature's use.

## Related Files
- `crates/ui/src/components/text_input.rs`, `input_group.rs`, `label/`, `toggle.rs`, `radio.rs`, `select.rs`, `combobox.rs`, `form_field.rs`
- `crates/ui/src/components/scrollbar.rs` (drag-geometry reference for Slider)
- New: `crates/ui/src/components/{checkbox,switch,slider,input_otp}.rs` (checkbox/switch only if confirmed missing)
- `crates/ui/Cargo.toml` (only if fuzzy-match crate added)

## Implementation Steps
1. Read `toggle.rs` fully — resolve Checkbox/Switch open question before anything else.
2. Align Input/Input Group/Textarea/Label/Radio Group/Select against shadcn anatomy; log gaps found.
3. Build/verify Checkbox, Switch per step 1's finding.
4. Build Slider (single-value first, then range mode).
5. Build Input OTP (fixed-length slots first, then paste-split logic).
6. Extend Combobox with fuzzy filter (resolve crate-vs-hand-roll first).
7. Build Form validation-state wrapper on `form_field.rs`.
8. Document Native Select skip (one paragraph, gallery "not applicable" note or docs file).
9. Gallery entries + interactive `#[gpui::test]` for Slider drag, Input OTP paste-split, Checkbox/Switch toggle, Combobox filter-narrows-results.

## Todo
- [ ] Resolve Checkbox/Switch existing-vs-missing
- [ ] Input/InputGroup/Textarea/Label/RadioGroup/Select aligned
- [ ] Checkbox done
- [ ] Switch done
- [ ] Slider (single + range) done
- [ ] Input OTP (fixed-length + paste-split) done
- [ ] Combobox fuzzy filter (dependency decision documented)
- [ ] Form validation-state wrapper on form_field.rs
- [ ] Native Select skip documented
- [ ] Gallery + harness tests for all interactive items
- [ ] `cargo build -p ui` / `cargo test -p ui` clean

## Success Criteria
- 12 in-scope components present + gallery-visible; Native Select explicitly documented as skipped with rationale.
- Slider drag and Input OTP paste-split each have a passing `#[gpui::test]`.
- No existing `toggle.rs`/`text_input.rs`/`select.rs`/`combobox.rs` public API broken.

## Risk & Dependencies
- Risk: Checkbox/Switch could reveal deeper toggle.rs coupling than expected — budget extra time if extraction is needed.
- Risk: fuzzy-match crate addition needs a quick compile/size sanity check (keep it lightweight, `nucleo`-class, not a heavy NLP dep).
- Depends on Phase 1 tokens (input_border, ring roles) for focus states.

## Security
Input OTP/paste handling: sanitize pasted content to slot-length alphanumeric only (no injection risk since it's local UI state, but truncate/reject overlong paste defensively).

## Next
[phase-04-overlays.md](./phase-04-overlays.md)
