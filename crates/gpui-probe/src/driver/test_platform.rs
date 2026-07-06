//! TEST-platform backend: [`TestHarness`] (ADR 0009). The only file touching
//! `TestAppContext`/`VisualTestContext` (gpui `test-support`, dev-only). The
//! [`ElementHandle`](crate::driver::handle::ElementHandle) DSL lives alongside
//! in `handle.rs` and drives this harness via its `pub(super)` helpers.

use std::time::Duration;

use gpui::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Pixels, Render, SharedString,
    TestAppContext, TestDispatcher, VisualTestContext, Window,
};

use crate::driver::ProbeError;
use crate::driver::actionability::center;
use crate::driver::handle::ElementHandle;
use crate::driver::locator::Locator;
use crate::registry::{ElementRegistry, ElementSnapshot, ElementTree};

pub(super) fn area(b: Bounds<Pixels>) -> f32 {
    f32::from(b.size.width) * f32::from(b.size.height)
}

/// A headless TEST-platform window plus the DSL entry point. Construct with
/// [`TestHarness::new`], then drive via [`TestHarness::find`].
pub struct TestHarness {
    cx: TestAppContext,
    window: AnyWindowHandle,
}

impl TestHarness {
    /// Open a headless test window whose root view is built by `build`. The same
    /// construction path a real app uses can be adapted here (Phase 08).
    pub fn new<V, F>(build: F) -> Self
    where
        V: 'static + Render,
        F: FnOnce(&mut Window, &mut Context<V>) -> V,
    {
        let mut cx = TestAppContext::build(TestDispatcher::new(0), Some("gpui_probe_harness"));
        let handle = cx.add_window(build);
        let window: AnyWindowHandle = handle.into();
        let mut harness = Self { cx, window };
        harness.pump(Duration::ZERO);
        harness
    }

    /// Resolve a locator into a handle for driving/asserting.
    pub fn find(&mut self, locator: impl Into<Locator>) -> ElementHandle<'_> {
        ElementHandle::new(self, locator.into())
    }

    /// The current frame's tracked-element tree (consumed by [`crate::snapshot`]).
    pub fn snapshot_tree(&self) -> ElementTree {
        self.cx
            .update(|app| app.global::<ElementRegistry>().snapshot_tree())
    }

    pub(super) fn visual(&self) -> VisualTestContext {
        VisualTestContext::from_window(self.window, &self.cx)
    }

    /// Advance one observation frame: bump the registry frame counter, repaint,
    /// advance the simulated clock, and drain ready tasks.
    pub(super) fn pump(&mut self, advance: Duration) {
        self.cx
            .update(|app| app.default_global::<ElementRegistry>().begin_frame());
        // Force the window to repaint so present elements are re-stamped with
        // the new frame counter (refresh() alone via the app doesn't redraw).
        let _ = self
            .cx
            .update_window(self.window, |_view, window, _app| window.refresh());
        if !advance.is_zero() {
            self.cx.executor().advance_clock(advance);
        }
        self.cx.run_until_parked();
    }

    pub(super) fn snapshot(
        &self,
        locator: &Locator,
    ) -> Result<Option<ElementSnapshot>, ProbeError> {
        match locator {
            Locator::TestId(id) => Ok(self
                .cx
                .update(|app| app.global::<ElementRegistry>().get(id))),
            Locator::Role(_) | Locator::Label(_) => Err(ProbeError::Unimplemented(
                "semantic selectors (Role/Label) require the Phase 06 upstream hit_test/AccessKit \
                 change; see docs/decisions/0008",
            )),
        }
    }

    /// Heuristic occlusion check (no z-order): a smaller visible element whose
    /// bounds contain this one's center is treated as "on top" (real occlusion → Phase 06).
    pub(super) fn covered_by(&self, target: &ElementSnapshot) -> Option<SharedString> {
        let c = center(target.bounds);
        let target_area = area(target.bounds);
        self.cx.update(|app| {
            app.global::<ElementRegistry>()
                .all_visible()
                .filter(|s| s.id != target.id)
                .find(|s| s.bounds.contains(&c) && area(s.bounds) < target_area)
                .map(|s| s.id.clone())
        })
    }
}
