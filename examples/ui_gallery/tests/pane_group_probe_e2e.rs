//! End-to-end UI-logic tests for `ui`'s split-tree layout, driven through the
//! `gpui_probe` in-process test driver (`TestHarness` + `.probe()` +
//! `find_by_test_id`) rather than raw `TestAppContext` assertions.
//!
//! Why probe-driven e2e (vs the state-level `pane_group_harness.rs`): this
//! renders a realistic root (a split/close toolbar above a live `PaneGroup`),
//! then drives it exactly as a user would — locate a control by test-id, click
//! it, and assert the resulting *rendered* tree (real painted bounds via
//! `gpui::canvas()`), including geometric layout (left/right vs top/bottom) and
//! element presence/absence. It exercises `PaneGroup::split`/`close_active`,
//! the recursive two-axis render, and pane content mounting together.
//!
//! Standalone (no `GalleryApp`): `GalleryApp` always spawns a real PTY
//! `TerminalView` whose reader thread aborts the binary on teardown (a
//! pre-existing `boltz-terminal` bug) — so this e2e builds its own root.
//!
//! Probe-caveat compliance: only *intrinsically sized* elements are probed
//! (toolbar buttons, fixed-size tab content) — never the `size_full`/flex pane
//! containers, whose layout the `track()` wrapper would distort.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyElement, App, Bounds, Context, Entity, InteractiveElement as _, Pixels, Render,
    StatefulInteractiveElement as _, Window,
};
use gpui_probe::{TestHarness, Trackable as _};
use ui::prelude::*;
use ui::{Pane, PaneGroup, SplitDirection, TabContent};

const TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Default)]
struct TestThemeSettingsProvider {
    ui_font: gpui::Font,
    buffer_font: gpui::Font,
}

impl theme::ThemeSettingsProvider for TestThemeSettingsProvider {
    fn ui_font<'a>(&'a self, _cx: &'a App) -> &'a gpui::Font {
        &self.ui_font
    }
    fn buffer_font<'a>(&'a self, _cx: &'a App) -> &'a gpui::Font {
        &self.buffer_font
    }
    fn ui_font_size(&self, _cx: &App) -> Pixels {
        px(14.0)
    }
    fn buffer_font_size(&self, _cx: &App) -> Pixels {
        px(14.0)
    }
    fn ui_density(&self, _cx: &App) -> theme::UiDensity {
        theme::UiDensity::Default
    }
}

/// Tab content that probes an intrinsically-sized inner box under
/// `content-{label}`, so the e2e driver can locate/assert each pane's content.
struct ProbedTab {
    label: SharedString,
}

impl ProbedTab {
    fn boxed(label: impl Into<SharedString>) -> Box<dyn TabContent> {
        Box::new(ProbedTab {
            label: label.into(),
        })
    }
}

impl TabContent for ProbedTab {
    fn render(&self, _focused: bool, _window: &mut Window, _cx: &mut App) -> AnyElement {
        let id = format!("content-{}", self.label);
        div()
            .w(px(240.))
            .h(px(160.))
            .child(Label::new(self.label.clone()))
            .probe(id)
            .into_any_element()
    }
    fn title(&self) -> SharedString {
        self.label.clone()
    }
}

/// Root view: a probed split/close toolbar above a live `PaneGroup`. Toolbar
/// clicks drive the group; the group's panes render `ProbedTab` content.
struct ProbeRoot {
    group: Entity<PaneGroup>,
}

impl ProbeRoot {
    fn button(
        &self,
        id: &'static str,
        dir: Option<SplitDirection>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let group = self.group.clone();
        div()
            .id(id)
            .w(px(90.))
            .h(px(28.))
            .bg(semantic::surface(cx))
            .on_click(cx.listener(move |_this, _ev, _window, cx| {
                group.update(cx, |g, gcx| match dir {
                    Some(dir) => g.split(dir, gcx),
                    None => {
                        let _ = g.close_active(gcx);
                    }
                });
            }))
            .probe(id)
            .into_any_element()
    }
}

impl Render for ProbeRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w(px(1000.))
            .h(px(720.))
            .child(
                h_flex()
                    .h(px(36.))
                    .gap_2()
                    .child(self.button("split-right", Some(SplitDirection::Right), cx))
                    .child(self.button("split-down", Some(SplitDirection::Down), cx))
                    .child(self.button("close", None, cx)),
            )
            .child(div().flex_1().min_h_0().child(self.group.clone()))
    }
}

/// Builds the harness with a single-pane group (initial content "A"); split
/// panes get auto-incrementing "P1", "P2", ... content labels.
fn open_harness() -> TestHarness {
    TestHarness::new(|_window, cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);

        let counter = Rc::new(Cell::new(0u32));
        let pane = cx.new(|_| Pane::new().with_tab(ProbedTab::boxed("A")));
        let group = cx.new(|gcx| {
            PaneGroup::new(gcx, pane).with_pane_factory(move |_| {
                let n = counter.get() + 1;
                counter.set(n);
                Pane::new().with_tab(ProbedTab::boxed(format!("P{n}")))
            })
        });
        ProbeRoot { group }
    })
}

fn node_bounds(harness: &TestHarness, id: &str) -> Bounds<Pixels> {
    harness
        .snapshot_tree()
        .roots
        .into_iter()
        .find(|n| n.id.as_ref() == id)
        .unwrap_or_else(|| panic!("no tracked element with id `{id}`"))
        .bounds
}

/// e2e: boot shows one pane; clicking "split-right" adds a second pane whose
/// content sits to the RIGHT of the first (real painted bounds).
#[test]
fn split_right_shows_second_pane_to_the_right() {
    let mut harness = open_harness();

    harness
        .find("content-A")
        .assert_visible(TIMEOUT)
        .expect("initial pane content A should be visible");

    harness
        .find("split-right")
        .click()
        .expect("split-right toolbar button should be clickable");

    harness
        .find("content-P1")
        .assert_visible(TIMEOUT)
        .expect("split-created pane content P1 should be visible");

    let a = node_bounds(&harness, "content-A");
    let p1 = node_bounds(&harness, "content-P1");
    assert!(
        a.origin.x < p1.origin.x,
        "P1 must render to the right of A (A.x={:?}, P1.x={:?})",
        a.origin.x,
        p1.origin.x
    );
}

/// e2e: "split-down" stacks the new pane BELOW the first.
#[test]
fn split_down_stacks_second_pane_below() {
    let mut harness = open_harness();
    harness.find("content-A").assert_visible(TIMEOUT).unwrap();

    harness.find("split-down").click().unwrap();
    harness.find("content-P1").assert_visible(TIMEOUT).unwrap();

    let a = node_bounds(&harness, "content-A");
    let p1 = node_bounds(&harness, "content-P1");
    assert!(
        a.origin.y < p1.origin.y,
        "P1 must render below A (A.y={:?}, P1.y={:?})",
        a.origin.y,
        p1.origin.y
    );
}

/// e2e: after a split, clicking "close" removes the active (new) pane — its
/// content disappears and the original remains.
#[test]
fn close_active_pane_removes_its_content() {
    let mut harness = open_harness();
    harness.find("split-right").click().unwrap();
    harness
        .find("content-P1")
        .assert_visible(TIMEOUT)
        .expect("P1 visible after split");

    harness
        .find("close")
        .click()
        .expect("close toolbar button should be clickable");

    harness
        .find("content-P1")
        .assert_not_present(TIMEOUT)
        .expect("closed pane content P1 should be gone");
    harness
        .find("content-A")
        .assert_visible(TIMEOUT)
        .expect("original pane A should remain");
}
