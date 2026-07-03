use gpui::{ClickEvent, ElementId};

use crate::prelude::*;

/// A single radio button. Group exclusivity is the caller's responsibility:
/// the parent holds the selected value and passes `checked` to each button.
#[derive(IntoElement, RegisterComponent)]
pub struct RadioButton {
    id: ElementId,
    label: Option<SharedString>,
    checked: bool,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl RadioButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            checked: false,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for RadioButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ring = if self.checked {
            palette::primary(600)
        } else {
            semantic::border(cx)
        };

        let control = div()
            .size_4()
            .rounded_full()
            .border_1()
            .border_color(ring)
            .flex()
            .items_center()
            .justify_center()
            .when(self.checked, |this| {
                this.child(div().size_1p5().rounded_full().bg(palette::primary(600)))
            });

        h_flex()
            .id(self.id)
            .items_center()
            .gap_2()
            .when(!self.disabled, |this| this.cursor_pointer())
            .when(self.disabled, |this| this.opacity(0.5))
            .child(control)
            .when_some(self.label, |this, label| {
                this.child(Label::new(label).size(LabelSize::Small))
            })
            .when_some(self.on_click.filter(|_| !self.disabled), |this, handler| {
                this.on_click(handler)
            })
    }
}

impl Component for RadioButton {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("A single radio button; group exclusivity is caller-managed.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            Label::new("States")
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .child(RadioButton::new("radio-a").label("Selected").checked(true))
                        .child(RadioButton::new("radio-b").label("Unselected"))
                        .child(
                            RadioButton::new("radio-c")
                                .label("Disabled, unselected")
                                .disabled(true),
                        )
                        .child(
                            RadioButton::new("radio-d")
                                .label("Disabled, selected")
                                .checked(true)
                                .disabled(true),
                        ),
                )
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            Label::new("Shipping method")
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .child(
                            RadioButton::new("ship-standard")
                                .label("Standard (5-7 days)")
                                .checked(true),
                        )
                        .child(RadioButton::new("ship-express").label("Express (2-3 days)"))
                        .child(RadioButton::new("ship-overnight").label("Overnight")),
                )
                .into_any_element(),
        )
    }
}
