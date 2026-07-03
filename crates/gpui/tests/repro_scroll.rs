use boltz_gpui::{
    div, prelude::*, px, size, Context, IntoElement, Render, ScrollDelta, ScrollHandle,
    ScrollWheelEvent, Styled, TestAppContext, TouchPhase, Window, Modifiers, point,
};

struct Repro {
    scroll: ScrollHandle,
}

impl Render for Repro {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("content")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(div().h(px(3000.)).w_full()),
            )
    }
}

#[boltz_gpui::test]
fn repro_scroll(cx: &mut TestAppContext) {
    let window = cx.open_window(size(px(400.), px(150.)), |_window, cx| Repro { scroll: ScrollHandle::new() });
    let view = window.root(cx).unwrap();
    let mut vcx = boltz_gpui::VisualTestContext::from_window(window.into(), cx).into_mut();
    vcx.run_until_parked();

    let max = view.read_with(vcx, |app, _| app.scroll.max_offset());
    eprintln!("REPRO max_offset = {:?}", max);
}
