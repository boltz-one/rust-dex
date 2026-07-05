//! Phase 03 end-to-end validation: locate a probed button by test-id, click it,
//! assert the resulting element becomes visible, and assert the element it
//! replaced is now absent. Runs headless on the TEST platform.

use std::time::Duration;

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use gpui_probe::{TestHarness, Trackable as _};

/// A button that, when clicked, swaps a "placeholder" element for a "result".
struct Smoke {
    clicked: bool,
}

impl gpui::Render for Smoke {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().flex().flex_col().items_start();

        root = root.child(
            div()
                .id("btn")
                .w(px(80.))
                .h(px(30.))
                .on_click(cx.listener(|this, _event, _window, cx| {
                    this.clicked = true;
                    cx.notify();
                }))
                .probe("btn"),
        );

        if self.clicked {
            root = root.child(div().w(px(120.)).h(px(20.)).probe("result"));
        } else {
            root = root.child(div().w(px(60.)).h(px(20.)).probe("placeholder"));
        }

        root
    }
}

#[test]
fn locate_click_assert_and_assert_not_present() {
    let mut harness = TestHarness::new(|_window, _cx| Smoke { clicked: false });
    let timeout = Duration::from_secs(2);

    // Before the click: placeholder is present, result is not.
    harness
        .find("placeholder")
        .assert_visible(timeout)
        .expect("placeholder should be visible initially");

    // Locate the button by test-id and click it.
    harness
        .find("btn")
        .click()
        .expect("button should be clickable");

    // After the click: result appears, placeholder is gone.
    harness
        .find("result")
        .assert_visible(timeout)
        .expect("result should be visible after click");
    harness
        .find("placeholder")
        .assert_not_present(timeout)
        .expect("placeholder should be gone after click");
}
