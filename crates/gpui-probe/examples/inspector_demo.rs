//! Toy window for manually exercising [`InspectorOverlay`] against a handful
//! of `.probe()`-ed elements of different sizes/positions.
//!
//! Runs on GPUI's headless TEST platform (via `TestAppContext`/
//! `TestDispatcher` — the same backend `gpui-probe`'s own smoke tests use, see
//! `tests/registry_smoke.rs` and `src/driver/test_platform.rs`) rather than a
//! real OS window: opening a *real* window requires an OS platform crate
//! (`gpui_platform` + `gpui_macos`/`gpui_linux`/...), which is not a
//! dependency of this crate and is out of scope to add here. The headless
//! platform still drives real layout/paint/`ElementRegistry` updates, so it
//! demonstrates the overlay's hover/panel logic end to end without a display.
//!
//! Run with:
//! ```sh
//! cargo run -p gpui-probe --example inspector_demo --features inspector
//! ```

use gpui::{
    AppContext as _, Context, IntoElement, Modifiers, ParentElement as _, Render, Styled as _,
    TestAppContext, TestDispatcher, Window, div, point, px,
};
use gpui_probe::{ElementRegistry, InspectorOverlay, Trackable as _};

/// Root view: a row of differently sized/positioned probed elements with the
/// [`InspectorOverlay`] mounted as an absolutely-positioned sibling on top.
struct DemoRoot {
    overlay: gpui::Entity<InspectorOverlay>,
}

impl Render for DemoRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .bg(gpui::rgb(0x1e1e1e))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_4()
                    .p_4()
                    .child(
                        div()
                            .w(px(120.0))
                            .h(px(60.0))
                            .bg(gpui::rgb(0x3478f6))
                            .probe("button-a"),
                    )
                    .child(
                        div()
                            .w(px(80.0))
                            .h(px(80.0))
                            .bg(gpui::rgb(0x34c759))
                            .probe("card-b"),
                    )
                    .child(
                        div()
                            .w(px(200.0))
                            .h(px(40.0))
                            .bg(gpui::rgb(0xff9f0a))
                            .probe("banner-c"),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .child(self.overlay.clone()),
            )
    }
}

fn main() {
    let mut cx = TestAppContext::build(TestDispatcher::new(0), Some("gpui_probe_inspector_demo"));
    let (_root, cx) = cx.add_window_view(|_window, cx| {
        let overlay = cx.new(|_cx| InspectorOverlay::new());
        DemoRoot { overlay }
    });
    cx.run_until_parked();

    // Move the simulated cursor over "button-a" and re-render so the
    // overlay's hover-highlight resolves against real ElementRegistry data.
    cx.simulate_mouse_move(point(px(60.0), px(30.0)), None, Modifiers::default());
    cx.run_until_parked();

    let tree = cx.update(|_window, cx| cx.global::<ElementRegistry>().snapshot_tree());
    println!(
        "gpui-probe inspector demo — {} tracked element(s):",
        tree.roots.len()
    );
    for node in &tree.roots {
        println!(
            "  {}  [{:.0}, {:.0}, {:.0}x{:.0}]  {}",
            node.id,
            f32::from(node.bounds.origin.x),
            f32::from(node.bounds.origin.y),
            f32::from(node.bounds.size.width),
            f32::from(node.bounds.size.height),
            if node.enabled { "enabled" } else { "disabled" },
        );
    }
}
