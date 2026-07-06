//! Reusable [`PaneGroup`](crate::PaneGroup) actions and their default
//! keybindings — shared interaction logic that ships from `boltz-ui` itself
//! (per the "shared logic -> shared crate" directive) so consumers mount a
//! `PaneGroup` and get split/close/focus keyboard driving for free, with no
//! per-app wiring.
//!
//! Keychord *values* are safe to ship as defaults: GPUI keybindings are
//! overridable at the app layer, so a consumer wanting different keys
//! rebinds locally (`cx.bind_keys([..])` after this call) with no republish
//! required — only the default set lives upstream.

use gpui::{App, KeyBinding, actions};

actions!(
    ui,
    [
        /// Splits the active pane, inserting the new pane to its right.
        SplitRight,
        /// Splits the active pane, inserting the new pane below it.
        SplitDown,
        /// Splits the active pane, inserting the new pane to its left.
        SplitLeft,
        /// Splits the active pane, inserting the new pane above it.
        SplitUp,
        /// Closes the active pane (no-op if it is the only pane left).
        ClosePane,
        /// Moves the active pane to its left neighbor, if any.
        FocusLeft,
        /// Moves the active pane to its right neighbor, if any.
        FocusRight,
        /// Moves the active pane to its neighbor above, if any.
        FocusUp,
        /// Moves the active pane to its neighbor below, if any.
        FocusDown,
    ]
);

/// Installs `boltz-ui`'s default `PaneGroup` keybindings, scoped to the
/// `"PaneGroup"` key context that [`crate::PaneGroup`]'s `Render` impl sets
/// on its root element:
///
/// - `super-d` -> [`SplitRight`]
/// - `super-shift-d` -> [`SplitDown`]
/// - `super-w` -> [`ClosePane`]
/// - `super-alt-left`/`right`/`up`/`down` -> [`FocusLeft`]/[`FocusRight`]/[`FocusUp`]/[`FocusDown`]
///
/// `super` resolves to Cmd on macOS and the platform/Super key elsewhere.
/// Call once during app startup; a consumer wanting different keys can call
/// `cx.bind_keys([..])` afterwards to override any of these.
///
/// # Example
/// ```ignore
/// gpui::App::new().run(|cx| {
///     ui::register_pane_keybindings(cx);
///     // ...
/// });
/// ```
pub fn register_pane_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("super-d", SplitRight, Some("PaneGroup")),
        KeyBinding::new("super-shift-d", SplitDown, Some("PaneGroup")),
        KeyBinding::new("super-w", ClosePane, Some("PaneGroup")),
        KeyBinding::new("super-alt-left", FocusLeft, Some("PaneGroup")),
        KeyBinding::new("super-alt-right", FocusRight, Some("PaneGroup")),
        KeyBinding::new("super-alt-up", FocusUp, Some("PaneGroup")),
        KeyBinding::new("super-alt-down", FocusDown, Some("PaneGroup")),
    ]);
}
