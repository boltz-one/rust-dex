use gpui::{AnyElement, App, Window};
use ui::prelude::*;
use ui::{Banner, Callout, ProgressBar};

use super::section;

/// Feedback components: alerts, progress, callouts.
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_8()
        .child(section("Alerts", Alert::preview(window, cx)))
        .child(section("Callout", Callout::preview(window, cx)))
        .child(section("Banner", Banner::preview(window, cx)))
        .child(section("Progress", ProgressBar::preview(window, cx)))
        .into_any_element()
}
