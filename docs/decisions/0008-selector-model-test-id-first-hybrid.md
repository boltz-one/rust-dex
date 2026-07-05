# 0008. Selector Model: Test-ID first, semantic (Role/Label) reserved until hit_test/AccessKit land in crates/gpui

- **Status:** accepted
- **Date:** 2026-07-06
- **Lane:** normal

## Context

`gpui-probe` needs one locator model shared by the inspector and the driver.
Research-02 (prior-art synthesis of Playwright/Testing-Library/egui_kittest/kittest)
recommends role-first querying via AccessKit as industry best practice, with test-id
as a last resort. Research-01, confirmed by direct source inspection (ADR 0007),
found that `boltz-gpui` has **zero** AccessKit integration (`grep accesskit` → 0 hits
in `crates/gpui/src/`) — no roles, no labels, no accessibility tree at all. Wiring one
in means writing a new AccessKit consumer against gpui's paint/layout internals from
scratch.

Because `gpui-probe` and `crates/gpui` share this workspace, that work is done
**upstream in `crates/gpui`** (adding the AccessKit consumer + public `hit_test`),
then released as a new `boltz-gpui` version, and only then consumed by `gpui-probe` —
not via a temporary fork/patch (ADR 0007).

Meanwhile, ADR 0007 established that a test-id-keyed registry (`probe::track(id,
element)` via `canvas()`) is fully available today on the shipped `boltz-gpui`, no
gpui change, no cfg gate.

## Decision

Adopt a hybrid locator that is **test-id-first now**:

```rust
pub enum Locator {
    TestId(SharedString),   // implemented now — resolves via ElementRegistry (ADR 0007)
    Role(&'static str),     // reserved — Err(ProbeError::Unimplemented) until upstream hit_test/AccessKit lands in crates/gpui
    Label(&'static str),    // reserved — Err(ProbeError::Unimplemented) until upstream hit_test/AccessKit lands in crates/gpui
}
```

- `TestId` is the only working variant today. All driver DSL
  (`find(Locator::id("submit-button"))` / shorthand
  `find_by_test_id("submit-button")`) and inspector selection operate on it.
- `Role`/`Label` variants are part of the public enum surface from the first release
  (so DSL and call sites written today do not need a breaking change later), but
  return a typed `ProbeError::Unimplemented` if invoked before the upstream
  hit_test/AccessKit work lands in `crates/gpui` and ships in a new `boltz-gpui`
  release.
- Once the AccessKit consumer is upstreamed into `crates/gpui` and released,
  `Role`/`Label` are implemented by traversing the AccessKit tree the same way
  `egui_kittest`/`kittest` do (per research-02) and folded into the same
  `ElementNode` tree model, so driver/inspector code does not need to know which
  locator kind resolved a match.

## Alternatives Considered

- **Role-first now** (research-02's default recommendation) — rejected for *now*,
  not outright: blocked on a verified, not assumed, gap (zero AccessKit refs in
  `crates/gpui`). Building "role-first" against a tree that doesn't exist isn't a
  trade-off, it's building on nothing. Reserving the enum variant preserves the
  option without paying the cost before the substrate exists (i.e. before AccessKit
  is upstreamed into `crates/gpui`).
- **Test-id only, forever** — rejected: forecloses the ergonomics research-02
  documents as standard practice (resilience to tree churn, assistive-tech
  alignment) and would force a breaking `Locator` API change whenever AccessKit
  support lands. Reserving the variant now is free (an enum arm + a documented
  error).
- **Geometric/coordinate selectors** (research-02 explicitly rules this out: "no
  geometric fallback needed if accessibility tree is correct") — not adopted as a
  locator kind; coordinates remain an internal implementation detail of dispatching
  an already-resolved bounds (ADR 0007), never a user-facing selector.

## Consequences

- (+) Zero breaking API change is needed to add semantic selectors later — the
  surface exists, only the resolution logic is deferred until the upstream
  `crates/gpui` change ships in `boltz-gpui`.
- (+) 100% of current functionality ships without any AccessKit dependency, on the
  released `boltz-gpui`.
- (-) Test-id churn is the app author's responsibility (no automatic resilience to
  renames) until the upstream hit_test/AccessKit path lands in `crates/gpui` and a
  new `boltz-gpui` release enables `Role`/`Label`. Mitigated by keeping test-id
  assignment centralized at each screen's top-level component construction.
