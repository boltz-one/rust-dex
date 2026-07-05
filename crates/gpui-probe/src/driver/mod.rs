//! Cypress/Playwright-style in-process UI test driver (ADR 0009).
//!
//! [`TestHarness`] opens a headless GPUI TEST-platform window running an app's
//! view; [`TestHarness::find`] returns an [`ElementHandle`] whose
//! `click`/`type_text`/`assert_visible`/`assert_not_present` methods drive and
//! observe the UI, resolving locators against the shared [`crate::registry`].
//!
//! IMPORTANT: `assert_visible`'s "not-covered" check is a bounds-overlap
//! HEURISTIC, not a compositor hit-test (see [`actionability`]). It does not
//! guarantee a pixel is unobstructed — real occlusion is Phase 06's job.

pub mod actionability;
pub mod locator;
// The TEST-platform driver needs gpui's `test-support` symbols
// (TestAppContext/VisualTestContext), so it only compiles when this crate's
// `test-support` feature (which pulls `gpui/test-support`) is on — or under the
// crate's own unit tests. Keeps plain `cargo build` (release) driver-free.
#[cfg(any(test, feature = "test-support"))]
pub mod handle;
#[cfg(any(test, feature = "test-support"))]
pub mod test_platform;

use std::fmt;

use gpui::SharedString;

pub use actionability::{ActionabilityStage, WaitConfig};
#[cfg(any(test, feature = "test-support"))]
pub use handle::ElementHandle;
pub use locator::{Locator, find_by_test_id};
#[cfg(any(test, feature = "test-support"))]
pub use test_platform::TestHarness;

/// Errors surfaced by the driver DSL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProbeError {
    /// No element with the given id was ever painted within the wait budget.
    NotFound(SharedString),
    /// The element was found but never became actionable; carries the stage
    /// that kept failing.
    Timeout(ActionabilityStage),
    /// A reserved locator (Role/Label) that is not yet implemented.
    Unimplemented(&'static str),
    /// The element was judged occluded by another (heuristic).
    CoveredBy(SharedString),
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeError::NotFound(id) => write!(f, "element not found: {id:?}"),
            ProbeError::Timeout(stage) => write!(f, "actionability timeout at stage {stage:?}"),
            ProbeError::Unimplemented(why) => write!(f, "unimplemented locator: {why}"),
            ProbeError::CoveredBy(id) => write!(f, "element covered by {id:?}"),
        }
    }
}

impl std::error::Error for ProbeError {}
