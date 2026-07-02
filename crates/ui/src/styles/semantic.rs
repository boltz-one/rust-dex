//! Theme-driven neutral color roles.
//!
//! Components read neutrals (surfaces, borders, text, hover backgrounds) from
//! here rather than hardcoding palette grays, so dark and light both work
//! automatically: [`theme::set_appearance`] swaps the active theme and every
//! role below follows `cx.theme().colors()`. Accents/status come from
//! [`crate::styles::palette`] directly (mode-agnostic).

use gpui::{App, Hsla};
use theme::ActiveTheme;

/// Base app background.
pub fn background(cx: &App) -> Hsla {
    cx.theme().colors().background
}

/// Primary surface for cards, panels, inputs.
pub fn surface(cx: &App) -> Hsla {
    cx.theme().colors().surface_background
}

/// Raised surface for popovers, dropdowns, modals.
pub fn elevated_surface(cx: &App) -> Hsla {
    cx.theme().colors().elevated_surface_background
}

/// Default border.
pub fn border(cx: &App) -> Hsla {
    cx.theme().colors().border
}

/// Muted/subtle border.
pub fn border_muted(cx: &App) -> Hsla {
    cx.theme().colors().border_variant
}

/// Border color for focused elements (also see `focus_ring`).
pub fn border_focused(cx: &App) -> Hsla {
    cx.theme().colors().border_focused
}

/// Primary text.
pub fn text(cx: &App) -> Hsla {
    cx.theme().colors().text
}

/// Secondary/muted text.
pub fn text_muted(cx: &App) -> Hsla {
    cx.theme().colors().text_muted
}

/// Placeholder text.
pub fn text_placeholder(cx: &App) -> Hsla {
    cx.theme().colors().text_placeholder
}

/// Hover background for interactive neutral elements.
pub fn hover_bg(cx: &App) -> Hsla {
    cx.theme().colors().element_hover
}

/// Active/pressed background for interactive neutral elements.
pub fn active_bg(cx: &App) -> Hsla {
    cx.theme().colors().element_active
}

/// Default icon color.
pub fn icon(cx: &App) -> Hsla {
    cx.theme().colors().icon
}

/// Muted icon color.
pub fn icon_muted(cx: &App) -> Hsla {
    cx.theme().colors().icon_muted
}
