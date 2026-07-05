//! Playwright-style actionability checks (backend-agnostic).
//!
//! Stages: present → stable → (visible) → enabled → not-covered. "Visible" is
//! folded into "present": the registry only returns a snapshot for an element
//! painted in the current frame (see [`ElementRegistry::get`]), so a resolved
//! snapshot is by definition visible-this-frame.
//!
//! NOTE: "not-covered" is a bounds-overlap HEURISTIC, not a real hit-test (the
//! flat registry has no z-order). It can miss true occlusion; real occlusion is
//! Phase 06's `hit_test` job (ADR 0007). `assert_visible` therefore does NOT
//! guarantee "the compositor confirms this pixel is unobstructed".

use std::time::Duration;

use gpui::{Bounds, Pixels, Point, SharedString};

use crate::registry::ElementSnapshot;

/// The stage a wait reached when it failed (reported via `ProbeError::Timeout`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionabilityStage {
    Present,
    Stable,
    Enabled,
    NotCovered,
}

/// Retry/backoff budget for a wait. In-process resolution is ~instant, so the
/// default total is far below Playwright's 30s — it only bounds pathological or
/// never-parking waits (see `test_platform`'s pump loop).
#[derive(Clone, Copy, Debug)]
pub struct WaitConfig {
    pub timeout: Duration,
    pub poll_interval: Duration,
    pub stable_polls: u32,
}

impl Default for WaitConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(2),
            poll_interval: Duration::from_millis(10),
            stable_polls: 2,
        }
    }
}

impl WaitConfig {
    /// Max poll iterations for this budget (at least enough to satisfy stability).
    pub fn max_polls(&self) -> u32 {
        let by_time = (self.timeout.as_millis() / self.poll_interval.as_millis().max(1)) as u32;
        by_time.max(self.stable_polls) + 1
    }
}

/// Geometric center of a bounds rect — the point actions are dispatched at.
pub fn center(b: Bounds<Pixels>) -> Point<Pixels> {
    Point {
        x: b.origin.x + b.size.width / 2.0,
        y: b.origin.y + b.size.height / 2.0,
    }
}

/// Tracks how many consecutive polls reported identical bounds.
pub struct Stability {
    last: Option<Bounds<Pixels>>,
    count: u32,
}

impl Stability {
    pub fn new() -> Self {
        Self {
            last: None,
            count: 0,
        }
    }

    pub fn observe(&mut self, bounds: Bounds<Pixels>) {
        if self.last == Some(bounds) {
            self.count += 1;
        } else {
            self.last = Some(bounds);
            self.count = 1;
        }
    }

    pub fn is_stable(&self, needed: u32) -> bool {
        self.count >= needed
    }
}

impl Default for Stability {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluate all actionability stages for the current poll. Returns the action
/// point (bounds center) when fully actionable, or the stage that failed.
///
/// `covered_by` is the heuristic occlusion check (returns the id of an element
/// judged to be on top, if any).
pub fn evaluate(
    snapshot: Option<&ElementSnapshot>,
    stability: &Stability,
    cfg: &WaitConfig,
    covered_by: impl FnOnce(&ElementSnapshot) -> Option<SharedString>,
) -> Result<Point<Pixels>, ActionabilityStage> {
    let snap = snapshot.ok_or(ActionabilityStage::Present)?;
    if !stability.is_stable(cfg.stable_polls) {
        return Err(ActionabilityStage::Stable);
    }
    if !snap.enabled {
        return Err(ActionabilityStage::Enabled);
    }
    if covered_by(snap).is_some() {
        return Err(ActionabilityStage::NotCovered);
    }
    Ok(center(snap.bounds))
}
