use gpui::{FontWeight, Hsla, white};

use crate::prelude::*;

/// Fill style of a [`Badge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    /// Light tinted background with dark text.
    #[default]
    Soft,
    /// Strong filled background with light text.
    Solid,
    /// Transparent with a colored border.
    Outline,
}

/// Semantic color role of a [`Badge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeColor {
    #[default]
    Neutral,
    Primary,
    Success,
    Warning,
    Danger,
}

impl BadgeColor {
    fn shade(self, step: u16) -> Hsla {
        match self {
            BadgeColor::Neutral => palette::neutral(step),
            BadgeColor::Primary => palette::primary(step),
            BadgeColor::Success => palette::success(step),
            BadgeColor::Warning => palette::warning(step),
            BadgeColor::Danger => palette::danger(step),
        }
    }
}

/// A small pill label conveying status or category.
#[derive(IntoElement, RegisterComponent)]
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
    color: BadgeColor,
    dot: bool,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::default(),
            color: BadgeColor::default(),
            dot: false,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(mut self, color: BadgeColor) -> Self {
        self.color = color;
        self
    }

    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut base = h_flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_0p5()
            .rounded_full();

        base = match self.variant {
            BadgeVariant::Soft => base
                .bg(self.color.shade(100))
                .text_color(self.color.shade(800)),
            BadgeVariant::Solid => base.bg(self.color.shade(600)).text_color(white()),
            BadgeVariant::Outline => base
                .border_1()
                .border_color(self.color.shade(300))
                .text_color(self.color.shade(700)),
        };

        base.when(self.dot, |this| {
            this.child(div().size_1p5().rounded_full().bg(self.color.shade(500)))
        })
        .child(
            Label::new(self.label)
                .size(LabelSize::XSmall)
                .weight(FontWeight::MEDIUM)
                .color(Color::Custom(match self.variant {
                    BadgeVariant::Solid => white(),
                    BadgeVariant::Soft => self.color.shade(800),
                    BadgeVariant::Outline => self.color.shade(700),
                })),
        )
    }
}

impl Component for Badge {
    fn scope() -> ComponentScope {
        ComponentScope::Status
    }

    fn description() -> Option<&'static str> {
        Some("A small pill label conveying status or category.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let colors = [
            ("Neutral", BadgeColor::Neutral),
            ("Primary", BadgeColor::Primary),
            ("Success", BadgeColor::Success),
            ("Warning", BadgeColor::Warning),
            ("Danger", BadgeColor::Danger),
        ];
        let row = |variant: BadgeVariant, dot: bool| {
            let mut r = h_flex().gap_2();
            for (name, color) in colors {
                r = r.child(Badge::new(name).variant(variant).color(color).dot(dot));
            }
            r
        };
        let status_row = || {
            h_flex()
                .gap_2()
                .child(
                    Badge::new("Active")
                        .variant(BadgeVariant::Soft)
                        .color(BadgeColor::Success)
                        .dot(true),
                )
                .child(
                    Badge::new("Pending")
                        .variant(BadgeVariant::Soft)
                        .color(BadgeColor::Warning)
                        .dot(true),
                )
                .child(
                    Badge::new("Failed")
                        .variant(BadgeVariant::Solid)
                        .color(BadgeColor::Danger),
                )
                .child(
                    Badge::new("Draft")
                        .variant(BadgeVariant::Outline)
                        .color(BadgeColor::Neutral),
                )
        };

        Some(
            v_flex()
                .gap_3()
                .child(row(BadgeVariant::Soft, true))
                .child(row(BadgeVariant::Solid, true))
                .child(row(BadgeVariant::Outline, true))
                .child(row(BadgeVariant::Soft, false))
                .child(row(BadgeVariant::Solid, false))
                .child(row(BadgeVariant::Outline, false))
                .child(status_row())
                .into_any_element(),
        )
    }
}
