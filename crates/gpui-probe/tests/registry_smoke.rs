//! Phase 02 validation gate (highest-risk assumption): the `canvas()` sibling
//! wrapper reports bounds equal to the tracked element's bounds, and stale
//! (unmounted) entries stop resolving. Runs headless on the TEST platform.

use gpui::{
    AvailableSpace, Context, IntoElement, ParentElement as _, Render, Styled as _, TestAppContext,
    TestDispatcher, Window, div, point, px, size,
};
use gpui_probe::{ElementRegistry, Trackable as _};

fn test_cx() -> TestAppContext {
    TestAppContext::build(TestDispatcher::new(0), Some("gpui_probe_registry_smoke"))
}

/// The canvas-reported bounds of a probed 100x50 div drawn at a known origin
/// must equal that div's own bounds — the core assumption the whole crate rests
/// on. If this drifts, the `track()` wrapper's style chain is wrong.
#[test]
fn canvas_bounds_match_tracked_div() {
    let mut app = test_cx();
    let cx = app.add_empty_window();
    let origin = point(px(10.), px(20.));
    let space = size(
        AvailableSpace::Definite(px(500.)),
        AvailableSpace::Definite(px(500.)),
    );

    // Embed the probe as a flex item in a `items_start` row so the tracking
    // wrapper hugs the inner element (a block-level wrapper would otherwise fill
    // the containing block's width). Flex parents are the GPUI norm; see the
    // caveat in `track`'s docs about non-flex / stretched parents.
    cx.draw(origin, space, |_window, _cx| {
        div()
            .flex()
            .items_start()
            .child(div().w(px(100.)).h(px(50.)).probe("box"))
            .into_element()
    });

    let snap = cx
        .update(|_window, cx| cx.global::<ElementRegistry>().get("box"))
        .expect("tracked element should resolve in the current frame");

    assert_eq!(snap.bounds.size.width, px(100.), "width: {:?}", snap.bounds);
    assert_eq!(
        snap.bounds.size.height,
        px(50.),
        "height: {:?}",
        snap.bounds
    );
    assert_eq!(snap.bounds.origin, origin, "origin: {:?}", snap.bounds);
    assert!(snap.enabled);
}

struct Toggle {
    show: bool,
}

impl Render for Toggle {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div();
        if self.show {
            root = root.child(div().w(px(100.)).h(px(50.)).probe("box"));
        }
        root
    }
}

/// An element present one frame then removed the next must go stale: `get`
/// returns `None` after the frame counter advances and it is not re-painted.
#[test]
fn removed_element_becomes_stale() {
    let mut app = test_cx();
    let (view, cx) = app.add_window_view(|_window, _cx| Toggle { show: true });
    cx.run_until_parked();

    assert!(
        cx.update(|_window, cx| cx.global::<ElementRegistry>().get("box"))
            .is_some(),
        "present element should resolve"
    );

    // Advance a frame, hide the element, re-render.
    cx.update(|_window, cx| cx.default_global::<ElementRegistry>().begin_frame());
    view.update(cx, |v, cx| {
        v.show = false;
        cx.notify();
    });
    cx.run_until_parked();

    assert!(
        cx.update(|_window, cx| cx.global::<ElementRegistry>().get("box"))
            .is_none(),
        "removed element should be stale (None) after the frame advances"
    );
}
