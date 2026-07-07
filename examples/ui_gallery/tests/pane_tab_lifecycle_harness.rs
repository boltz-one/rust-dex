//! `#[gpui::test]` coverage for the `TabContent` lifecycle hooks
//! (`on_focus_in`/`on_focus_out`/`on_resize`/`on_close`) + `Pane::close_all_tabs`
//! added in the `boltz-ui` 0.2.9 upgrade (Phase 1 of the terminal-core port).
//!
//! Standalone (never through `GalleryApp`, which spawns a real PTY-backed
//! `TerminalView` whose reader thread aborts the test binary during teardown —
//! a pre-existing cleanup bug). No real PTY is involved here: the hooks are
//! exercised with a `RecordingTab` that just logs which hook fired.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Entity, Pixels, TestAppContext, VisualTestContext, Window, px, size,
};
use ui::prelude::*;
use ui::{Pane, PaneGroup, SplitDirection, TabContent};

/// Minimal `theme::ThemeSettingsProvider` so `semantic::*`/font helpers used by
/// `Pane`/`PaneGroup` render resolve (mirrors `pane_group_harness.rs`).
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
    fn ui_font_size(&self, _cx: &App) -> gpui::Pixels {
        px(14.0)
    }
    fn buffer_font_size(&self, _cx: &App) -> gpui::Pixels {
        px(14.0)
    }
    fn ui_density(&self, _cx: &App) -> theme::UiDensity {
        theme::UiDensity::Default
    }
}

type Log = Rc<RefCell<Vec<String>>>;

/// Tab that appends `"<title>:<hook>"` to a shared log every time a lifecycle
/// hook fires — lets tests assert exact hook ordering across tabs.
struct RecordingTab {
    title: SharedString,
    log: Log,
}

impl RecordingTab {
    fn boxed(title: impl Into<SharedString>, log: &Log) -> Box<dyn TabContent> {
        Box::new(RecordingTab {
            title: title.into(),
            log: log.clone(),
        })
    }
    fn record(&self, hook: &str) {
        self.log
            .borrow_mut()
            .push(format!("{}:{}", self.title, hook));
    }
}

impl TabContent for RecordingTab {
    fn render(&self, _focused: bool, _window: &mut Window, _cx: &mut App) -> AnyElement {
        div().size_full().into_any_element()
    }
    fn title(&self) -> SharedString {
        self.title.clone()
    }
    fn on_focus_in(&mut self, _cx: &mut App) {
        self.record("focus_in");
    }
    fn on_focus_out(&mut self, _cx: &mut App) {
        self.record("focus_out");
    }
    fn on_resize(&mut self, _bounds: Bounds<Pixels>, _cx: &mut App) {
        self.record("resize");
    }
    fn on_close(&mut self, _cx: &mut App) {
        self.record("close");
    }
}

fn init_theme(cx: &mut App) {
    theme::init(theme::LoadThemes::JustBase, cx);
    theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
}

/// Opens a real `1200x800` window rooting a single-pane `PaneGroup` whose one
/// pane is seeded with a `RecordingTab "A"`. Split-created panes are seeded
/// with a `RecordingTab "child"`. Returns the group + a `VisualTestContext`.
fn open_group<'a>(
    cx: &'a mut TestAppContext,
    log: &Log,
) -> (Entity<PaneGroup>, &'a mut VisualTestContext) {
    cx.update(init_theme);
    let log_a = log.clone();
    let log_child = log.clone();
    let window = cx.open_window(size(px(1200.), px(800.)), |_window, cx| {
        let pane = cx.new(|_| Pane::new().with_tab(RecordingTab::boxed("A", &log_a)));
        PaneGroup::new(cx, pane).with_pane_factory(move |_| {
            Pane::new().with_tab(RecordingTab::boxed("child", &log_child))
        })
    });
    let group = window.root(cx).expect("pane group window has no root");
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    (group, visual_cx)
}

fn drain(log: &Log) -> Vec<String> {
    log.borrow_mut().drain(..).collect()
}

/// Drains, keeping only focus hooks — filters out `on_resize` entries, which
/// fire on a timing-dependent render tick and are asserted separately.
fn drain_focus(log: &Log) -> Vec<String> {
    log.borrow_mut()
        .drain(..)
        .filter(|e| e.ends_with(":focus_in") || e.ends_with(":focus_out"))
        .collect()
}

/// Drains, keeping only `on_close` entries.
fn drain_close(log: &Log) -> Vec<String> {
    log.borrow_mut()
        .drain(..)
        .filter(|e| e.ends_with(":close"))
        .collect()
}

/// `activate` on a focused pane fires `on_focus_out` on the old active tab and
/// `on_focus_in` on the new one, in that order.
#[gpui::test]
fn activate_fires_focus_out_then_in(cx: &mut TestAppContext) {
    let log: Log = Default::default();
    let (group, cx) = open_group(cx, &log);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());
    pane.update(cx, |p, cx| {
        p.add_tab(RecordingTab::boxed("B", &log), cx); // B active now
    });
    drain(&log); // clear add-tab noise

    pane.update(cx, |p, cx| p.activate(0, cx)); // back to A
    assert_eq!(drain_focus(&log), vec!["B:focus_out", "A:focus_in"]);
}

/// `set_focused(false)` fires `on_focus_out` on the active tab; regaining focus
/// fires `on_focus_in`.
#[gpui::test]
fn set_focused_toggles_focus_hooks(cx: &mut TestAppContext) {
    let log: Log = Default::default();
    let (group, cx) = open_group(cx, &log);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());
    drain(&log);

    pane.update(cx, |p, cx| p.set_focused(false, cx));
    assert_eq!(drain_focus(&log), vec!["A:focus_out"]);

    pane.update(cx, |p, cx| p.set_focused(true, cx));
    assert_eq!(drain_focus(&log), vec!["A:focus_in"]);
}

/// `close_tab` fires `on_close` for the closed tab before it is removed.
#[gpui::test]
fn close_tab_fires_on_close(cx: &mut TestAppContext) {
    let log: Log = Default::default();
    let (group, cx) = open_group(cx, &log);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());
    pane.update(cx, |p, cx| {
        p.add_tab(RecordingTab::boxed("B", &log), cx);
    });
    drain(&log);

    pane.update(cx, |p, cx| p.close_tab(1, cx)); // close B
    assert_eq!(drain_close(&log), vec!["B:close"]);
}

/// Closing the ACTIVE tab hands focus to its replacement, which must receive
/// `on_focus_in` (regression: previously never fired — left the newly shown
/// terminal in a blurred state). Closing a NON-active tab must not.
#[gpui::test]
fn closing_active_tab_focuses_replacement(cx: &mut TestAppContext) {
    let log: Log = Default::default();
    let (group, cx) = open_group(cx, &log);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());
    pane.update(cx, |p, cx| {
        p.add_tab(RecordingTab::boxed("B", &log), cx); // A, B; B active
    });
    drain(&log);

    // Close active B -> A becomes active and must be focused.
    pane.update(cx, |p, cx| p.close_tab(1, cx));
    assert_eq!(drain_focus(&log), vec!["A:focus_in"]);

    // Re-add B (active), then close the NON-active A: no focus change.
    pane.update(cx, |p, cx| p.add_tab(RecordingTab::boxed("B", &log), cx));
    drain(&log);
    pane.update(cx, |p, cx| p.close_tab(0, cx)); // close A (inactive)
    assert_eq!(drain_focus(&log), Vec::<String>::new());
}

/// A tab that becomes active without any physical pane resize still receives
/// an initial `on_resize` (regression: resize was pane-scoped, so a second tab
/// never got sized until a window resize — PTY would start at the wrong size).
#[gpui::test]
fn newly_active_tab_gets_initial_resize(cx: &mut TestAppContext) {
    let log: Log = Default::default();
    let (group, cx) = open_group(cx, &log);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());

    // Let A get its initial resize.
    pane.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    assert!(log.borrow().iter().any(|e| e == "A:resize"));
    drain(&log);

    // Add B (becomes active) and render — B must get on_resize despite the
    // pane's own bounds being unchanged.
    pane.update(cx, |p, cx| p.add_tab(RecordingTab::boxed("B", &log), cx));
    cx.run_until_parked();
    assert!(
        log.borrow().iter().any(|e| e == "B:resize"),
        "newly active tab B must receive on_resize; got {:?}",
        log.borrow()
    );
}

/// Removing a WHOLE pane fires `on_close` for EVERY tab in it, including the
/// inactive ones — the specific guarantee Phase 3's no-orphan requirement
/// depends on.
#[gpui::test]
fn remove_pane_closes_all_tabs_including_inactive(cx: &mut TestAppContext) {
    let log: Log = Default::default();
    let (group, cx) = open_group(cx, &log);

    // Split → new active pane B (seeded "child"); add two more recording tabs.
    group.update(cx, |g, cx| g.split(SplitDirection::Right, cx));
    let b = group.read_with(cx, |g, _| g.active_pane().clone());
    b.update(cx, |p, cx| {
        p.add_tab(RecordingTab::boxed("B1", &log), cx);
        p.add_tab(RecordingTab::boxed("B2", &log), cx); // B2 active, B1 + child inactive
    });
    drain(&log);

    // Close the whole active pane B (A then regains focus — filtered out here;
    // this test asserts only the close guarantee).
    let _ = group.update(cx, |g, cx| g.close_active(cx));
    cx.run_until_parked();

    let mut got = drain_close(&log);
    got.sort();
    assert_eq!(
        got,
        vec![
            "B1:close".to_string(),
            "B2:close".to_string(),
            "child:close".to_string(),
        ],
        "every tab of the removed pane (active AND inactive) must get on_close"
    );
}

/// After the pane is laid out in a real window, the `canvas()` measurement
/// drives `on_resize` on the active tab with concrete (non-degenerate) bounds.
#[gpui::test]
fn layout_fires_on_resize_with_bounds(cx: &mut TestAppContext) {
    let log: Log = Default::default();
    let (group, cx) = open_group(cx, &log);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());

    // First paint recorded bounds via canvas; force one more render so the
    // pane reads them back and delivers on_resize.
    pane.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();

    assert!(
        log.borrow().iter().any(|e| e == "A:resize"),
        "active tab must receive on_resize once its content area is measured; got {:?}",
        log.borrow()
    );
}
