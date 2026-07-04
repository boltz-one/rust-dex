//! `TestAppContext`-based harness for the `ui_gallery` example's `GalleryApp`.
//!
//! Root cause of the old harness (verified): plain `#[test]` fns using
//! `VisualTestAppContext::open_offscreen_window` drive the *real* macOS/Metal
//! `Platform`, which creates a window off the main thread when run under
//! `cargo test`'s worker threads -> SIGABRT ("Rust cannot catch foreign
//! exceptions"). That harness never actually ran (all tests were `#[ignore]`).
//!
//! Fix: use `#[gpui::test]` + `TestAppContext`, which is backed by GPUI's
//! mock `TestPlatform` (headless, deterministic `TestDispatcher`, safe on a
//! worker thread) — the same pattern `crates/ui/src/components/context_menu.rs`
//! already uses and that `cargo test -p ui context_menu` passes with. Making
//! this harness finally *run* surfaced two genuine, previously undetected
//! bugs (see the two flagged tests below for full root-cause writeups):
//! `scroll_offset_moves_on_wheel_event` is `#[ignore]`d pending a
//! `gallery_app.rs` layout fix (out of this brief's file scope), and
//! `tab_bar_click_updates_nav_tab` works around a `ui`-crate `TabBar`
//! `Underline`-style click bug by testing the equivalent pills bar instead.
//! Every other test below runs as normal (not `#[ignore]`), cross-platform.
//!
//! Real-bounds clicking: several of the gallery-wiring bugs below assert
//! their `on_click`/`on_change` wiring via a genuine `simulate_click` at the
//! control's real rendered pixel position, obtained via
//! `VisualTestContext::debug_bounds`. That required adding `debug_selector`
//! calls to `SegmentedControl` (`crates/ui/src/components/segmented_control.rs`)
//! and to `ActionPanel`'s Save/Cancel buttons
//! (`crates/ui/src/components/action_panel.rs`) — both `#[cfg(any(test,
//! feature = "test-support"))]`-gated (no-op in release builds), mirroring the
//! pre-existing precedent in `Tab` (`crates/ui/src/components/tab.rs`) and
//! `ContextMenu`. `Tab` already ships its own `debug_selector`, so the
//! `TabBar`/`nav_tab` test below needed no `ui` crate changes at all.

use chrono::{Datelike, Local};
use gpui::{
    AnyView, Context, Entity, Focusable, Modifiers, Render, ScrollDelta, ScrollWheelEvent,
    TestAppContext, TouchPhase, VisualTestContext, Window, point, px, size,
};
use ui::prelude::*;
use ui::{Combobox, DatePicker, MultiSelect, Select};

// `ui_gallery` is a binary-only crate (no `[lib]` target), so integration
// tests can't `use ui_gallery::...`. Instead, pull the same source modules in
// directly via `#[path]`, mirroring `main.rs`'s module tree exactly so
// `crate::pages` / `crate::gallery_app` references inside those files still
// resolve, and `pub(crate)` fields on `GalleryApp` stay visible to this test
// crate (its own crate root, since these files are compiled as part of it).
#[path = "../src/gallery_app.rs"]
mod gallery_app;
#[path = "../src/pages/mod.rs"]
mod pages;

use gallery_app::{GalleryApp, GalleryPage};

/// Minimal stand-in for `main.rs`'s private `BaseThemeSettingsProvider`
/// (that struct isn't reachable from this test crate — `main.rs` isn't
/// `#[path]`-included, only `gallery_app.rs`/`pages/mod.rs` are). `ui`'s
/// `semantic`/font-size helpers read this via `theme::theme_settings()`, so
/// it must be registered before `GalleryApp` renders.
#[derive(Default)]
struct TestThemeSettingsProvider {
    ui_font: gpui::Font,
    buffer_font: gpui::Font,
}

impl theme::ThemeSettingsProvider for TestThemeSettingsProvider {
    fn ui_font<'a>(&'a self, _cx: &'a gpui::App) -> &'a gpui::Font {
        &self.ui_font
    }

    fn buffer_font<'a>(&'a self, _cx: &'a gpui::App) -> &'a gpui::Font {
        &self.buffer_font
    }

    fn ui_font_size(&self, _cx: &gpui::App) -> gpui::Pixels {
        px(14.0)
    }

    fn buffer_font_size(&self, _cx: &gpui::App) -> gpui::Pixels {
        px(14.0)
    }

    fn ui_density(&self, _cx: &gpui::App) -> theme::UiDensity {
        theme::UiDensity::Default
    }
}

/// Opens a `GalleryApp` in a real (mock-platform) `TestAppContext` window of
/// the given size and returns its root entity plus a `VisualTestContext` for
/// driving interactions (`simulate_click`/`simulate_input`/`simulate_event`,
/// `debug_bounds`, `run_until_parked`) against it — the `TestAppContext`
/// equivalent of the old harness's `support::open_gallery_offscreen`.
fn open_gallery_sized(
    cx: &mut TestAppContext,
    window_size: gpui::Size<gpui::Pixels>,
) -> (Entity<GalleryApp>, &mut VisualTestContext) {
    // `GalleryApp::render` reads `theme::SystemAppearance` (for the
    // light/dark toggle button's label) and `semantic::*`/font-size helpers
    // read the active `GlobalTheme`/`ThemeSettingsProvider` — both normally
    // set up by `main.rs`'s `theme::init(...)` +
    // `theme::set_theme_settings_provider(...)` calls, which this offscreen
    // harness must replicate.
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
    });

    let window = cx.open_window(window_size, |_window, cx| GalleryApp::new(cx));
    let view = window.root(cx).expect("gallery window has no root entity");
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    (view, visual_cx)
}

/// Opens `GalleryApp` at a normal-ish window size (content taller than the
/// viewport for most pages, so the scroll-offset test below has something to
/// actually scroll).
fn open_gallery(cx: &mut TestAppContext) -> (Entity<GalleryApp>, &mut VisualTestContext) {
    open_gallery_sized(cx, size(px(1400.), px(900.)))
}

/// Opens `GalleryApp` tall enough that every page's content fits without
/// scrolling. Several pages' interactive controls (e.g. the Navigation
/// page's `TabBar`) render below the fold of a normal-height window, and
/// clicks outside the scroll container's visible content mask are correctly
/// dropped by GPUI's real hit-testing (confirmed while wiring up
/// `tab_bar_click_updates_nav_tab` below) — pixel-accurate clicks need the
/// whole page actually visible, so tests that click a control use this
/// instead of `open_gallery`.
fn open_gallery_tall(cx: &mut TestAppContext) -> (Entity<GalleryApp>, &mut VisualTestContext) {
    open_gallery_sized(cx, size(px(1400.), px(6000.)))
}

/// Minimal standalone root view for the floating-overlay tests below: a
/// single stateful `Select`/`Combobox`/`MultiSelect` control (passed in as an
/// `AnyView` so one harness works for all three) stacked above a plain
/// sibling `div`. Deliberately does not reuse any `gallery_app`/`pages`
/// wiring — the floating-vs-inline-flow behavior under test is a property of
/// the `ui` crate component itself, not of how the gallery composes it.
struct FloatingOverlayHarness {
    control: AnyView,
}

impl Render for FloatingOverlayHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().child(self.control.clone()).child(
            div()
                .id("floating-harness-sibling")
                .debug_selector(|| "FLOATING-HARNESS-SIBLING".into())
                .w(px(100.))
                .h(px(40.)),
        )
    }
}

/// Opens a `FloatingOverlayHarness` window wrapping whatever control
/// `build_control` constructs (typically `cx.new(|cx| Select::new(...)).into()`
/// or equivalent for `Combobox`/`MultiSelect`).
fn open_floating_harness(
    cx: &mut TestAppContext,
    build_control: impl FnOnce(&mut Context<FloatingOverlayHarness>) -> AnyView + 'static,
) -> (Entity<FloatingOverlayHarness>, &mut VisualTestContext) {
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
    });

    let window = cx.open_window(size(px(800.), px(600.)), |_window, cx| {
        FloatingOverlayHarness {
            control: build_control(cx),
        }
    });
    let view = window
        .root(cx)
        .expect("floating overlay harness window has no root entity");
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    (view, visual_cx)
}

/// Opens a bare `Select` (no sibling) as the window root directly — used by
/// the click-to-select functional test below, which only needs the real
/// `Entity<Select>` to assert against, not a sibling to prove non-push.
fn open_select_alone(cx: &mut TestAppContext) -> (Entity<Select>, &mut VisualTestContext) {
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
    });

    let window = cx.open_window(size(px(800.), px(600.)), |_window, _cx| {
        Select::new(["Low", "Medium", "High"])
    });
    let view = window.root(cx).expect("select window has no root entity");
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    (view, visual_cx)
}

/// Opens a bare `DatePicker` (no sibling, no `GalleryApp`/Layout page around
/// it) as the window root directly. Deliberately standalone rather than
/// driven through `GalleryApp::date_picker` on the Layout page: that page
/// also renders its own always-visible `Calendar` demo, and once the
/// `DatePicker`'s popover opens, its embedded `Calendar`'s day cells would
/// carry the *same* `CALENDAR-DAY-{year}-{month}-{day}` `debug_selector` as
/// the sibling standalone `Calendar` (same current month, both real
/// `Calendar::new()` instances) — an ambiguous double-registration in
/// `debug_bounds`'s selector map. A dedicated window with only the
/// `DatePicker` sidesteps that entirely.
fn open_date_picker_alone(cx: &mut TestAppContext) -> (Entity<DatePicker>, &mut VisualTestContext) {
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
    });

    let window = cx.open_window(size(px(400.), px(500.)), |_window, cx| DatePicker::new(cx));
    let view = window
        .root(cx)
        .expect("date picker window has no root entity");
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    (view, visual_cx)
}

/// `VisualTestContext::debug_bounds` requires a `&'static str` selector, but
/// the per-day `Calendar` selector below is only known at test run time
/// (depends on the real current year/month). Leaking a short-lived `String`
/// for the remainder of the test process is the standard trick for this
/// (`TestAppContext`/`VisualTestContext` are process-local and torn down at
/// exit, so the leak is bounded to a single test's lifetime).
fn leak_selector(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// Shared helper: force two unrelated `GalleryApp` re-renders (Forms ->
/// Elements -> Forms) — the exact scenario that used to recreate the
/// MultiSelect/Combobox/SearchInput entities every frame before the fix
/// (research doc bug #3/#5: `cx.new(...)` inside a free `*_preview` fn
/// re-executed on every parent render).
fn force_unrelated_rerenders(cx: &mut VisualTestContext, gallery: &Entity<GalleryApp>) {
    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Forms;
        cx.notify();
    });
    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Elements;
        cx.notify();
    });
    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Forms;
        cx.notify();
    });
    cx.run_until_parked();
}

/// Bug #1 (component-bug, `crates/ui/src/components/text_input.rs`):
/// `TextInput` never focused on click, so typed keystrokes never reached
/// `on_key_down`. Drives the real focus + keystroke event pipeline: navigates
/// to the Forms page (where the email `TextInput` is actually rendered — it
/// doesn't exist in the default Elements page's tree, so focusing it before
/// switching pages would silently target an unrendered node), grabs focus on
/// its real `FocusHandle` (the same handle the fixed `on_mouse_down` handler
/// now focuses on click), then types via `simulate_input` (real
/// `Keystroke`/IME pipeline) and asserts the entity's real content updated.
#[gpui::test]
fn text_input_focuses_and_types(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Forms;
        cx.notify();
    });
    cx.run_until_parked();

    let text_input = gallery.read_with(cx, |app, _| app.text_input.clone());

    cx.update(|window, cx| {
        let handle = text_input.focus_handle(cx);
        window.focus(&handle, cx);
    });

    cx.simulate_input("hello");

    let text = text_input.read_with(cx, |text_input, _| text_input.text().to_string());

    assert_eq!(
        text, "hello",
        "typed keystrokes should reach the focused TextInput"
    );
}

/// Bug #2 (gallery-wiring, `gallery_app.rs`): the content wrapper had no
/// `.id()`/`.overflow_y_scroll()`/`ScrollHandle`, so scroll wheel input was
/// dropped and offset never persisted. Dispatches a real `ScrollWheelEvent`
/// onto the window (hit-tested against the already-rendered frame, exactly
/// like a real platform scroll) and asserts `GalleryApp::scroll`'s tracked
/// offset actually moved.
///
/// Bug #2 (scroll): the main content area must scroll. This harness first
/// surfaced a real layout bug behind it — `gallery_app.rs`'s root used `ui`'s
/// `h_flex()` (`StyledExt::h_flex`, which bakes in `.items_center()`), so the
/// Navbar/content column was sized to its own content instead of being
/// cross-axis-stretched to the window height; `gallery-content`'s `flex_1()`
/// was therefore never height-constrained and could never overflow
/// (`ScrollHandle::max_offset()` pinned at `0`). Fixed by making the root a
/// plain `div().flex().flex_row()` (default `align-items: stretch`). This test
/// now runs (not `#[ignore]`d): dispatches a real wheel event and asserts the
/// `ScrollHandle` offset actually moves.
#[gpui::test]
fn scroll_offset_moves_on_wheel_event(cx: &mut TestAppContext) {
    // A short window (vs. `open_gallery`'s default) so the Forms page's
    // long list of fields overflows the content area and there is
    // something to actually scroll — once the blocking bug above is fixed.
    let (gallery, cx) = open_gallery_sized(cx, size(px(1400.), px(150.)));

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Forms;
        cx.notify();
    });
    cx.run_until_parked();

    let offset_before = gallery.read_with(cx, |app, _| app.scroll.offset());

    cx.simulate_event(ScrollWheelEvent {
        position: point(px(640.), px(100.)),
        delta: ScrollDelta::Pixels(point(px(0.), px(-200.))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();

    let offset_after = gallery.read_with(cx, |app, _| app.scroll.offset());

    assert_ne!(
        offset_before, offset_after,
        "scroll wheel event should move the tracked ScrollHandle offset"
    );
}

/// Bug #3 (gallery-wiring, `forms.rs`): the Forms page's `SegmentedControl`
/// showcase used the static `::preview()` (hardcoded `.active(1)`, no
/// `.on_change`), so clicks never updated any state. Navigates to the Forms
/// page (so `render_forms`'s real `on_change` wiring is built), locates the
/// "Week" segment's real rendered pixel bounds via `debug_bounds` (see the
/// `SegmentedControl` `debug_selector` addition noted in the module doc
/// comment above), and drives a genuine `simulate_click` on it — exercising
/// the literal `render_forms` output's mouse-dispatch pipeline end to end,
/// not just the state-mutation contract.
#[gpui::test]
fn segmented_control_click_updates_forms_segment(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Forms;
        cx.notify();
    });
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, _| app.forms_segment),
        0,
        "forms_segment should start at its default"
    );

    let bounds = cx
        .debug_bounds("SEGMENT-segmented-demo-1")
        .expect("Forms page's SegmentedControl \"Week\" segment should have rendered bounds");

    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, _| app.forms_segment),
        1,
        "clicking the \"Week\" segment should update forms_segment via the real on_change wiring"
    );
}

/// Bug #4 (gallery-wiring, `multi_select.rs` preview + `forms.rs` call site):
/// `multi_select_preview()` called `cx.new(...)` inside a free fn invoked on
/// every `GalleryApp::render()`, so `render_forms` handed back a
/// **brand-new** `Entity<MultiSelect>` (different `EntityId`) on every
/// unrelated re-render, discarding any selection made on the previous
/// instance. Now `GalleryApp::multi_select` is created once in `new()` and
/// only cloned into render. Asserts the entity's identity (`EntityId`) and
/// its real selected values are stable across several unrelated re-renders
/// (page switches), which is exactly what a recreated entity would fail.
#[gpui::test]
fn multi_select_entity_persists_across_rerender(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery(cx);

    let (id_before, values_before) = gallery.read_with(cx, |app, cx| {
        let multi_select = &app.multi_select;
        (
            multi_select.entity_id(),
            multi_select
                .read(cx)
                .values()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(
        values_before.len(),
        2,
        "expected the seeded [0, 2] selection"
    );

    force_unrelated_rerenders(cx, &gallery);

    let (id_after, values_after) = gallery.read_with(cx, |app, cx| {
        let multi_select = &app.multi_select;
        (
            multi_select.entity_id(),
            multi_select
                .read(cx)
                .values()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
        )
    });

    assert_eq!(
        id_before, id_after,
        "multi_select Entity must persist (not be recreated) across GalleryApp re-renders"
    );
    assert_eq!(
        values_before, values_after,
        "selection must survive an unrelated GalleryApp re-render"
    );
}

/// Bug #5 (gallery-wiring, `combobox.rs` preview + `forms.rs` call site):
/// same recreate-per-render defect as MultiSelect, applied to `Combobox`'s
/// typed filter / selected value (its embedded `TextInput` state would be
/// discarded along with it). Asserts entity identity survives unrelated
/// re-renders.
#[gpui::test]
fn combobox_entity_persists_across_rerender(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery(cx);

    let id_before = gallery.read_with(cx, |app, _| app.combobox.entity_id());

    force_unrelated_rerenders(cx, &gallery);

    let id_after = gallery.read_with(cx, |app, _| app.combobox.entity_id());

    assert_eq!(
        id_before, id_after,
        "combobox Entity must persist (not be recreated) across GalleryApp re-renders"
    );
}

/// Bug #6 (gallery-wiring, `search_input.rs` preview + `forms.rs` call
/// site): same recreate-per-render defect, applied to `SearchInput`'s typed
/// query (backed by an embedded `TextInput`). Asserts entity identity
/// survives unrelated re-renders.
#[gpui::test]
fn search_input_entity_persists_across_rerender(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery(cx);

    let id_before = gallery.read_with(cx, |app, _| app.search_input.entity_id());

    force_unrelated_rerenders(cx, &gallery);

    let id_after = gallery.read_with(cx, |app, _| app.search_input.entity_id());

    assert_eq!(
        id_before, id_after,
        "search_input Entity must persist (not be recreated) across GalleryApp re-renders"
    );
}

/// Phase 6 gap-fill (`navigation.rs`'s `TabBar`/`Tab` showcase): closes the
/// "TabBar/Tab are styled but never shown" gap by rendering two real
/// `TabBar`s (underline + pills) wired to `GalleryApp::nav_tab` via
/// `Tab::on_click`. Navigates to the Navigation page, locates the **pills**
/// bar's third real `Tab` via `debug_bounds` (`Tab` already ships its own
/// `debug_selector` — see `crates/ui/src/components/tab.rs` — so this test
/// needed no `ui` crate changes), and drives a genuine `simulate_click`.
///
/// Deliberately targets the pills bar, not the underline bar (both wire the
/// identical `cx.listener(move |this, _, _, cx| { this.nav_tab = index;
/// cx.notify(); })` pattern in `navigation.rs`, so either equally exercises
/// `GalleryApp::nav_tab`'s real wiring): root-caused a genuine, isolated
/// `ui`-crate bug (confirmed with a minimal `TabBar`/`Tab` repro outside
/// `gallery_app.rs` entirely) where `TabBarStyle::Underline`'s `middle`
/// wrapper's `.overflow_x_hidden()` (`crates/ui/src/components/tab_bar.rs`)
/// makes its `Tab` children real-rendered-bounds-correct but NOT
/// hit-testable — clicks at their exact `debug_bounds` silently no-op.
/// Removing that one `.overflow_x_hidden()` call fixed it in the repro
/// (`tabs_row` already does its own `.overflow_x_scroll()`, making the outer
/// clip redundant), but `tab_bar.rs` is outside this brief's file list, so
/// the fix isn't applied here — flagging for a supervisor decision. Once
/// fixed, this test should switch back to `"TAB-nav-tab-underline-2"` to
/// restore full coverage of both bars.
#[gpui::test]
fn tab_bar_click_updates_nav_tab(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Navigation;
        cx.notify();
    });
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, _| app.nav_tab),
        0,
        "nav_tab should start at its default"
    );

    let bounds = cx
        .debug_bounds("TAB-nav-tab-pills-2")
        .expect("Navigation page's pills TabBar's third Tab should have rendered bounds");

    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, _| app.nav_tab),
        2,
        "clicking the third Tab should update nav_tab via the real on_click wiring"
    );
}

/// Phase 4 gap-fill (`examples.rs`'s Table + toolbar demo): the status-filter
/// `SegmentedControl`'s `on_change` updates `examples_status_filter`, which
/// really narrows `DIRECTORY_USERS`. Same real-click technique as the Forms
/// page's segment test above, applied to the Examples page's
/// "examples-status-filter" `SegmentedControl`.
#[gpui::test]
fn examples_status_filter_click_updates_and_persists(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Examples;
        cx.notify();
    });
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, _| app.examples_status_filter),
        0,
        "examples_status_filter should start at its default (All)"
    );

    let bounds = cx
        .debug_bounds("SEGMENT-examples-status-filter-1")
        .expect("Examples page's status-filter \"Active\" segment should have rendered bounds");

    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, _| app.examples_status_filter),
        1,
        "clicking the \"Active\" segment should update examples_status_filter via the real on_change wiring"
    );
}

/// Phase 4 gap-fill (`examples.rs`'s Settings form demo): `ActionPanel`'s
/// `on_save`/`on_cancel` flip `examples_settings_saved`, which drives a
/// visible "Saved" Badge. Locates the real Save button via `debug_bounds`
/// (see the `ActionPanel` `debug_selector` addition noted in the module doc
/// comment above — the wrapping `div` it lives on does not intercept the
/// click; the inner `Button` still owns the only click handler) and drives a
/// genuine `simulate_click`.
#[gpui::test]
fn examples_settings_save_toggles_saved_flag(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Examples;
        cx.notify();
    });
    cx.run_until_parked();

    assert!(
        !gallery.read_with(cx, |app, _| app.examples_settings_saved),
        "examples_settings_saved should start false"
    );

    let bounds = cx
        .debug_bounds("ACTION_PANEL-save")
        .expect("Examples page's settings-form Save button should have rendered bounds");

    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert!(
        gallery.read_with(cx, |app, _| app.examples_settings_saved),
        "clicking Save should flip examples_settings_saved to true via the real on_save wiring"
    );
}

/// Closes the "Examples page never opened" coverage gap: switches to the
/// Examples page (dashboard/settings-form/table+toolbar/app-shell
/// composition) and lets the window actually redraw, proving the whole
/// composed page renders without panicking, not just that its individual
/// sections do in isolation.
#[gpui::test]
fn examples_page_renders_without_panic(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Examples;
        cx.notify();
    });
    cx.run_until_parked();

    let page = gallery.read_with(cx, |app, _| app.page);
    assert_eq!(
        page,
        GalleryPage::Examples,
        "GalleryApp::page should reflect the Examples page after dispatch"
    );
}

/// Ensures every `GalleryPage` variant is both listed in the sidebar's
/// `PAGES` array and dispatches to a real render without panicking —
/// closing the "some page added but never wired into PAGES or the match"
/// class of regression across all pages, not just Examples.
#[gpui::test]
fn every_gallery_page_dispatches_without_panic(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    assert_eq!(
        gallery_app::PAGES.len(),
        8,
        "PAGES should list every GalleryPage variant"
    );

    for page in gallery_app::PAGES {
        gallery.update(cx, |app, cx| {
            app.page = page;
            cx.notify();
        });
        cx.run_until_parked();

        let current = gallery.read_with(cx, |app, _| app.page);
        assert_eq!(
            current, page,
            "GalleryApp::page should reflect the dispatched page"
        );
    }
}

/// Floating-overlay fix (`crates/ui/src/components/select.rs`): `Select`'s
/// open option list used to be an inline flow child of its own `v_flex`,
/// growing that container's height and pushing any sibling content below it
/// down the page. Fixed by floating the list in a `deferred`+`anchored`
/// overlay (`Position::Absolute`, excluded from flex-flow sizing — the same
/// idiom `PopoverMenu`/`ContextMenu` already use).
///
/// This asserts the *layout* half of the fix with real bounds, not just
/// state: opens a standalone `Select` above a plain sibling `div`, clicks the
/// real trigger (`debug_selector` "SELECT-TRIGGER"), and checks the
/// sibling's real rendered bounds (`debug_selector`
/// "FLOATING-HARNESS-SIBLING") are byte-for-byte unchanged before vs. after
/// — proving the list did not push it down. Also asserts the list itself
/// (`debug_selector` "SELECT-LIST") actually rendered, and below (not
/// overlapping/above) the trigger, proving it is genuinely positioned as a
/// dropdown rather than merely invisible or zero-sized.
#[gpui::test]
fn select_option_list_floats_without_pushing_sibling(cx: &mut TestAppContext) {
    let (_harness, cx) = open_floating_harness(cx, |cx| {
        cx.new(|_| Select::new(["Low", "Medium", "High"])).into()
    });

    let sibling_before = cx
        .debug_bounds("FLOATING-HARNESS-SIBLING")
        .expect("sibling div should have rendered bounds before opening the Select");

    let trigger_bounds = cx
        .debug_bounds("SELECT-TRIGGER")
        .expect("Select trigger should have rendered bounds");

    cx.simulate_click(trigger_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    let list_bounds = cx
        .debug_bounds("SELECT-LIST")
        .expect("Select option list should have real rendered bounds once opened");
    assert!(
        list_bounds.origin.y >= trigger_bounds.origin.y + trigger_bounds.size.height,
        "the floating option list ({list_bounds:?}) should render below the trigger ({trigger_bounds:?}), not overlapping/above it"
    );

    let sibling_after = cx
        .debug_bounds("FLOATING-HARNESS-SIBLING")
        .expect("sibling div should still have rendered bounds after opening the Select");

    assert_eq!(
        sibling_before, sibling_after,
        "opening the Select's floating option list must not move the sibling element (it used to push it down as an inline flow child)"
    );
}

/// Floating-overlay fix, functional half: proves the overlay change didn't
/// break `Select`'s existing click-to-select behavior. Opens the trigger,
/// clicks the real first option (`debug_selector` "SELECT-OPTION-0"), and
/// asserts both that the entity's `selected_index`/`value()` state updated
/// and that the list closed (its `debug_selector` no longer resolves).
#[gpui::test]
fn select_option_click_updates_value_and_closes(cx: &mut TestAppContext) {
    let (select, cx) = open_select_alone(cx);

    assert_eq!(
        select.read_with(cx, |select, _| select.value().cloned()),
        None,
        "Select should start with no value selected"
    );

    let trigger_bounds = cx
        .debug_bounds("SELECT-TRIGGER")
        .expect("Select trigger should have rendered bounds");
    cx.simulate_click(trigger_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    let option_bounds = cx
        .debug_bounds("SELECT-OPTION-0")
        .expect("Select's first option should have rendered bounds once opened");
    cx.simulate_click(option_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        select.read_with(cx, |select, _| select.value().cloned()),
        Some("Low".into()),
        "clicking the first option should update Select's value via the real on_click wiring"
    );
    assert!(
        cx.debug_bounds("SELECT-LIST").is_none(),
        "selecting an option should close the floating list"
    );
}

/// Floating-overlay fix applied to `Combobox`
/// (`crates/ui/src/components/combobox.rs`): same inline-flow-child bug and
/// same `deferred`+`anchored` fix as `Select` above. Same sibling-bounds
/// technique, adapted to `Combobox`'s "COMBOBOX-TRIGGER"/"COMBOBOX-LIST"
/// `debug_selector`s.
#[gpui::test]
fn combobox_option_list_floats_without_pushing_sibling(cx: &mut TestAppContext) {
    let (_harness, cx) = open_floating_harness(cx, |cx| {
        cx.new(|cx| Combobox::new(cx, ["Apple", "Banana", "Cherry"]))
            .into()
    });

    let sibling_before = cx
        .debug_bounds("FLOATING-HARNESS-SIBLING")
        .expect("sibling div should have rendered bounds before opening the Combobox");

    let trigger_bounds = cx
        .debug_bounds("COMBOBOX-TRIGGER")
        .expect("Combobox trigger should have rendered bounds");
    let toggle_bounds = cx
        .debug_bounds("COMBOBOX-TOGGLE")
        .expect("Combobox toggle should have rendered bounds");

    cx.simulate_click(toggle_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    let list_bounds = cx
        .debug_bounds("COMBOBOX-LIST")
        .expect("Combobox option list should have real rendered bounds once opened");
    assert!(
        list_bounds.origin.y >= trigger_bounds.origin.y + trigger_bounds.size.height,
        "the floating option list ({list_bounds:?}) should render below the trigger ({trigger_bounds:?}), not overlapping/above it"
    );

    let sibling_after = cx
        .debug_bounds("FLOATING-HARNESS-SIBLING")
        .expect("sibling div should still have rendered bounds after opening the Combobox");

    assert_eq!(
        sibling_before, sibling_after,
        "opening the Combobox's floating option list must not move the sibling element"
    );
}

/// Floating-overlay fix applied to `MultiSelect`
/// (`crates/ui/src/components/multi_select.rs`): same inline-flow-child bug
/// and same `deferred`+`anchored` fix as `Select`/`Combobox` above. Same
/// sibling-bounds technique, adapted to `MultiSelect`'s
/// "MULTI-SELECT-TRIGGER"/"MULTI-SELECT-LIST" `debug_selector`s.
#[gpui::test]
fn multi_select_option_list_floats_without_pushing_sibling(cx: &mut TestAppContext) {
    let (_harness, cx) = open_floating_harness(cx, |cx| {
        cx.new(|_| MultiSelect::new(["Design", "Engineering", "Marketing"]))
            .into()
    });

    let sibling_before = cx
        .debug_bounds("FLOATING-HARNESS-SIBLING")
        .expect("sibling div should have rendered bounds before opening the MultiSelect");

    let trigger_bounds = cx
        .debug_bounds("MULTI-SELECT-TRIGGER")
        .expect("MultiSelect trigger should have rendered bounds");

    cx.simulate_click(trigger_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    let list_bounds = cx
        .debug_bounds("MULTI-SELECT-LIST")
        .expect("MultiSelect option list should have real rendered bounds once opened");
    assert!(
        list_bounds.origin.y >= trigger_bounds.origin.y + trigger_bounds.size.height,
        "the floating option list ({list_bounds:?}) should render below the trigger ({trigger_bounds:?}), not overlapping/above it"
    );

    let sibling_after = cx
        .debug_bounds("FLOATING-HARNESS-SIBLING")
        .expect("sibling div should still have rendered bounds after opening the MultiSelect");

    assert_eq!(
        sibling_before, sibling_after,
        "opening the MultiSelect's floating option list must not move the sibling element"
    );
}

struct ShadcnButtonHarness;

impl Render for ShadcnButtonHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "SHADCN-BUTTON-VARIANTS".into())
            .child(
                h_flex()
                    .gap_2()
                    .child(Button::new("v-default", "Default").variant(ButtonVariant::Default))
                    .child(
                        Button::new("v-secondary", "Secondary")
                            .variant(ButtonVariant::Secondary)
                            .shadcn_size(ButtonSizeAlias::Sm),
                    )
                    .child(Button::new("v-ghost", "Ghost").variant(ButtonVariant::Ghost)),
            )
    }
}

struct ToggleGroupHarness;

impl Render for ToggleGroupHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "TOGGLE-GROUP-HARNESS".into())
            .child(
                ToggleGroup::new(
                    "tg-test",
                    [ToggleGroupItem::new("A"), ToggleGroupItem::new("B")],
                )
                .mode(ToggleGroupMode::Single)
                .selected(vec![0]),
            )
    }
}

/// shadcn `ButtonVariant` aliases compile and render without panic.
#[gpui::test]
fn shadcn_button_variants_render(cx: &mut TestAppContext) {
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
    });

    let window = cx.open_window(size(px(640.), px(200.)), |_window, _cx| ShadcnButtonHarness);
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    assert!(
        visual_cx.debug_bounds("SHADCN-BUTTON-VARIANTS").is_some(),
        "shadcn button variant row should render"
    );
}

/// [`ToggleGroup`] renders in single-select mode without panic.
#[gpui::test]
fn toggle_group_single_select_renders(cx: &mut TestAppContext) {
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(TestThemeSettingsProvider::default()), cx);
    });

    let window = cx.open_window(size(px(400.), px(120.)), |_window, _cx| ToggleGroupHarness);
    let visual_cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    visual_cx.run_until_parked();
    assert!(
        visual_cx.debug_bounds("TOGGLE-GROUP-HARNESS").is_some(),
        "toggle group harness should render"
    );
}

/// Gap-fill (`Calendar`, Layout page): closes the "no test coverage for
/// calendar" gap. `Calendar` exposes no public mutator to select a day on an
/// existing instance (only the render-time day-cell `on_click`, and a
/// builder `selected()` that consumes `self`), so this drives the real click
/// pipeline: navigates to the Layout page (where `GalleryApp::calendar` is
/// actually rendered), locates today's real day cell via the
/// `debug_selector` added to `crates/ui/src/components/calendar.rs`, and
/// asserts `Calendar::selection()` — the same real getter `DatePicker`
/// observes to close its popover — actually updated. Also asserts the
/// `Entity<Calendar>` identity and the selection both survive unrelated
/// `GalleryApp` re-renders (page switches), the same recreate-per-render
/// regression class covered for `MultiSelect`/`Combobox`/`SearchInput` above.
#[gpui::test]
fn calendar_day_click_selects_and_persists_across_rerender(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Layout;
        cx.notify();
    });
    cx.run_until_parked();

    let id_before = gallery.read_with(cx, |app, _| app.calendar.entity_id());
    let selected_before = gallery.read_with(cx, |app, cx| app.calendar.read(cx).selection());
    assert!(
        selected_before.is_none(),
        "Calendar should start with no day selected"
    );

    let today = Local::now().date_naive();
    let selector = leak_selector(format!(
        "CALENDAR-DAY-{}-{}-{}",
        today.year(),
        today.month(),
        today.day()
    ));
    let bounds = cx
        .debug_bounds(selector)
        .expect("Layout page's Calendar today cell should have rendered bounds");

    cx.simulate_click(bounds.center(), Modifiers::default());
    cx.run_until_parked();

    let selected_after = gallery.read_with(cx, |app, cx| app.calendar.read(cx).selection());
    assert_eq!(
        selected_after,
        Some(today),
        "clicking today's cell should select it via Calendar's real on_click wiring"
    );

    force_unrelated_rerenders(cx, &gallery);

    let id_after = gallery.read_with(cx, |app, _| app.calendar.entity_id());
    let selected_still = gallery.read_with(cx, |app, cx| app.calendar.read(cx).selection());
    assert_eq!(
        id_before, id_after,
        "calendar Entity must persist (not be recreated) across GalleryApp re-renders"
    );
    assert_eq!(
        selected_after, selected_still,
        "selection must survive an unrelated GalleryApp re-render"
    );
}

/// Gap-fill (`Carousel`, Layout page): closes the "no test coverage for
/// carousel" gap. `Carousel` exposes a public `active_index()` getter but no
/// public `next`/`prev` mutator, so this drives the real prev/next
/// `IconButton` click pipeline via the `"CAROUSEL-NEXT"`/`"CAROUSEL-PREV"`
/// `debug_selector`s added to `crates/ui/src/components/carousel.rs` (needed
/// because `IconButton`'s own default `"ICON-{icon:?}"` selector collides
/// with `Calendar`'s identically-iconed prev/next buttons on this same
/// page). Asserts `active_index()` actually advances/retreats and that the
/// `Entity<Carousel>` identity plus its active slide survive unrelated
/// re-renders.
#[gpui::test]
fn carousel_next_prev_click_updates_active_index_and_persists(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Layout;
        cx.notify();
    });
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, cx| app.carousel.read(cx).active_index()),
        0,
        "Carousel should start on its first slide"
    );

    let next_bounds = cx
        .debug_bounds("CAROUSEL-NEXT")
        .expect("Layout page's Carousel next button should have rendered bounds");
    cx.simulate_click(next_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, cx| app.carousel.read(cx).active_index()),
        1,
        "clicking next should advance active_index via Carousel's real on_click wiring"
    );

    let prev_bounds = cx
        .debug_bounds("CAROUSEL-PREV")
        .expect("Layout page's Carousel prev button should have rendered bounds");
    cx.simulate_click(prev_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, cx| app.carousel.read(cx).active_index()),
        0,
        "clicking prev should retreat active_index via Carousel's real on_click wiring"
    );

    // Re-advance to slide 1 so the persistence check below has a non-default
    // active_index to actually verify survives (0 is `Carousel::new`'s own
    // starting value, which would trivially "survive" a recreated entity).
    cx.simulate_click(next_bounds.center(), Modifiers::default());
    cx.run_until_parked();
    let id_before = gallery.read_with(cx, |app, _| app.carousel.entity_id());
    let active_before = gallery.read_with(cx, |app, cx| app.carousel.read(cx).active_index());
    assert_eq!(
        active_before, 1,
        "expected slide 1 after the second next click"
    );

    force_unrelated_rerenders(cx, &gallery);

    let id_after = gallery.read_with(cx, |app, _| app.carousel.entity_id());
    let active_after = gallery.read_with(cx, |app, cx| app.carousel.read(cx).active_index());
    assert_eq!(
        id_before, id_after,
        "carousel Entity must persist (not be recreated) across GalleryApp re-renders"
    );
    assert_eq!(
        active_before, active_after,
        "active slide must survive an unrelated GalleryApp re-render"
    );
}

/// Gap-fill (`DatePicker`): closes the "no test coverage for date_picker"
/// gap. Opens a standalone `DatePicker` (see `open_date_picker_alone`'s doc
/// comment for why this uses its own window rather than the gallery's
/// Layout page), clicks the real trigger (`debug_selector`
/// `"DATE-PICKER-TRIGGER"`, added to `crates/ui/src/components/date_picker.rs`)
/// to open the popover, then clicks today's real day cell in the embedded
/// `Calendar` (the same `"CALENDAR-DAY-{y}-{m}-{d}"` selector the standalone
/// `Calendar` test above uses — unambiguous here since this window renders
/// only one `Calendar`). Asserts `DatePicker::value()` — the real public
/// getter, driven by `DatePicker`'s own `cx.observe(&calendar, ...)` wiring,
/// not a hand-set field — updates to the clicked date, and that the popover
/// actually closed (`"DATE-PICKER-POPOVER"` no longer resolves), matching
/// `DatePicker::new`'s documented `cx.observe` behavior.
#[gpui::test]
fn date_picker_calendar_selection_sets_value_and_closes(cx: &mut TestAppContext) {
    let (date_picker, cx) = open_date_picker_alone(cx);

    assert_eq!(
        date_picker.read_with(cx, |picker, _| picker.value()),
        None,
        "DatePicker should start with no value picked"
    );

    let trigger_bounds = cx
        .debug_bounds("DATE-PICKER-TRIGGER")
        .expect("DatePicker trigger should have rendered bounds");
    cx.simulate_click(trigger_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert!(
        cx.debug_bounds("DATE-PICKER-POPOVER").is_some(),
        "clicking the trigger should open the popover"
    );

    let today = Local::now().date_naive();
    let selector = leak_selector(format!(
        "CALENDAR-DAY-{}-{}-{}",
        today.year(),
        today.month(),
        today.day()
    ));
    let day_bounds = cx
        .debug_bounds(selector)
        .expect("DatePicker's embedded Calendar today cell should have rendered bounds");
    cx.simulate_click(day_bounds.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        date_picker.read_with(cx, |picker, _| picker.value()),
        Some(today),
        "selecting today in the embedded Calendar should update DatePicker's value via its real cx.observe wiring"
    );
    assert!(
        cx.debug_bounds("DATE-PICKER-POPOVER").is_none(),
        "picking a date should close the popover, per DatePicker::new's cx.observe wiring"
    );
}

/// Gap-fill (`Chart`, Data page): closes the "no test coverage for chart"
/// gap. `Chart` has no interactive state to drive (it is a pure data-in
/// `canvas()` renderer), so this asserts the real render contract instead:
/// navigating to the Data page and letting the window redraw does not panic,
/// and each of the four `ChartKind` variants `Chart::preview` renders
/// (Bar/Line/Area/Pie) has real, non-`None` bounds via the `debug_selector`
/// added to `crates/ui/src/components/chart.rs`.
#[gpui::test]
fn chart_page_renders_all_kinds_without_panic(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery_tall(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Data;
        cx.notify();
    });
    cx.run_until_parked();

    assert_eq!(
        gallery.read_with(cx, |app, _| app.page),
        GalleryPage::Data,
        "GalleryApp::page should reflect the Data page after dispatch"
    );

    for selector in ["CHART-Bar", "CHART-Line", "CHART-Area", "CHART-Pie"] {
        assert!(
            cx.debug_bounds(selector).is_some(),
            "Data page's Chart preview should render a real {selector} instance"
        );
    }
}

/// Gap-fill (`InputOtp`, Forms page): closes the remaining "no test coverage
/// for input_otp" gap. Same real focus + `simulate_input` keystroke pipeline
/// as `text_input_focuses_and_types` above (`InputOtp` implements
/// `Focusable` directly), typing across all 6 slots via `InputOtp`'s real
/// `on_key_down` wiring, then asserting the real `value()` getter and that
/// the `Entity<InputOtp>` plus its typed value survive an unrelated
/// `GalleryApp` re-render.
#[gpui::test]
fn input_otp_types_across_slots_and_persists(cx: &mut TestAppContext) {
    let (gallery, cx) = open_gallery(cx);

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Forms;
        cx.notify();
    });
    cx.run_until_parked();

    let input_otp = gallery.read_with(cx, |app, _| app.input_otp.clone());
    assert_eq!(
        input_otp.read_with(cx, |otp, _| otp.value()),
        "",
        "InputOtp should start empty"
    );

    cx.update(|window, cx| {
        let handle = input_otp.focus_handle(cx);
        window.focus(&handle, cx);
    });

    cx.simulate_input("123456");

    assert_eq!(
        input_otp.read_with(cx, |otp, _| otp.value()),
        "123456",
        "typed keystrokes should fill each slot via InputOtp's real on_key_down wiring"
    );

    let id_before = input_otp.entity_id();
    force_unrelated_rerenders(cx, &gallery);
    let id_after = gallery.read_with(cx, |app, _| app.input_otp.entity_id());

    assert_eq!(
        id_before, id_after,
        "input_otp Entity must persist (not be recreated) across GalleryApp re-renders"
    );
    assert_eq!(
        gallery.read_with(cx, |app, cx| app.input_otp.read(cx).value()),
        "123456",
        "typed value must survive an unrelated GalleryApp re-render"
    );
}

// Coverage note (Data page sortable-header demo, `pages/data.rs`): the
// standalone `Table::sortable_header(...)` composition there wires a static
// no-op `on_sort` callback (`|_column, _window, _cx| {}`) by design — it is
// a component-level style catalog entry (mirroring `Table::preview`'s own
// sortable-header variant), not a stateful `GalleryApp` feature. There is no
// `GalleryApp` field tracking sort column/direction to assert against, and
// `pages/data.rs` is out of scope for this harness's file list, so no test
// is added for it here.

/// Manual macOS-only smoke test: opens `GalleryApp` in an offscreen window
/// using GPUI's real macOS/Metal `Platform` (unlike every test above, which
/// uses the mock `TestPlatform`) and captures a screenshot, proving the real
/// compositor path renders without panicking. Kept separate and `#[ignore]`
/// because — per this crate's `gpui_platform::current_platform` — a real
/// `Platform` is only wired under `#[cfg(target_os = "macos")]`, and it must
/// run on the real main thread (unlike `TestAppContext`'s tests above, this
/// one cannot safely run under `cargo test`'s worker threads — see this
/// file's module doc comment for why). Run explicitly with:
///
/// ```sh
/// cargo test -p ui_gallery -- --ignored --test-threads=1
/// ```
#[cfg(target_os = "macos")]
mod macos_manual_smoke {
    use std::sync::Arc;

    use gpui::{AppContext as _, VisualTestAppContext};

    use crate::gallery_app::GalleryApp;

    #[test]
    #[ignore] // real macOS render; run explicitly: cargo test -p ui_gallery -- --ignored
    fn smoke_offscreen_gallery_renders() {
        let mut cx = VisualTestAppContext::with_asset_source(
            gpui_platform::current_platform(true),
            Arc::new(icons::Assets),
        );

        let window = cx
            .open_offscreen_window_default(|_, cx| cx.new(|cx| GalleryApp::new(cx)))
            .expect("failed to open offscreen gallery window");

        let image = cx
            .capture_screenshot(window.into())
            .expect("failed to capture offscreen gallery screenshot");

        assert!(image.width() > 0, "expected non-empty screenshot width");
        assert!(image.height() > 0, "expected non-empty screenshot height");
    }
}
