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
//! already uses and that `cargo test -p ui context_menu` passes with. All
//! tests below run as normal (not `#[ignore]`) `cargo test -p ui_gallery`
//! tests, cross-platform.
//!
//! Real-bounds clicking: three of the gallery-wiring bugs below assert their
//! `on_click`/`on_change` wiring via a genuine `simulate_click` at the
//! control's real rendered pixel position, obtained via
//! `VisualTestContext::debug_bounds`. That required adding `debug_selector`
//! calls to `SegmentedControl` (`crates/ui/src/components/segmented_control.rs`)
//! and to `ActionPanel`'s Save/Cancel buttons
//! (`crates/ui/src/components/action_panel.rs`) — both `#[cfg(any(test,
//! feature = "test-support"))]`-gated (no-op in release builds), mirroring the
//! pre-existing precedent in `Tab` (`crates/ui/src/components/tab.rs`) and
//! `ContextMenu`. `Tab` already ships its own `debug_selector`, so the
//! `TabBar`/`nav_tab` test below needed no `ui` crate changes at all.

use gpui::{
    Entity, Focusable, Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase,
    VisualTestContext, point, px, size,
};

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
#[gpui::test]
fn scroll_offset_moves_on_wheel_event(cx: &mut TestAppContext) {
    // A short window (vs. `open_gallery`'s default) so the Forms page's
    // long list of fields overflows the content area and there is
    // something to actually scroll.
    let (gallery, cx) = open_gallery_sized(cx, size(px(1400.), px(150.)));

    gallery.update(cx, |app, cx| {
        app.page = GalleryPage::Forms;
        cx.notify();
    });
    cx.run_until_parked();
    // `ScrollHandle`'s `max_offset`/`child_bounds` bookkeeping is only fully
    // caught up one paint *after* the content that changes it first renders
    // (it's recorded from the previous frame's measured child bounds), so a
    // second no-op redraw is needed before scrolling will see real overflow.
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();

    let offset_before = gallery.read_with(cx, |app, _| app.scroll.offset());
    let max_before = gallery.read_with(cx, |app, _| app.scroll.max_offset());
    eprintln!("DEBUG max_offset before = {:?}", max_before);
    cx.update(|window, _| eprintln!("DEBUG viewport = {:?}", window.viewport_size()));

    cx.simulate_event(ScrollWheelEvent {
        position: point(px(640.), px(100.)),
        delta: ScrollDelta::Pixels(point(px(0.), px(-200.))),
        modifiers: Modifiers::default(),
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();
    cx.update(|window, _| eprintln!("DEBUG mouse_position after = {:?}", window.mouse_position()));

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
/// `Tab::on_click`. Navigates to the Navigation page, locates the underline
/// bar's third real `Tab` via `debug_bounds` (`Tab` already ships its own
/// `debug_selector` — see `crates/ui/src/components/tab.rs` — so this test
/// needed no `ui` crate changes), and drives a genuine `simulate_click`.
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
        .debug_bounds("TAB-nav-tab-underline-2")
        .expect("Navigation page's underline TabBar's third Tab should have rendered bounds");

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
