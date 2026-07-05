//! Element locators (ADR 0008: test-id-first, semantic reserved).
//!
//! `TestId` is the only variant resolved today. `Role`/`Label` exist so the DSL
//! is built against its final surface, but resolve to
//! [`ProbeError::Unimplemented`](crate::driver::ProbeError::Unimplemented) until
//! the Phase 06 upstream `hit_test`/AccessKit work lands.

use gpui::SharedString;

/// How to locate a tracked element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Locator {
    /// Match by the `id` an element was `.probe()`-ed with.
    TestId(SharedString),
    /// Reserved: match by accessibility role (Phase 06).
    Role(&'static str),
    /// Reserved: match by accessibility label (Phase 06).
    Label(&'static str),
}

impl Locator {
    /// Construct a test-id locator.
    pub fn id(id: impl Into<SharedString>) -> Locator {
        Locator::TestId(id.into())
    }
}

/// Shorthand for [`Locator::id`].
pub fn find_by_test_id(id: impl Into<SharedString>) -> Locator {
    Locator::id(id)
}

impl From<&str> for Locator {
    fn from(s: &str) -> Self {
        Locator::TestId(SharedString::from(s.to_owned()))
    }
}

impl From<String> for Locator {
    fn from(s: String) -> Self {
        Locator::TestId(SharedString::from(s))
    }
}

impl From<SharedString> for Locator {
    fn from(s: SharedString) -> Self {
        Locator::TestId(s)
    }
}
