//! The opt-in `.probe(id)` wrapper (ADR 0007).
//!
//! [`track`] wraps an arbitrary `impl IntoElement` together with a sibling
//! `gpui::canvas()` inside a `relative` container. The canvas is the only
//! ungated, public GPUI seam that hands back an element's real
//! `Bounds<Pixels>` every frame, in every build mode (verified: no `cfg` on
//! `canvas()`). On paint it writes those bounds into the [`ElementRegistry`].
//!
//! There is NO `cfg(test)`/feature gate on this path — `.probe()` works
//! identically in `cargo build`, `cargo test`, real windows, and TEST windows.

use gpui::{App, IntoElement, ParentElement as _, SharedString, Styled as _, canvas, div};

use crate::registry::ElementRegistry;

/// Wrap `element` so its real per-frame bounds are recorded under `id` in the
/// [`ElementRegistry`]. `enabled` is the app-defined enabled flag (GPUI has no
/// universal notion of it).
///
/// The wrapper is a `relative` div holding `element` in normal flow plus an
/// absolutely-positioned (`top_0`/`left_0`/`size_full`) sibling `canvas()`, so
/// the canvas fills — and therefore reports — the wrapper's box.
///
/// Caveat 1 (measurement): the reported bounds are the *wrapper's* box, which
/// equals `element`'s bounds when the wrapper hugs its content. That holds in
/// flex parents (the GPUI norm) where the wrapper is a content-sized flex item.
/// In a block parent the block-level wrapper fills the available width, so
/// reported width may exceed the inner element. Real occlusion / exact hit
/// geometry is Phase 06's `hit_test` job (ADR 0007); treat bounds as "tight in
/// flex layouts" until then.
///
/// Caveat 2 (layout participation): the wrapper is NOT layout-transparent
/// (`display: contents` has no GPUI equivalent). Flex-participation styles set
/// on `element` itself — `flex_1()`/`flex_grow()`/`flex_shrink()`/`flex_basis()`
/// — now sit on a grandchild with no siblings, so they no longer influence how
/// the PARENT sizes it (the parent sees the content-sized wrapper instead).
/// Prefer probing leaf / intrinsically-sized elements; if you must probe a
/// flex-growing element, move the growth styles onto a parent you don't probe.
pub fn track(
    id: impl Into<SharedString>,
    enabled: bool,
    element: impl IntoElement,
) -> impl IntoElement {
    let id = id.into();
    div().relative().child(element).child(
        canvas(
            |_bounds, _window, _cx| (),
            move |bounds, _prepaint, _window, cx: &mut App| {
                cx.default_global::<ElementRegistry>()
                    .upsert(id, bounds, enabled);
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full(),
    )
}

/// Fluent `.probe(id)` / `.probe_enabled(id, enabled)` on any `IntoElement`.
///
/// ```ignore
/// use gpui_probe::Trackable as _;
/// my_button.probe("submit-button")
/// ```
pub trait Trackable: IntoElement + Sized {
    /// Track this element under `id`, marked enabled.
    fn probe(self, id: impl Into<SharedString>) -> impl IntoElement {
        track(id, true, self)
    }

    /// Track this element under `id` with an explicit `enabled` flag.
    fn probe_enabled(self, id: impl Into<SharedString>, enabled: bool) -> impl IntoElement {
        track(id, enabled, self)
    }
}

impl<E: IntoElement> Trackable for E {}
