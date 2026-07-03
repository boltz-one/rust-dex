pub mod data;
pub mod elements;
pub mod feedback;
pub mod forms;
pub mod layout;
pub mod navigation;
pub mod overlays;

use gpui::AnyElement;
use ui::prelude::*;

/// Wraps a control with a small label above it (used by static field-style
/// showcases, e.g. the Forms page's hand-built entries).
pub(crate) fn field(label: &str, control: AnyElement) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Label::new(label.to_string()).size(LabelSize::Small))
        .child(control)
        .into_any_element()
}

/// Wraps a component's `preview()` output with a section title.
pub(crate) fn section(title: &str, body: Option<AnyElement>) -> AnyElement {
    v_flex()
        .gap_3()
        .child(Label::new(title.to_string()).size(LabelSize::Large))
        .children(body)
        .into_any_element()
}
