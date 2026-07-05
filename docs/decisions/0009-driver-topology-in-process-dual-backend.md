# 0009. Driver Topology: in-process, dual backend (TEST platform + real window), no IPC

- **Status:** accepted
- **Date:** 2026-07-06
- **Lane:** normal

## Context

We need to decide how the `boltz-gpui-probe` test driver (Cypress/Playwright-style)
actually delivers actions (click/type/wait) to a running `boltz-gpui` app.
Research-02 surveys `egui_kittest` (in-process, frame-by-frame, no IPC) as the
closest analog for an immediate-mode-adjacent Rust UI framework and finds no Rust
precedent for an out-of-process/IPC driver in this space.

Two concrete backends are available on the current `boltz-gpui`, both in-process,
verified by source inspection of `crates/gpui`:

1. **TEST platform** — `TestAppContext::build(...)` (public, `app/test_context.rs:126`)
   + `VisualTestContext` (public, same file) give
   `simulate_click/mouse_down/up/move/keystrokes/input/event`. Gated behind
   `cfg(any(test, feature = "test-support"))` at the module level (`app.rs:30-68`) —
   requires the gpui `test-support` Cargo feature, which is safe to scope to
   `[dev-dependencies]` only (`test-support` never reaches a consuming app's release
   binary this way). Runs against a `TestWindow`, not a real on-screen window.
2. **Real window** — `WindowHandle::update()` (public, `window.rs:5499`) gives
   `&mut Window, &mut App` for any live window; `Window::dispatch_event(event:
   PlatformInput, cx: &mut App)` (public, `window.rs:4274`) accepts a
   manually-constructed `MouseDownEvent`/`MouseUpEvent`/`KeyDownEvent` (all public
   structs, public fields) and runs it through gpui's real (private) hit-test +
   listener dispatch, exactly as a live OS event would. No `test-support` feature
   required.

## Decision

Use an in-process design with two backends behind one `Locator`/`Action` trait,
**no IPC / no out-of-process runner**:

- **`TestHarness` (TEST platform) — PRIMARY.** Used for `cargo test` / CI.
  Deterministic, headless (no real window/compositor needed on CI runners —
  important for Linux CI without a display and for Windows/macOS headless runners).
- **`RealWindowDriver` (live window) — SECONDARY.** Attaches to a real,
  currently-running app window via its `WindowHandle`. Used by the inspector for
  optional "click an element in the tree panel to trigger it in the live app"
  interactions, and available as an escape hatch for smoke-testing the actual
  shipped binary rather than a `TestWindow` stand-in. Not part of the CI-blocking
  test suite (a live window needs a display).
- Both backends share the ADR 0007 registry for locating elements and the same
  `Locator`/`Action` trait surface, so driver DSL code (`find(id).click()`) is
  backend-agnostic at the call site.

## Alternatives Considered

- **Out-of-process / IPC driver** (external runner talking to the app over a
  socket/pipe, JSON or bincode tree snapshots) — rejected. Research-02 found no Rust
  precedent and flags it as "experimental / opinion, handle with care". Confirmed
  independently: since *both* available backends are reachable in-process via public
  API, crossing a process boundary would add serialization, latency, and
  non-determinism (message ordering, timing races) for zero capability gain.
- **TEST-platform-only (no real-window backend)** — rejected. Would satisfy the
  driver alone but leave the inspector ("form = both") with a registry it can *read*
  but never *act* through, undermining the "one element-tree core" shared-core
  requirement. The real-window backend costs little to add since
  `dispatch_event`/`Keystroke::parse` are already public (verified, ADR 0007) — it
  is not a capability gated on the upstream hit_test/AccessKit work.
- **Real-window-only (no TEST platform)** — rejected. CI needs a headless,
  display-less path; driving a real window requires an actual OS window/compositor
  (or a virtual display like Xvfb on Linux, extra CI infra with no upside over the
  purpose-built `TestPlatform`). TEST platform is gpui's own answer to exactly this
  need and should be the CI-facing default.

## Consequences

- (+) Zero IPC/serialization code to write or maintain.
- (+) CI runs headless via TEST platform on all three OSes (matches the existing
  `ci.yml` matrix: macos-latest, ubuntu-latest, windows-latest — none currently run
  a display server for tests).
- (+) Inspector gets a real interaction path (optional click-through) without waiting
  on the upstream hit_test/AccessKit work landing in `crates/gpui`.
- (-) Two backends to test-cover and keep in sync against the shared
  `Locator`/`Action` trait (moderate maintenance surface, mitigated by a shared
  trait + shared registry so the actual divergence is limited to "how do I get a
  `&mut Window`", not "how do I find/click an element").
- (-) `RealWindowDriver`'s "not covered by an overlay" check inherits the same
  bounds-overlap heuristic limitation noted in ADR 0007 (no real hit-test until the
  upstream `hit_test` lands in `crates/gpui` and ships in a `boltz-gpui` release).
