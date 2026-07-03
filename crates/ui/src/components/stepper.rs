use gpui::{AnyElement, ClickEvent, white};
use smallvec::SmallVec;

use crate::prelude::*;

/// A single step within a [`Stepper`].
pub struct StepperStep {
    label: SharedString,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl StepperStep {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StepState {
    Completed,
    Current,
    Upcoming,
}

/// A horizontal row of step circles connected by a line, for multi-step flows
/// (e.g. checkout, onboarding wizards).
///
/// Presentational only: the caller owns the current step index; this
/// component does not hold any state itself.
#[derive(IntoElement, RegisterComponent)]
pub struct Stepper {
    current_step: usize,
    steps: SmallVec<[StepperStep; 4]>,
}

impl Stepper {
    /// `current_step` is 0-indexed: `0` means the first step is in progress.
    pub fn new(current_step: usize) -> Self {
        Self {
            current_step,
            steps: SmallVec::new(),
        }
    }

    pub fn step(mut self, step: StepperStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn steps(mut self, steps: impl IntoIterator<Item = StepperStep>) -> Self {
        self.steps.extend(steps);
        self
    }
}

impl RenderOnce for Stepper {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let total = self.steps.len();
        let current_step = self.current_step;

        h_flex()
            .items_start()
            .children(self.steps.into_iter().enumerate().map(|(index, step)| {
                let state = if index < current_step {
                    StepState::Completed
                } else if index == current_step {
                    StepState::Current
                } else {
                    StepState::Upcoming
                };

                let (circle_bg, circle_border, accent_color) = match state {
                    StepState::Completed => (palette::primary(600), palette::primary(600), white()),
                    StepState::Current => (
                        gpui::transparent_black(),
                        palette::primary(600),
                        palette::primary(600),
                    ),
                    StepState::Upcoming => (
                        gpui::transparent_black(),
                        semantic::border_muted(cx),
                        semantic::text_muted(cx),
                    ),
                };

                let circle_content = if state == StepState::Completed {
                    Icon::new(IconName::Check)
                        .size(IconSize::Small)
                        .color(Color::Custom(white()))
                        .into_any_element()
                } else {
                    Label::new((index + 1).to_string())
                        .size(LabelSize::Small)
                        .color(Color::Custom(accent_color))
                        .into_any_element()
                };

                let circle = h_flex()
                    .id(("stepper-step", index))
                    .size_8()
                    .justify_center()
                    .rounded_full()
                    .border_2()
                    .border_color(circle_border)
                    .bg(circle_bg)
                    .when(step.on_click.is_some(), |this| this.cursor_pointer())
                    .child(circle_content)
                    .when_some(step.on_click, |this, handler| this.on_click(handler));

                h_flex()
                    .items_start()
                    .child(
                        v_flex().items_center().gap_1().child(circle).child(
                            Label::new(step.label)
                                .size(LabelSize::XSmall)
                                .color(Color::Custom(accent_color)),
                        ),
                    )
                    .when(index + 1 < total, |this| {
                        let line_color = if state == StepState::Completed {
                            palette::primary(600)
                        } else {
                            semantic::border_muted(cx)
                        };
                        this.child(div().w_8().h_px().mt(px(16.)).bg(line_color))
                    })
            }))
    }
}

impl Component for Stepper {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some("A horizontal row of step circles connected by a line, for multi-step flows.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let steps = || {
            [
                StepperStep::new("Account"),
                StepperStep::new("Profile"),
                StepperStep::new("Confirm"),
            ]
        };

        Some(
            v_flex()
                .gap_6()
                .child(example_group_with_title(
                    "Progress States",
                    vec![
                        single_example(
                            "Not Started",
                            Stepper::new(0).steps(steps()).into_any_element(),
                        ),
                        single_example(
                            "Second Step In Progress",
                            Stepper::new(1).steps(steps()).into_any_element(),
                        ),
                        single_example(
                            "All Complete",
                            Stepper::new(3).steps(steps()).into_any_element(),
                        ),
                    ],
                ))
                .into_any_element(),
        )
    }
}
