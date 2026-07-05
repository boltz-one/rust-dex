//! Phase 04: tree-text snapshots are deterministic (byte-identical across runs),
//! sorted by id, and float-free by default; plus one end-to-end `insta` snapshot.

use gpui::{Context, IntoElement, ParentElement as _, Styled as _, Window, div, px};
use gpui_probe::{SnapshotRedactions, TestHarness, Trackable as _};

struct View;

impl gpui::Render for View {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Intentionally probed out of alphabetical order to prove sorting.
        div()
            .flex()
            .flex_col()
            .items_start()
            .child(div().w(px(40.)).h(px(10.)).probe("zeta"))
            .child(div().w(px(40.)).h(px(10.)).probe("alpha"))
    }
}

#[test]
fn tree_text_deterministic_sorted_no_floats() {
    let h1 = TestHarness::new(|_window, _cx| View);
    let h2 = TestHarness::new(|_window, _cx| View);

    let a = gpui_probe::tree_text(&h1.snapshot_tree(), &SnapshotRedactions::default());
    let b = gpui_probe::tree_text(&h2.snapshot_tree(), &SnapshotRedactions::default());

    assert_eq!(a, b, "snapshot must be byte-identical across runs");
    assert!(!a.is_empty(), "expected tracked entries, got: {a:?}");
    assert!(!a.contains('.'), "no raw floats by default: {a:?}");

    let alpha = a.find("alpha").expect("alpha present");
    let zeta = a.find("zeta").expect("zeta present");
    assert!(alpha < zeta, "entries must be sorted by id:\n{a}");
}

#[test]
fn tree_text_insta_snapshot() {
    let h = TestHarness::new(|_window, _cx| View);
    let text = gpui_probe::tree_text(&h.snapshot_tree(), &SnapshotRedactions::default());
    insta::assert_snapshot!("smoke_tree", text);
}
