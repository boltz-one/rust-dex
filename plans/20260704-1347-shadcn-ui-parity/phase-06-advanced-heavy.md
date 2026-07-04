---
title: "Phase 6 — Advanced/heavy components (go/no-go)"
status: pending
effort: 8h
---

# Phase 6: Advanced/Heavy Components

[← plan.md](./plan.md) | Prev: [phase-05](./phase-05-data-nav.md) | Next: [phase-07](./phase-07-gallery-verify.md)

## Context
The 5 components research flags as genuinely heavy (each wraps a substantial JS lib/dep with no direct Rust/GPUI equivalent): Calendar, Date Picker, Carousel, Chart, Sonner (toast stack). Per user decision #2 ("TOÀN BỘ ~50 kể cả nặng"), these are IN SCOPE to build, not silently dropped — but each gets a short feasibility spike first because at least one (Chart) has a real dependency decision that needs to be made explicitly rather than assumed. This phase is the one place in the plan where a "no-go" outcome is legitimate (per planning iron rules: dependency-conflict is one of the 3 valid defer reasons) — but a no-go must produce a concrete, cited technical reason, never "too hard."

## Component Table

| Component | What it wraps upstream | Feasibility | Spike question |
|---|---|---|---|
| Sonner (toast stack) | sonner JS lib | 🟡 buildable | Extend `notification.rs` to a queue (stack of N toasts, auto-dismiss timers, enter/exit transitions) vs. new `toast_stack.rs`. No external dep needed — this is the most likely full go. |
| Resizable | *(already built in Phase 5 — not repeated here)* | — | — |
| Calendar | react-day-picker | 🟡 buildable | Date-grid layout + month/locale math, doable natively (~1-2wk per research estimate, budget-compress to what fits this phase; if it doesn't fit, ship a functional single-month grid first, flag multi-month/localization as a follow-up item, not a silent cut) |
| Date Picker | Calendar + Popover composition | 🟡 buildable, depends on Calendar | Straightforward once Calendar exists — Popover-trigger composition only, reuses Phase-4's Popover |
| Carousel | embla-carousel (JS drag/snap physics) | 🟡 buildable, high effort | No embla equivalent in Rust ecosystem — must hand-roll drag-inertia + snap-point animation. Spike: prototype snap-to-nearest-item on drag-release using `animation.rs` helpers; if snap-physics quality is unacceptable in the time budget, ship a functional non-inertial (instant-snap) version and document the simplification explicitly |
| Chart | Recharts (SVG/DOM Bar/Line/Area/Pie/Radar) | ✅ DECIDED: hand-roll via GPUI `canvas()` | **Resolved (2026-07-04): hand-roll minimal chart primitives in a GPUI `canvas()` element (draw paths/arcs directly), NO external plotting crate** — most Rust plotting crates render to an image buffer (not GPUI elements) and off-thread blitting is disproportionate complexity for a UI kit. Scope = **basic chart types only** (Bar, Line, Area, Pie), NOT full Recharts parity (Radar/composed/etc. explicitly deferred, documented — not faked). Colors from `palette::*` (`--chart-1..5` map to distinct palette roles). |

## Key Insights
- 4 of 5 items (Sonner, Calendar, Date Picker, Carousel) are effort-heavy but have no external-dependency blocker — "no-go" is not a legitimate outcome for these under the iron rules (effort alone isn't one of the 3 valid defer reasons); they should be built, potentially in a reduced-but-functional first pass with any trimmed sub-feature explicitly flagged (e.g. "single-month Calendar first, multi-month later") rather than silently declared complete.
- Chart dependency path is RESOLVED (hand-roll via GPUI `canvas()`, no external crate — see table + plan.md Resolved Decision #2). No user sign-off pending; build basic types (Bar/Line/Area/Pie), document deferred advanced types.
- Sonner reuses `notification.rs` — start here, it's the fastest full win and de-risks the phase's time budget for the harder three.

## Requirements
- Sonner: stack must support success/error/info/loading/default variants, auto-dismiss with configurable timer, and correct enter/exit stacking order (newest on top or bottom — match shadcn's default, newest-on-top).
- Calendar: at minimum, single-month grid, day selection, disabled/out-of-range dates, today-indicator — this is the floor, not a placeholder; if multi-month or full locale/i18n month-name support doesn't fit budget, ship the floor and explicitly list what's deferred (with a reason: time budget within this phase, not infeasibility).
- Date Picker: must compose Calendar + Phase-4 Popover with zero duplicated overlay-positioning code.
- Carousel: must support at least manual next/prev navigation + drag-to-advance; snap-inertia quality is the negotiable part, not the existence of drag/navigation.
- Chart: hand-roll basic types (Bar/Line/Area/Pie) via GPUI `canvas()`, no external crate. Advanced types (Radar/composed/scatter) explicitly deferred + documented, not faked.

## Architecture
- `toast_stack.rs` (new, or `notification.rs` extended) — queue of active toast entries + per-entry countdown timer, positioned via the existing deferred+anchored overlay pattern (fixed screen corner, not per-trigger anchored).
- `calendar.rs` (new) — pure date-grid render + selection state, no Popover coupling (Date Picker composes it).
- `date_picker.rs` (new, thin) — Popover(trigger: formatted-date button, content: Calendar).
- `carousel.rs` (new) — item-track div with pointer-drag-driven transform offset, snap-on-release to nearest item boundary.
- `chart.rs` (new) — GPUI `canvas()`-based draw of Bar/Line/Area/Pie; data in, paths/arcs out via the paint context; colors from `palette::*`. No external crate.

## Related Files
- `crates/ui/src/components/notification.rs` (Sonner base)
- `crates/ui/src/components/popover.rs` (Date Picker composition)
- `crates/ui/src/styles/animation.rs` (Carousel snap transition, Calendar month-change transition if any)
- New: `crates/ui/src/components/{toast_stack,calendar,date_picker,carousel}.rs`, `chart.rs` (post-decision)
- (No `crates/ui/Cargo.toml` change — Chart adds no external dependency, hand-rolled via `canvas()`)

## Implementation Steps
1. Build Sonner/toast stack first (fastest win, de-risks remaining budget).
2. Build Calendar (single-month floor first; add multi-month/locale only if budget remains).
3. Build Date Picker on top of Calendar + Popover.
4. Spike Carousel drag+snap; ship functional version, document any inertia/physics simplification explicitly.
5. Build Chart via GPUI `canvas()`: implement Bar/Line/Area/Pie (data → paths/arcs in the paint context, `palette::*` colors, axis/legend). Advanced types (Radar/composed/scatter) explicitly deferred + documented, not faked.
6. Gallery entries for all 5 (or 4 + documented Chart status); `#[gpui::test]` for: toast auto-dismiss timing, Calendar date selection, Date Picker open/select/close, Carousel next/prev + drag-snap.

## Todo
- [ ] Sonner/toast stack built
- [ ] Calendar (single-month floor) built
- [ ] Calendar multi-month/locale — built or explicitly deferred with reason
- [ ] Date Picker built on Calendar+Popover
- [ ] Carousel built (navigation + drag-snap, physics-quality noted)
- [ ] Chart dependency decision surfaced to user and confirmed
- [ ] Chart implemented per confirmed path (or explicitly tracked as pending user decision if not confirmed in time)
- [ ] Gallery entries for all 5
- [ ] Harness tests: toast timing, Calendar selection, Date Picker flow, Carousel nav
- [ ] `cargo build -p ui` / `cargo test -p ui` clean

## Success Criteria
- Sonner, Calendar, Date Picker, Carousel functionally present and gallery-visible, with any trimmed sub-feature explicitly named in code comments + gallery notes (never silently absent).
- Chart either implemented per a user-confirmed path, or explicitly logged as blocked-on-decision (not silently skipped, not unilaterally built on an unconfirmed dependency).

## Risk & Dependencies
- Depends on Phase 4's Popover (Date Picker) and Phase 3's Slider drag-math family (Carousel's drag handling is the same delta-clamp pattern, extended with velocity/snap).
- Risk: this phase's 8h budget is tight for 4 build items + 1 decision gate — if Calendar/Carousel run over, compress scope on the *documented* sub-features (multi-month locale, inertia physics) first, never on the floor feature set defined above.

## Security
Chart (if plotting-crate path chosen): validate any file-path/image-buffer handling for the off-thread render doesn't introduce a path-traversal or unbounded-memory risk from arbitrarily large datasets — bound the render size.

## Next
[phase-07-gallery-verify.md](./phase-07-gallery-verify.md)
