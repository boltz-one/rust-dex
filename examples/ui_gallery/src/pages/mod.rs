pub mod data;
pub mod elements;
pub mod examples;
pub mod feedback;
pub mod forms;
pub mod layout;
pub mod navigation;
pub mod overlays;

use gpui::AnyElement;
use ui::prelude::*;

/// Wraps a component's `preview()` output with a section title.
pub(crate) fn section(title: &str, body: Option<AnyElement>) -> AnyElement {
    v_flex()
        .gap_3()
        .child(Label::new(title.to_string()).size(LabelSize::Large))
        .children(body)
        .into_any_element()
}
