use gpui::{AnyElement, App, Window};
use ui::prelude::*;

use super::section;

/// Static "Feedback" showcase (unchanged by this phase; Alert already covers
/// the sole Feedback-page deliverable in the current catalog).
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_8()
        .child(section("Alerts", Alert::preview(window, cx)))
        .into_any_element()
}
