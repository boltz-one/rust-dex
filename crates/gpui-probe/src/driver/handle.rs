//! [`ElementHandle`] — the Playwright-style DSL over a [`TestHarness`].
//!
//! Methods block, internally pumping the TEST executor until the element is
//! actionable (present → stable → enabled → not-covered) or the wait budget is
//! exhausted. See [`crate::driver::actionability`] for the staged checks and the
//! "not-covered is a heuristic" caveat.

use std::time::Duration;

use gpui::{Modifiers, Pixels, Point, SharedString};

use crate::driver::ProbeError;
use crate::driver::actionability::{ActionabilityStage, Stability, WaitConfig, evaluate};
use crate::driver::locator::Locator;
use crate::driver::test_platform::TestHarness;

/// A resolved handle to an element, borrowing its [`TestHarness`].
pub struct ElementHandle<'a> {
    harness: &'a mut TestHarness,
    locator: Locator,
}

impl<'a> ElementHandle<'a> {
    pub(super) fn new(harness: &'a mut TestHarness, locator: Locator) -> Self {
        Self { harness, locator }
    }

    fn locator_id(&self) -> SharedString {
        match &self.locator {
            Locator::TestId(id) => id.clone(),
            Locator::Role(r) => SharedString::from(r.to_string()),
            Locator::Label(l) => SharedString::from(l.to_string()),
        }
    }

    fn wait_actionable(&mut self, cfg: WaitConfig) -> Result<Point<Pixels>, ProbeError> {
        let mut stability = Stability::new();
        let mut last_stage = ActionabilityStage::Present;
        for _ in 0..cfg.max_polls() {
            self.harness.pump(cfg.poll_interval);
            let snap = self.harness.snapshot(&self.locator)?;
            if let Some(s) = &snap {
                stability.observe(s.bounds);
            }
            match evaluate(snap.as_ref(), &stability, &cfg, |s| {
                self.harness.covered_by(s)
            }) {
                Ok(point) => return Ok(point),
                Err(stage) => last_stage = stage,
            }
        }
        Err(match last_stage {
            ActionabilityStage::Present => ProbeError::NotFound(self.locator_id()),
            other => ProbeError::Timeout(other),
        })
    }

    /// Wait until actionable, then dispatch a click at the element's center.
    pub fn click(&mut self) -> Result<(), ProbeError> {
        let point = self.wait_actionable(WaitConfig::default())?;
        {
            let mut vcx = self.harness.visual();
            vcx.simulate_click(point, Modifiers::default());
        }
        self.harness.pump(Duration::ZERO);
        Ok(())
    }

    /// Wait until actionable, then type `text` (as input to the focused element).
    pub fn type_text(&mut self, text: &str) -> Result<(), ProbeError> {
        self.wait_actionable(WaitConfig::default())?;
        {
            let mut vcx = self.harness.visual();
            vcx.simulate_input(text);
        }
        self.harness.pump(Duration::ZERO);
        Ok(())
    }

    /// Assert the element is present + stable within `timeout` (visibility only;
    /// ignores enabled/covered).
    pub fn assert_visible(&mut self, timeout: Duration) -> Result<(), ProbeError> {
        let cfg = WaitConfig {
            timeout,
            ..Default::default()
        };
        let mut stability = Stability::new();
        let mut last = ActionabilityStage::Present;
        for _ in 0..cfg.max_polls() {
            self.harness.pump(cfg.poll_interval);
            match self.harness.snapshot(&self.locator)? {
                Some(s) => {
                    stability.observe(s.bounds);
                    if stability.is_stable(cfg.stable_polls) {
                        return Ok(());
                    }
                    last = ActionabilityStage::Stable;
                }
                None => last = ActionabilityStage::Present,
            }
        }
        Err(ProbeError::Timeout(last))
    }

    /// Assert the element is absent (goes/stays stale) within `timeout`.
    pub fn assert_not_present(&mut self, timeout: Duration) -> Result<(), ProbeError> {
        let cfg = WaitConfig {
            timeout,
            ..Default::default()
        };
        for _ in 0..cfg.max_polls() {
            self.harness.pump(cfg.poll_interval);
            if self.harness.snapshot(&self.locator)?.is_none() {
                return Ok(());
            }
        }
        Err(ProbeError::Timeout(ActionabilityStage::Present))
    }
}
