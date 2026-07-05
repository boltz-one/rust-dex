//! Shared visual chrome for modal-overlay components (`CommandPalette`,
//! `TabSwitcher`). Extracted after `TabSwitcher` became the second overlay
//! with this exact backdrop/panel styling (full-screen dim backdrop, dark
//! floating panel with the same border/shadow/radius) — see the plan's
//! phase-01 ADR §1 for why a full generic `Picker<T: PickerDelegate>` is
//! NOT extracted alongside this: `CommandPalette` (fuzzy query input) and
//! `TabSwitcher` (plain list, no query) still have genuinely different
//! *behavior* shapes, so genericizing that would either force an unused
//! query slot onto `TabSwitcher` or split the delegate trait into
//! query-capable/non-query-capable variants — more upfront complexity than
//! two ~150-line bespoke files justify with only two call sites. Only the
//! *visual* duplication (identical colors/shadow/radius) was worth
//! deduplicating; it's pure styling with zero behavioral risk to extract.
//!
//! `pub(crate)` — this is plumbing for this crate's own overlay components,
//! not a public API surface.

use gpui::{BoxShadow, black, point, rgb};

use crate::prelude::*;

/// Panel background (dark, fixed — overlays intentionally don't follow the
/// active theme, matching `CommandPalette`'s original convention).
pub(crate) const OVERLAY_PANEL_BG: u32 = 0x12161C;
/// Panel border color.
pub(crate) const OVERLAY_PANEL_BORDER: u32 = 0x2A313B;

/// The dim, full-viewport backdrop every overlay is centered in.
pub(crate) fn overlay_backdrop() -> Div {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(black().opacity(0.5))
}

/// The floating panel chrome (background/border/radius/shadow/clip) shared
/// by every overlay. Callers add their own width/max-height and children.
pub(crate) fn overlay_panel() -> Div {
    v_flex()
        .bg(rgb(OVERLAY_PANEL_BG))
        .border_1()
        .border_color(rgb(OVERLAY_PANEL_BORDER))
        .rounded(px(14.))
        .shadow(vec![BoxShadow {
            color: black().opacity(0.6),
            offset: point(px(0.), px(24.)),
            blur_radius: px(70.),
            spread_radius: px(0.),
        }])
        .overflow_hidden()
}
