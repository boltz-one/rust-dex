# 0007. Element-Tree Access: Public canvas()-Based Registry (primary), hit_test/AccessKit upstreamed into crates/gpui before enablement

- **Status:** accepted
- **Date:** 2026-07-06
- **Lane:** high-risk

## Context

`gpui-probe` (GPUI inspector + UI test driver) is a new reusable crate that lives
**inside this workspace (`boltz-gpui` / `rust-dex`)** alongside `crates/gpui`
(published as `boltz-gpui`, currently v0.2.4). The crate depends on `crates/gpui`
via a workspace **path + version** dependency (not a pinned `=0.2.4` literal, and
not a git checkout), and is itself published Apache-2.0 to crates.io as a
standalone reusable crate.

Both the inspector overlay and the test driver need one shared way to answer
"where is element X, is it visible/enabled" against `boltz-gpui`. Two research
reports plus direct verification against the in-workspace `crates/gpui` source
were used (line refs below are against that source).

Initial assumption (from research) was that `Window::rendered_frame.debug_bounds:
FxHashMap<String, Bounds<Pixels>>` is a public field populated every frame — a
ready-made, fork-free seam. **This is incorrect**, verified directly:

- `Frame::debug_bounds` is `pub(crate)` (`window.rs:805`), and
  `Window::rendered_frame` is also `pub(crate)` (`window.rs:982`). Neither is
  reachable from an external crate by field access, in any build mode.
- The only way in, `VisualTestContext::debug_bounds(selector)`
  (`app/test_context.rs:763`), is a method defined *inside* the gpui crate itself
  (so it can see the private field) — it is not a general escape hatch.
- Population itself only happens when `.debug_selector(f)` is called on an element
  AND the crate is compiled with `cfg(any(test, feature = "test-support"))`
  (`elements/div.rs:789-801`); outside that cfg, `.debug_selector()` is a hard
  no-op (`div.rs:794-801`). This means the mechanism **cannot ever populate in a
  shipped release window**, regardless of which Cargo features a consuming app
  enables — the `test`/`test-support` gate is unconditional.
- `VisualTestContext` itself (and all `simulate_*` methods) lives in a module gated
  the same way (`app.rs:30-68`), and is bound to `TestPlatform`/`TestWindow`, not
  real on-screen windows.

A separate, usable public seam was found: `gpui::canvas()` (`elements/canvas.rs:10`)
— "a canvas element, meant for accessing the low level paint API without defining
a whole custom element". Its `prepaint`/`paint` closures receive the element's real
`Bounds<Pixels>` every frame, with **no cfg gate at all** — it compiles and runs
identically in debug, release, real windows, and TEST windows.

Also verified (relevant to ADR 0009, noted here since it affects scope):
`Window::hit_test()` is `pub(crate)` (`window.rs:905`) — general "what's under this
point" queries are genuinely unavailable today without changing gpui itself. But
`Window::dispatch_event(&mut self, event: PlatformInput, cx: &mut App) ->
DispatchEventResult` **is already `pub`** (`window.rs:4274`), and
`PlatformInput`/`MouseDownEvent`/`MouseUpEvent`/`Keystroke::parse` are all public
with public fields. Routing (`hit_test`) is private, but *dispatching a known
event* is not.

Because `gpui-probe` lives in the same workspace as `crates/gpui`, the missing
capabilities are not a "maintain a fork" problem — they are an **upstream** problem:
the seams can be made `pub` in `crates/gpui` directly, released as a new
`boltz-gpui` version, and then depended on cleanly. There is no need for a
temporary `[patch]` fork.

## Decision

**PRIMARY (available on the current released `boltz-gpui`, no gpui change needed):
a public-API parallel registry, built on `canvas()`.**

- `gpui-probe` provides `probe::track(id: impl Into<SharedString>, element: impl
  IntoElement) -> impl IntoElement`, which wraps `element` together with a
  same-bounds `canvas()` sibling. The canvas's `paint` closure writes
  `ElementSnapshot { id, bounds, enabled, frame_seq }` into a `gpui::Global`
  registry (`ElementRegistry`) every paint pass.
- Opt-in per element (app calls `.probe(id)` on elements it wants
  inspectable/testable) — matches the "min opt-in for consuming apps" constraint.
  No cfg gate, no feature flag, works in real windows and TEST windows identically.
- This registry is the single shared data source consumed by both the inspector
  and the test driver.

**DEFERRED, UPSTREAM-FIRST (not a fork): expose `Window::hit_test()` as public API
in `base/crates/gpui` and add an AccessKit consumer there, then release a new
`boltz-gpui` version, and only then enable the dependent `gpui-probe` capabilities.**
Needed only for capabilities the registry cannot provide:
1. General "what's at point (x, y)" queries without a known test-id (registry only
   answers "where is test-id X", not "what is at this pixel").
2. True occlusion/z-order-aware hit-testing (the registry's own "not covered" check
   is a bounds-overlap heuristic, not a real paint-order hit-test).
3. Semantic role/label selectors (ADR 0008) — blocked on AccessKit, confirmed zero
   references in the crate source, requiring a new consumer to be written from
   scratch either way.

The sequence is strict: **land the `pub` `hit_test` + AccessKit consumer in
`crates/gpui` upstream → cut a new `boltz-gpui` release → bump `gpui-probe`'s
workspace dependency to it → then turn on the dependent features.** No temporary
`[patch]`-based fork of gpui is used at any point.

`Window::dispatch_event()` does **not** need any upstream change — it is already
public and used as-is by both the TEST-platform driver (via
`VisualTestContext::simulate_*`, which call it internally) and, optionally, a
real-window driver backend (ADR 0009) that calls it directly via
`WindowHandle::update`.

## Alternatives Considered

- **Read the private field directly** — impossible (compile error: field is
  private). Not a real option; listed only because it was the original research
  assumption, corrected here.
- **Require `test-support`/`inspector` feature on consuming apps' production builds**
  to unlock `debug_selector` — rejected. Verified dead end: the non-test-support
  branch of `debug_selector()` (`div.rs:794-801`) is an unconditional no-op
  independent of any feature flag the *app* enables; only `cfg(test)` or gpui's own
  `test-support` feature (in effect only while gpui itself is compiled that way)
  can activate it — and that path is bound to `TestPlatform`, not a shipped window.
- **Full custom `Element` trait implementation per tracked widget** — rejected as
  over-engineering (YAGNI). `canvas()` is gpui's own documented minimal API for
  exactly this ("low level paint API without defining a whole custom element").
- **A temporary `[patch]` fork of gpui as the mechanism for the deferred
  capabilities** — rejected. Because `gpui-probe` and `crates/gpui` share one
  workspace and one release train, the correct move is to upstream the missing
  `pub` surface into `crates/gpui` and release a new `boltz-gpui` version, never to
  carry a patched fork. A fork would fragment the source of `boltz-gpui` and block
  clean crates.io publication of both crates.
- **Registry via a fork from day one** — rejected. It would make every consuming
  app depend on a non-released gpui just to get basic bounds tracking, when the
  public `canvas()` seam delivers the same data on the shipped `boltz-gpui`.

## Consequences

- (+) `gpui-probe`'s core has zero dependency on any non-released gpui or non-default
  Cargo feature; any app on the current `boltz-gpui` can add the dependency and
  start tracking elements today.
- (+) Same registry serves both inspector and driver — one source of truth, per the
  "one element-tree core" requirement.
- (+) Living in the base workspace means the deferred capabilities graduate cleanly
  through an upstream `crates/gpui` change + `boltz-gpui` release, keeping both
  crates publishable to crates.io without a fork.
- (-) Opt-in only: untracked elements are invisible to both inspector and driver (no
  auto-discovery of the full tree). Accepted — matches the "min opt-in" constraint;
  auto-coverage is deferred to the upstream hit_test/AccessKit path.
- (-) "Visible" and "not covered by an overlay" are approximated via bounds/z-
  heuristics, not a real compositor-accurate hit-test, until the upstream `hit_test`
  lands in `crates/gpui` and ships in a `boltz-gpui` release. Documented as a known
  limitation, not silently glossed over.
