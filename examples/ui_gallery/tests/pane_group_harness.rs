//! `#[gpui::test]` coverage for `ui`'s recursive split-tree layout system
//! (`PaneGroup`/`Pane`/`SplitDirection` + `pane_actions`), added in the
//! `boltz-ui` 0.2.0 upgrade.
//!
//! Deliberately standalone (builds `PaneGroup`/`Pane` directly via `ui::`,
//! never through `GalleryApp`): `GalleryApp` always constructs a real
//! PTY-backed `TerminalView`, whose reader thread aborts the whole test
//! binary during teardown (a pre-existing `boltz-terminal` cleanup bug,
//! unrelated to this layout work) — so these tests must not pull it in.
//!
//! Uses `#[gpui::test]` + `TestAppContext` (mock `TestPlatform`, headless,
//! worker-thread-safe), the same pattern as `visual_harness.rs`. Tree shape
//! is asserted behaviorally through the real public API (`split`/`focus`/
//! `close_active`/`active_pane`) — no mocks, real entity state.

use gpui::{
    AnyElement, App, Entity, Focusable, TestAppContext, VisualTestContext, Window, px, size,
};
use ui::prelude::*;
use ui::{CannotRemoveLastPane, Pane, PaneGroup, SplitDirection, SplitRight, TabContent};

/// Minimal `theme::ThemeSettingsProvider` so `semantic::*`/font helpers used
/// by `Pane`/`PaneGroup` render resolve (mirrors `visual_harness.rs`).
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

/// A trivial tab whose title is fixed at construction — enough to assert
/// reorder/activation results.
struct TestTab {
    title: SharedString,
}

impl TestTab {
    fn boxed(title: impl Into<SharedString>) -> Box<dyn TabContent> {
        Box::new(TestTab {
            title: title.into(),
        })
    }
}

impl TabContent for TestTab {
    fn render(&self, _focused: bool, _window: &mut Window, _cx: &mut App) -> AnyElement {
        div().child(self.title.clone()).into_any_element()
    }
    fn title(&self) -> SharedString {
        self.title.clone()
    }
}

fn init_theme(cx: &mut App) {
    theme::init(theme::LoadThemes::JustBase, cx);
    theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
}

/// Opens a window rooting a single-leaf `PaneGroup` (its one pane seeded with
/// a tab titled "A"); split-created panes are seeded with a "child" tab.
/// Returns the group and a `VisualTestContext`.
fn open_group(cx: &mut TestAppContext) -> (Entity<PaneGroup>, &mut VisualTestContext) {
    cx.update(init_theme);
    let window = cx.open_window(size(px(1200.), px(800.)), |_window, cx| {
        let pane = cx.new(|_| Pane::new().with_tab(TestTab::boxed("A")));
        PaneGroup::new(cx, pane)
            .with_pane_factory(|_| Pane::new().with_tab(TestTab::boxed("child")))
    });
    let group = window.root(cx).expect("pane group window has no root");
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    (group, visual_cx)
}

fn active_id(group: &Entity<PaneGroup>, cx: &mut VisualTestContext) -> gpui::EntityId {
    group.read_with(cx, |g, _| g.active_pane().entity_id())
}

/// Scenario 1: `split(Right)` makes the new pane active and inserts it as a
/// horizontal sibling *after* the original — `focus(Left)` returns to it.
#[gpui::test]
fn split_right_creates_horizontal_sibling(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    let a = active_id(&group, cx);

    group.update(cx, |g, cx| g.split(SplitDirection::Right, cx));
    let b = active_id(&group, cx);
    assert_ne!(a, b, "split must create and activate a new pane");

    group.update(cx, |g, cx| g.focus(SplitDirection::Left, cx));
    assert_eq!(active_id(&group, cx), a, "left neighbor of B must be A");

    group.update(cx, |g, cx| g.focus(SplitDirection::Right, cx));
    assert_eq!(active_id(&group, cx), b, "right neighbor of A must be B");
}

/// Scenario 2 (vertical axis / resize direction): `split(Down)` inserts a
/// vertical sibling below; `focus(Up)` returns to the original.
#[gpui::test]
fn split_down_creates_vertical_sibling(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    let a = active_id(&group, cx);

    group.update(cx, |g, cx| g.split(SplitDirection::Down, cx));
    let b = active_id(&group, cx);
    assert_ne!(a, b);

    group.update(cx, |g, cx| g.focus(SplitDirection::Up, cx));
    assert_eq!(active_id(&group, cx), a, "up neighbor of B must be A");
}

/// Scenario 7: splitting the same direction repeatedly appends siblings into
/// one axis (N-way) — three panes in a row, navigable end-to-end by focus.
#[gpui::test]
fn nway_split_appends_three_in_a_row(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    let a = active_id(&group, cx);
    group.update(cx, |g, cx| g.split(SplitDirection::Right, cx));
    let b = active_id(&group, cx);
    group.update(cx, |g, cx| g.split(SplitDirection::Right, cx));
    let c = active_id(&group, cx);
    assert!(a != b && b != c && a != c, "three distinct panes");

    // From C, walking left visits B then A (flat row, not nested pairs).
    group.update(cx, |g, cx| g.focus(SplitDirection::Left, cx));
    assert_eq!(active_id(&group, cx), b);
    group.update(cx, |g, cx| g.focus(SplitDirection::Left, cx));
    assert_eq!(active_id(&group, cx), a);
}

/// Scenario 4: reordering tabs changes their order while keeping the same tab
/// active.
#[gpui::test]
fn reorder_tabs_changes_order_preserves_active(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());
    pane.update(cx, |p, cx| {
        p.add_tab(TestTab::boxed("B"), cx);
        p.add_tab(TestTab::boxed("C"), cx);
        p.activate(0, cx); // activate "A"
    });

    pane.update(cx, |p, cx| p.reorder(0, 2, cx));
    let (titles, active) = pane.read_with(cx, |p, _| (p.titles(), p.active_index()));
    assert_eq!(
        titles,
        vec!["B".into(), "C".into(), "A".into()] as Vec<SharedString>
    );
    assert_eq!(titles[active], "A", "active tab 'A' follows the move");
}

/// Scenario 2 (tabs): the "+" path — `add_tab` grows the count and activates
/// the new tab; Scenario 3: `close_tab` removes it and reassigns active.
#[gpui::test]
fn add_and_close_tab_update_count_and_active(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    let pane = group.read_with(cx, |g, _| g.active_pane().clone());

    pane.update(cx, |p, cx| {
        p.add_tab(TestTab::boxed("B"), cx);
    });
    let (count, active) = pane.read_with(cx, |p, _| (p.tab_count(), p.active_index()));
    assert_eq!(count, 2);
    assert_eq!(active, 1, "newly added tab becomes active");

    let empty = pane.update(cx, |p, cx| p.close_tab(1, cx));
    assert!(!empty);
    let (count, active) = pane.read_with(cx, |p, _| (p.tab_count(), p.active_index()));
    assert_eq!(count, 1);
    assert_eq!(active, 0, "active index clamps back onto remaining tab");
}

/// Scenario 6: closing a pane's last tab removes the pane from the tree; the
/// remaining pane becomes active. And the sole remaining pane cannot be
/// closed (`CannotRemoveLastPane`).
#[gpui::test]
fn close_last_tab_removes_pane_and_last_pane_is_protected(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    let a = group.read_with(cx, |g, _| g.active_pane().clone());
    group.update(cx, |g, cx| g.split(SplitDirection::Right, cx));
    let b = group.read_with(cx, |g, _| g.active_pane().clone());
    assert_ne!(a.entity_id(), b.entity_id());

    // Close B's only tab -> B emits Empty -> PaneGroup removes B.
    b.update(cx, |p, cx| p.close_tab(0, cx));
    cx.run_until_parked();
    assert_eq!(
        active_id(&group, cx),
        a.entity_id(),
        "removing active pane B reassigns active to A"
    );

    // A is now the only pane: close_active must refuse.
    let result: Result<(), CannotRemoveLastPane> = group.update(cx, |g, cx| g.close_active(cx));
    assert_eq!(result, Err(CannotRemoveLastPane));
}

/// Scenario 1 (render smoke): the recursive two-axis render path (nested
/// `PaneAxis` + `ResizablePanel::axis`) produces a frame without panicking
/// for a mixed horizontal+vertical tree.
#[gpui::test]
fn nested_two_axis_tree_renders(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    group.update(cx, |g, cx| {
        g.split(SplitDirection::Right, cx);
        g.split(SplitDirection::Down, cx); // vertical split inside the right column
    });
    cx.run_until_parked();
    // Reaching here without a panic exercises h_flex/v_flex + both-axis
    // ResizablePanel; assert the active (factory-created "child") pane resolves
    // with its seeded tab.
    let active_tabs = group.read_with(cx, |g, cx| g.active_pane().read(cx).tab_count());
    assert_eq!(active_tabs, 1, "factory-created pane keeps its seeded tab");
}

/// Scenario 8: the shared interaction layer — dispatching the `SplitRight`
/// action (what `super-d` binds to) to the focused group splits the active
/// pane, with zero app-side wiring.
#[gpui::test]
fn split_right_action_splits_active_pane(cx: &mut TestAppContext) {
    let (group, cx) = open_group(cx);
    let a = active_id(&group, cx);

    cx.update(|window, cx| {
        let handle = group.focus_handle(cx);
        window.focus(&handle, cx);
    });
    cx.run_until_parked();

    cx.dispatch_action(SplitRight);
    cx.run_until_parked();

    let b = active_id(&group, cx);
    assert_ne!(
        a, b,
        "SplitRight action must create and activate a new pane"
    );
    group.update(cx, |g, cx| g.focus(SplitDirection::Left, cx));
    assert_eq!(active_id(&group, cx), a, "new pane sits to the right of A");
}
