use gpui::{Hsla, IntoElement, PathBuilder, canvas, point};

use crate::prelude::*;

pub fn divider() -> Divider {
    Divider {
        style: DividerStyle::Solid,
        direction: DividerDirection::Horizontal,
        color: DividerColor::default(),
        inset: false,
        label: None,
    }
}

pub fn vertical_divider() -> Divider {
    Divider {
        style: DividerStyle::Solid,
        direction: DividerDirection::Vertical,
        color: DividerColor::default(),
        inset: false,
        label: None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DividerStyle {
    Solid,
    Dashed,
}

#[derive(Clone, Copy, PartialEq)]
enum DividerDirection {
    Horizontal,
    Vertical,
}

/// The color of a [`Divider`].
#[derive(Default)]
pub enum DividerColor {
    Border,
    BorderFaded,
    #[default]
    BorderVariant,
}

impl DividerColor {
    pub fn hsla(self, cx: &mut App) -> Hsla {
        match self {
            DividerColor::Border => semantic::border(cx),
            DividerColor::BorderFaded => semantic::border(cx).opacity(0.6),
            DividerColor::BorderVariant => semantic::border_muted(cx),
        }
    }
}

#[derive(IntoElement, RegisterComponent)]
pub struct Divider {
    style: DividerStyle,
    direction: DividerDirection,
    color: DividerColor,
    inset: bool,
    label: Option<SharedString>,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            style: DividerStyle::Solid,
            direction: DividerDirection::Horizontal,
            color: DividerColor::default(),
            inset: false,
            label: None,
        }
    }

    pub fn vertical() -> Self {
        Self {
            style: DividerStyle::Solid,
            direction: DividerDirection::Vertical,
            color: DividerColor::default(),
            inset: false,
            label: None,
        }
    }

    pub fn horizontal_dashed() -> Self {
        Self {
            style: DividerStyle::Dashed,
            direction: DividerDirection::Horizontal,
            color: DividerColor::default(),
            inset: false,
            label: None,
        }
    }

    pub fn vertical_dashed() -> Self {
        Self {
            style: DividerStyle::Dashed,
            direction: DividerDirection::Vertical,
            color: DividerColor::default(),
            inset: false,
            label: None,
        }
    }

    pub fn inset(mut self) -> Self {
        self.inset = true;
        self
    }

    pub fn color(mut self, color: DividerColor) -> Self {
        self.color = color;
        self
    }

    /// Renders a label centered on the divider line, with the line
    /// continuing on both sides. Only supported for horizontal dividers.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn render_solid(self, base: Div, cx: &mut App) -> impl IntoElement {
        base.bg(self.color.hsla(cx))
    }

    pub fn render_dashed(self, base: Div) -> impl IntoElement {
        base.relative().child(
            canvas(
                |_, _, _| {},
                move |bounds, _, window, cx| {
                    let mut builder = PathBuilder::stroke(px(1.)).dash_array(&[px(4.), px(2.)]);
                    let (start, end) = match self.direction {
                        DividerDirection::Horizontal => {
                            let x = bounds.origin.x;
                            let y = bounds.origin.y + px(0.5);
                            (point(x, y), point(x + bounds.size.width, y))
                        }
                        DividerDirection::Vertical => {
                            let x = bounds.origin.x + px(0.5);
                            let y = bounds.origin.y;
                            (point(x, y), point(x, y + bounds.size.height))
                        }
                    };
                    builder.move_to(start);
                    builder.line_to(end);
                    if let Ok(line) = builder.build() {
                        window.paint_path(line, self.color.hsla(cx));
                    }
                },
            )
            .absolute()
            .size_full(),
        )
    }
}

impl RenderOnce for Divider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.direction == DividerDirection::Horizontal {
            if let Some(label) = self.label.clone() {
                let line_color = self.color.hsla(cx);
                return h_flex()
                    .w_full()
                    .items_center()
                    .gap_2()
                    .child(div().h_px().flex_1().bg(line_color))
                    .child(
                        Label::new(label)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(div().h_px().flex_1().bg(line_color))
                    .into_any_element();
            }
        }

        let base = match self.direction {
            DividerDirection::Horizontal => div()
                .min_w_0()
                .h_px()
                .w_full()
                .when(self.inset, |this| this.mx_1p5()),
            DividerDirection::Vertical => div()
                .min_w_0()
                .w_px()
                .h_full()
                .when(self.inset, |this| this.my_1p5()),
        };

        match self.style {
            DividerStyle::Solid => self.render_solid(base, cx).into_any_element(),
            DividerStyle::Dashed => self.render_dashed(base).into_any_element(),
        }
    }
}

impl Component for Divider {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some(
            "Visual separator used to create divisions between groups of content or sections in a layout.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .children(vec![
                    example_group_with_title(
                        "Horizontal Dividers",
                        vec![
                            single_example("Default", Divider::horizontal().into_any_element()),
                            single_example(
                                "Border Color",
                                Divider::horizontal()
                                    .color(DividerColor::Border)
                                    .into_any_element(),
                            ),
                            single_example(
                                "Inset",
                                Divider::horizontal().inset().into_any_element(),
                            ),
                            single_example(
                                "Dashed",
                                Divider::horizontal_dashed().into_any_element(),
                            ),
                            single_example(
                                "Labeled",
                                Divider::horizontal().label("OR").into_any_element(),
                            ),
                        ],
                    ),
                    example_group_with_title(
                        "Vertical Dividers",
                        vec![
                            single_example(
                                "Default",
                                div().h_16().child(Divider::vertical()).into_any_element(),
                            ),
                            single_example(
                                "Border Color",
                                div()
                                    .h_16()
                                    .child(Divider::vertical().color(DividerColor::Border))
                                    .into_any_element(),
                            ),
                            single_example(
                                "Inset",
                                div()
                                    .h_16()
                                    .child(Divider::vertical().inset())
                                    .into_any_element(),
                            ),
                            single_example(
                                "Dashed",
                                div()
                                    .h_16()
                                    .child(Divider::vertical_dashed())
                                    .into_any_element(),
                            ),
                        ],
                    ),
                    example_group_with_title(
                        "Example Usage",
                        vec![single_example(
                            "Between Content",
                            v_flex()
                                .w_full()
                                .gap_4()
                                .px_4()
                                .child(Label::new("Section One"))
                                .child(Divider::horizontal())
                                .child(Label::new("Section Two"))
                                .child(Divider::horizontal_dashed())
                                .child(Label::new("Section Three"))
                                .child(Divider::horizontal().label("End"))
                                .into_any_element(),
                        )],
                    ),
                ])
                .into_any_element(),
        )
    }
}
