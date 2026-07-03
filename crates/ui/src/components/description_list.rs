use gpui::AnyElement;

use crate::prelude::*;

/// Layout mode of a [`DescriptionList`] row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DescriptionListMode {
    /// Label above value (mobile-style).
    #[default]
    Stacked,
    /// Label left, value right.
    Horizontal,
}

/// A key-value list for displaying structured details (Tailwind "Description List").
#[derive(IntoElement, RegisterComponent)]
pub struct DescriptionList {
    mode: DescriptionListMode,
    items: Vec<(SharedString, AnyElement)>,
}

impl DescriptionList {
    pub fn new() -> Self {
        Self {
            mode: DescriptionListMode::default(),
            items: Vec::new(),
        }
    }

    pub fn mode(mut self, mode: DescriptionListMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn item(mut self, label: impl Into<SharedString>, value: impl IntoElement) -> Self {
        self.items.push((label.into(), value.into_any_element()));
        self
    }
}

impl Default for DescriptionList {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for DescriptionList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mode = self.mode;

        v_flex()
            .w_full()
            .children(
                self.items
                    .into_iter()
                    .enumerate()
                    .map(move |(index, (label, value))| {
                        let label = Label::new(label).size(LabelSize::Small).color(Color::Muted);

                        let row = match mode {
                            DescriptionListMode::Stacked => v_flex()
                                .gap_1()
                                .child(label)
                                .child(value)
                                .into_any_element(),
                            DescriptionListMode::Horizontal => h_flex()
                                .justify_between()
                                .gap_4()
                                .child(label)
                                .child(div().child(value))
                                .into_any_element(),
                        };

                        div()
                            .w_full()
                            .py_4()
                            .when(index > 0, |this| {
                                this.border_t_1().border_color(semantic::border_muted(cx))
                            })
                            .child(row)
                    }),
            )
    }
}

impl Component for DescriptionList {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> Option<&'static str> {
        Some("A key-value list for displaying structured details, in stacked or horizontal mode.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .children(vec![
                    example_group_with_title(
                        "Stacked",
                        vec![single_example(
                            "Stacked mode",
                            DescriptionList::new()
                                .item("Full name", Label::new("Margot Foster"))
                                .item("Application for", Label::new("Backend Developer"))
                                .item("Email address", Label::new("margotfoster@example.com"))
                                .into_any_element(),
                        )],
                    ),
                    example_group_with_title(
                        "Horizontal",
                        vec![single_example(
                            "Horizontal mode",
                            DescriptionList::new()
                                .mode(DescriptionListMode::Horizontal)
                                .item("Full name", Label::new("Margot Foster"))
                                .item("Application for", Label::new("Backend Developer"))
                                .item("Email address", Label::new("margotfoster@example.com"))
                                .into_any_element(),
                        )],
                    ),
                ])
                .into_any_element(),
        )
    }
}
