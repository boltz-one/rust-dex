use gpui::AnyElement;
use smallvec::SmallVec;

use crate::prelude::*;

/// A segmented group of connected buttons: no gap between children, a
/// shared divider border between each pair, and rounding only on the
/// outer corners (first/last child).
///
/// Children keep their own [`ButtonStyle`]/content untouched — rounding is
/// achieved by clipping the group container (`overflow_hidden` + outer
/// `rounded_md`), not by mutating each child's own style API.
///
/// # Examples
///
/// ```
/// use ui::prelude::*;
/// use ui::ButtonGroup;
///
/// ButtonGroup::new()
///     .child(Button::new("left", "Left"))
///     .child(Button::new("right", "Right"));
/// ```
#[derive(IntoElement, RegisterComponent)]
pub struct ButtonGroup {
    children: SmallVec<[AnyElement; 4]>,
}

impl ButtonGroup {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ButtonGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let border_color = semantic::border(cx);
        let last_index = self.children.len().saturating_sub(1);

        h_flex()
            .items_stretch()
            .rounded_md()
            .overflow_hidden()
            .border_1()
            .border_color(border_color)
            .children(self.children.into_iter().enumerate().map(|(index, child)| {
                div()
                    .when(index > 0 && index <= last_index, |this| {
                        this.border_l_1().border_color(border_color)
                    })
                    .child(child)
            }))
    }
}

impl Component for ButtonGroup {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("A segmented group of connected buttons sharing borders and outer rounding.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(example_group_with_title(
                    "Button Group",
                    vec![
                        single_example(
                            "Default",
                            ButtonGroup::new()
                                .child(Button::new("bg_left", "Left"))
                                .child(Button::new("bg_center", "Center"))
                                .child(Button::new("bg_right", "Right"))
                                .into_any_element(),
                        ),
                        single_example(
                            "Primary",
                            ButtonGroup::new()
                                .child(Button::new("bg_p_left", "Day").primary())
                                .child(Button::new("bg_p_center", "Week").primary())
                                .child(Button::new("bg_p_right", "Month").primary())
                                .into_any_element(),
                        ),
                    ],
                ))
                .into_any_element(),
        )
    }
}
