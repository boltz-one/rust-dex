//! Read-only element-list panel for [`crate::inspector::InspectorOverlay`].
//!
//! Renders every currently visible [`ElementSnapshot`][crate::registry::ElementSnapshot]
//! in the shared [`ElementRegistry`] as a simple scrolled-look column:
//! `id  [x, y, wxh]  enabled/disabled`. Sorted by `id` for a stable,
//! diff-friendly ordering across frames.

use gpui::{App, IntoElement, ParentElement as _, Styled as _, div, px, rgb, rgba};

use crate::registry::ElementRegistry;

const PANEL_WIDTH_PX: f32 = 280.0;

/// Render the side panel listing all tracked elements visible this frame.
/// Reads [`ElementRegistry`] directly — no state of its own.
pub fn render_panel(cx: &App) -> impl IntoElement {
    let mut entries: Vec<_> = cx
        .try_global::<ElementRegistry>()
        .map(|registry| registry.all_visible().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    entries.sort_by(|a, b| a.id.cmp(&b.id));

    let mut list = div().flex().flex_col().gap_1();
    for snap in &entries {
        let line = format!(
            "{}  [{:.0}, {:.0}, {:.0}x{:.0}]  {}",
            snap.id,
            f32::from(snap.bounds.origin.x),
            f32::from(snap.bounds.origin.y),
            f32::from(snap.bounds.size.width),
            f32::from(snap.bounds.size.height),
            if snap.enabled { "enabled" } else { "disabled" },
        );
        list = list.child(div().text_color(rgb(0xffffff)).text_xs().child(line));
    }

    div()
        .absolute()
        .top_0()
        .right_0()
        .h_full()
        .w(px(PANEL_WIDTH_PX))
        .bg(rgba(0x000000cc))
        .p_2()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_color(rgb(0xffffff))
                .text_sm()
                .child(format!("Inspector — {} tracked", entries.len())),
        )
        .child(list)
}
