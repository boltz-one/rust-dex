//! Appearance-aware neutral color roles.
//!
//! Components read neutrals (surfaces, borders, text, hover backgrounds) from
//! here rather than hardcoding palette grays, so dark and light both work
//! automatically. Each role resolves to a light or dark palette swatch based on
//! the active [`SystemAppearance`], so flipping the appearance re-themes the
//! whole UI. Only accents/status come from [`crate::styles::palette`] directly.

use gpui::{App, Hsla, white};
use theme::{Appearance, SystemAppearance};

use crate::styles::palette;

fn is_light(cx: &App) -> bool {
    SystemAppearance::global(cx).0 == Appearance::Light
}

fn pick(cx: &App, light: Hsla, dark: Hsla) -> Hsla {
    if is_light(cx) { light } else { dark }
}

/// Base app background.
pub fn background(cx: &App) -> Hsla {
    pick(cx, palette::neutral(100), palette::neutral(950))
}

/// Primary surface for cards, panels, inputs.
pub fn surface(cx: &App) -> Hsla {
    pick(cx, white(), palette::neutral(900))
}

/// Raised surface for popovers, dropdowns, modals.
pub fn elevated_surface(cx: &App) -> Hsla {
    pick(cx, white(), palette::neutral(800))
}

/// Default border.
pub fn border(cx: &App) -> Hsla {
    pick(cx, palette::neutral(200), palette::neutral(700))
}

/// Muted/subtle border.
pub fn border_muted(cx: &App) -> Hsla {
    pick(cx, palette::neutral(100), palette::neutral(800))
}

/// Border color for focused elements (also see `focus_ring`).
pub fn border_focused(_cx: &App) -> Hsla {
    palette::primary(500)
}

/// Primary text.
pub fn text(cx: &App) -> Hsla {
    pick(cx, palette::neutral(900), palette::neutral(100))
}

/// Secondary/muted text.
pub fn text_muted(cx: &App) -> Hsla {
    pick(cx, palette::neutral(500), palette::neutral(400))
}

/// Placeholder text.
pub fn text_placeholder(cx: &App) -> Hsla {
    pick(cx, palette::neutral(400), palette::neutral(500))
}

/// Hover background for interactive neutral elements.
pub fn hover_bg(cx: &App) -> Hsla {
    pick(cx, palette::neutral(100), palette::neutral(800))
}

/// Active/pressed background for interactive neutral elements.
pub fn active_bg(cx: &App) -> Hsla {
    pick(cx, palette::neutral(200), palette::neutral(700))
}

/// Default icon color.
pub fn icon(cx: &App) -> Hsla {
    pick(cx, palette::neutral(500), palette::neutral(400))
}

/// Muted icon color.
pub fn icon_muted(cx: &App) -> Hsla {
    pick(cx, palette::neutral(400), palette::neutral(500))
}
