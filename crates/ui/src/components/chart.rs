//! GPUI `canvas()`-based charts: Bar, Line, Area, and Pie.
//!
//! ## Deferred (not faked)
//!
//! Radar, composed, and scatter chart types are explicitly out of scope —
//! Recharts parity for those would require substantially more layout/series
//! machinery than this hand-rolled primitive set.

use gpui::{Bounds, Hsla, PathBuilder, canvas, fill, point, px};

use crate::prelude::*;

/// Basic chart kinds supported by this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartKind {
    #[default]
    Bar,
    Line,
    Area,
    Pie,
}

/// Default chart color roles mapped from the design-system palette.
pub fn chart_colors() -> [Hsla; 5] {
    [
        palette::primary(500),
        palette::info(500),
        palette::success(500),
        palette::warning(500),
        palette::danger(500),
    ]
}

/// A lightweight data-in chart rendered via GPUI `canvas()`.
#[derive(IntoElement, RegisterComponent)]
pub struct Chart {
    kind: ChartKind,
    labels: Vec<SharedString>,
    values: Vec<f32>,
    colors: Vec<Hsla>,
    height: Pixels,
}

impl Chart {
    pub fn new(kind: ChartKind, labels: Vec<SharedString>, values: Vec<f32>) -> Self {
        let colors: Vec<Hsla> = values
            .iter()
            .enumerate()
            .map(|(i, _)| chart_colors()[i % chart_colors().len()])
            .collect();
        Self {
            kind,
            labels,
            values,
            colors,
            height: px(200.),
        }
    }

    pub fn bar(labels: Vec<SharedString>, values: Vec<f32>) -> Self {
        Self::new(ChartKind::Bar, labels, values)
    }

    pub fn line(labels: Vec<SharedString>, values: Vec<f32>) -> Self {
        Self::new(ChartKind::Line, labels, values)
    }

    pub fn area(labels: Vec<SharedString>, values: Vec<f32>) -> Self {
        Self::new(ChartKind::Area, labels, values)
    }

    pub fn pie(labels: Vec<SharedString>, values: Vec<f32>) -> Self {
        Self::new(ChartKind::Pie, labels, values)
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn colors(mut self, colors: Vec<Hsla>) -> Self {
        self.colors = colors;
        self
    }
}

fn draw_bar_chart(bounds: Bounds<Pixels>, values: &[f32], colors: &[Hsla], window: &mut Window) {
    if values.is_empty() {
        return;
    }
    let max = values.iter().copied().fold(0.0f32, f32::max).max(1.0);
    let padding = px(16.);
    let chart_width = bounds.size.width - padding * 2.;
    let chart_height = bounds.size.height - padding * 2.;
    let bar_gap = px(8.);
    let bar_width = (chart_width - bar_gap * (values.len().saturating_sub(1) as f32))
        / values.len().max(1) as f32;

    for (i, value) in values.iter().enumerate() {
        let bar_h = chart_height * (*value / max);
        let x = bounds.origin.x + padding + (bar_width + bar_gap) * i as f32;
        let y = bounds.origin.y + padding + chart_height - bar_h;
        let color = colors.get(i).copied().unwrap_or(palette::primary(500));
        window.paint_quad(fill(
            Bounds::from_corners(point(x, y), point(x + bar_width, y + bar_h)),
            color,
        ));
    }
}

fn draw_line_or_area(
    bounds: Bounds<Pixels>,
    values: &[f32],
    color: Hsla,
    fill_area: bool,
    window: &mut Window,
) {
    if values.len() < 2 {
        return;
    }
    let max = values.iter().copied().fold(0.0f32, f32::max).max(1.0);
    let padding = px(16.);
    let chart_width = bounds.size.width - padding * 2.;
    let chart_height = bounds.size.height - padding * 2.;
    let step = chart_width / (values.len() - 1) as f32;

    let points: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = bounds.origin.x + padding + step * i as f32;
            let y = bounds.origin.y + padding + chart_height * (1.0 - v / max);
            point(x, y)
        })
        .collect();

    if fill_area {
        let mut builder = PathBuilder::fill();
        let baseline_y = bounds.origin.y + padding + chart_height;
        builder.move_to(point(points[0].x, baseline_y));
        for p in &points {
            builder.line_to(*p);
        }
        builder.line_to(point(points.last().unwrap().x, baseline_y));
        builder.close();
        if let Ok(path) = builder.build() {
            window.paint_path(path, color.opacity(0.25));
        }
    }

    let mut stroke = PathBuilder::stroke(px(2.));
    stroke.move_to(points[0]);
    for p in points.iter().skip(1) {
        stroke.line_to(*p);
    }
    if let Ok(path) = stroke.build() {
        window.paint_path(path, color);
    }
}

fn draw_pie_chart(bounds: Bounds<Pixels>, values: &[f32], colors: &[Hsla], window: &mut Window) {
    let total: f32 = values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let center = point(
        bounds.origin.x + bounds.size.width / 2.,
        bounds.origin.y + bounds.size.height / 2.,
    );
    let radius = (bounds.size.width.min(bounds.size.height) / 2.) - px(16.);
    let mut start_angle = -std::f32::consts::FRAC_PI_2;

    for (i, value) in values.iter().enumerate() {
        let sweep = std::f32::consts::TAU * (value / total);
        let end_angle = start_angle + sweep;
        let color = colors.get(i).copied().unwrap_or(palette::primary(500));

        let mut builder = PathBuilder::fill();
        builder.move_to(center);
        let start = point(
            center.x + radius * start_angle.cos(),
            center.y + radius * start_angle.sin(),
        );
        builder.line_to(start);
        builder.arc_to(
            point(radius, radius),
            px(0.),
            sweep.abs() > std::f32::consts::PI,
            sweep > 0.0,
            point(
                center.x + radius * end_angle.cos(),
                center.y + radius * end_angle.sin(),
            ),
        );
        builder.close();
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
        start_angle = end_angle;
    }
}

impl RenderOnce for Chart {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let kind = self.kind;
        let values = self.values;
        let colors = self.colors.clone();
        let legend_colors = self.colors;
        let labels = self.labels;
        let height = self.height;
        let border = semantic::border(cx);
        let canvas_colors = colors.clone();

        v_flex()
            .w_full()
            .gap_2()
            // Test-only, no-op in release builds (mirrors the
            // `Tab`/`ContextMenu` `debug_selector` precedent) — `Chart` has
            // no interactive state to assert, so integration tests use this
            // to confirm each kind actually rendered (real bounds) rather
            // than merely not panicking.
            .debug_selector(move || format!("CHART-{kind:?}"))
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, _cx| match kind {
                        ChartKind::Bar => draw_bar_chart(bounds, &values, &canvas_colors, window),
                        ChartKind::Line => draw_line_or_area(
                            bounds,
                            &values,
                            canvas_colors
                                .first()
                                .copied()
                                .unwrap_or(palette::primary(500)),
                            false,
                            window,
                        ),
                        ChartKind::Area => draw_line_or_area(
                            bounds,
                            &values,
                            canvas_colors
                                .first()
                                .copied()
                                .unwrap_or(palette::primary(500)),
                            true,
                            window,
                        ),
                        ChartKind::Pie => draw_pie_chart(bounds, &values, &canvas_colors, window),
                    },
                )
                .w_full()
                .h(height)
                .rounded_md()
                .border_1()
                .border_color(border),
            )
            .when(!labels.is_empty(), |this| {
                this.child(h_flex().gap_3().flex_wrap().children(
                    labels.into_iter().enumerate().map(|(i, label)| {
                        let color = legend_colors
                            .get(i)
                            .copied()
                            .unwrap_or(palette::neutral(400));
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(div().size_2().rounded_full().bg(color))
                            .child(
                                Label::new(label)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            )
                    }),
                ))
            })
    }
}

impl Component for Chart {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> Option<&'static str> {
        Some("Bar/Line/Area/Pie charts via GPUI canvas(); Radar/composed/scatter deferred.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let labels = vec!["Jan".into(), "Feb".into(), "Mar".into(), "Apr".into()];
        let values = vec![12.0, 18.0, 9.0, 22.0];
        Some(
            v_flex()
                .gap_6()
                .child(Chart::bar(labels.clone(), values.clone()))
                .child(Chart::line(labels.clone(), values.clone()))
                .child(Chart::area(labels.clone(), values.clone()))
                .child(Chart::pie(labels, values))
                .into_any_element(),
        )
    }
}
