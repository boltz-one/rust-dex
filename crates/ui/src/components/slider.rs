use std::{cell::Cell, rc::Rc};

use gpui::{Bounds, Context, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Render, canvas};

use crate::prelude::*;

const THUMB_SIZE: Pixels = px(16.);
const TRACK_HEIGHT: Pixels = px(4.);

/// A single-value or range slider with pointer-drag on the track/thumb.
///
/// Pointer geometry mirrors [`crate::scrollbar`]'s thumb-drag pattern: mouse
/// position within the track bounds maps to a clamped 0.0–1.0 fraction, then to
/// `[min, max]`.
///
/// Stateful view — create with `cx.new(|cx| Slider::new(cx))`.
pub struct Slider {
    min: f32,
    max: f32,
    step: f32,
    /// Single-thumb value, or low end when `range` is enabled.
    value: f32,
    /// High end when range mode is enabled.
    range_high: f32,
    range_mode: bool,
    disabled: bool,
    dragging_thumb: Option<usize>,
    track_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl Slider {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            min: 0.,
            max: 100.,
            step: 1.,
            value: 0.,
            range_high: 100.,
            range_mode: false,
            disabled: false,
            dragging_thumb: None,
            track_bounds: Rc::new(Cell::new(None)),
        }
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step.max(f32::MIN_POSITIVE);
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Enables two-thumb range mode with `(low, high)` initial values.
    pub fn range(mut self, low: f32, high: f32) -> Self {
        self.range_mode = true;
        self.value = low;
        self.range_high = high;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn values(&self) -> (f32, f32) {
        if self.range_mode {
            (
                self.clamp_value(self.value),
                self.clamp_value(self.range_high),
            )
        } else {
            (self.clamp_value(self.value), self.clamp_value(self.value))
        }
    }

    fn clamp_value(&self, raw: f32) -> f32 {
        // `f32::clamp` panics if `min > max`; guard against a caller passing
        // an inverted range via `.min()`/`.max()` by normalizing the bounds
        // here rather than trusting call order.
        let (lo, hi) = (self.min.min(self.max), self.min.max(self.max));
        let clamped = raw.clamp(lo, hi);
        if self.step <= 0. {
            return clamped;
        }
        let steps = ((clamped - lo) / self.step).round();
        (lo + steps * self.step).clamp(lo, hi)
    }

    fn fraction_from_x(&self, x: Pixels, bounds: &Bounds<Pixels>) -> f32 {
        if bounds.size.width <= Pixels::ZERO {
            return 0.;
        }
        let offset = (x - bounds.origin.x).clamp(Pixels::ZERO, bounds.size.width);
        (offset / bounds.size.width).clamp(0., 1.)
    }

    fn value_from_x(&self, x: Pixels, bounds: &Bounds<Pixels>) -> f32 {
        let fraction = self.fraction_from_x(x, bounds);
        self.clamp_value(self.min + fraction * (self.max - self.min))
    }

    fn nearest_thumb(&self, value: f32) -> usize {
        if !self.range_mode {
            return 0;
        }
        let low_dist = (value - self.value).abs();
        let high_dist = (value - self.range_high).abs();
        if high_dist < low_dist { 1 } else { 0 }
    }

    fn set_thumb_value(&mut self, thumb: usize, value: f32, cx: &mut Context<Self>) {
        let value = self.clamp_value(value);
        if self.range_mode {
            if thumb == 0 {
                self.value = value.min(self.range_high);
            } else {
                self.range_high = value.max(self.value);
            }
        } else {
            self.value = value;
        }
        cx.notify();
    }

    fn update_from_position(&mut self, x: Pixels, thumb: Option<usize>, cx: &mut Context<Self>) {
        let Some(bounds) = self.track_bounds.get() else {
            return;
        };
        let value = self.value_from_x(x, &bounds);
        let thumb = thumb.unwrap_or_else(|| self.nearest_thumb(value));
        self.set_thumb_value(thumb, value, cx);
    }
}

impl Render for Slider {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (low, high) = self.values();
        let span = (self.max - self.min).max(f32::MIN_POSITIVE);
        let low_pct = (low - self.min) / span;
        let high_pct = (high - self.min) / span;
        let range_mode = self.range_mode;
        let disabled = self.disabled;
        let track_bounds = self.track_bounds.clone();
        let dragging = self.dragging_thumb;

        let thumb = |id: ElementId, pct: f32| {
            div()
                .id(id)
                .absolute()
                .top(px(-6.))
                .left(relative(pct))
                .ml(-THUMB_SIZE / 2.)
                .size(THUMB_SIZE)
                .rounded_full()
                .bg(palette::primary(600))
                .border_2()
                .border_color(semantic::elevated_surface(cx))
                .shadow_level(Shadow::Sm)
                .when(!disabled, |this| this.cursor_grab())
        };

        let range_left = low_pct.min(high_pct);
        let range_width = (high_pct - low_pct).abs();

        div()
            .id("slider")
            .w_full()
            .py_3()
            .when(disabled, |this| this.opacity(0.5))
            .child(
                div()
                    .id("slider-track-container")
                    .relative()
                    .w_full()
                    .h(THUMB_SIZE)
                    .child(
                        div()
                            .absolute()
                            .top(px(6.))
                            .left_0()
                            .right_0()
                            .h(TRACK_HEIGHT)
                            .rounded_full()
                            .bg(semantic::border_muted(cx)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(6.))
                            .left(relative(range_left))
                            .w(relative(range_width.max(if range_mode {
                                0.
                            } else {
                                high_pct
                            })))
                            .h(TRACK_HEIGHT)
                            .rounded_full()
                            .bg(palette::primary(500)),
                    )
                    .child(thumb("slider-thumb-low".into(), low_pct))
                    .when(range_mode, |this| {
                        this.child(thumb("slider-thumb-high".into(), high_pct))
                    })
                    .child({
                        let track_bounds = track_bounds.clone();
                        canvas(
                            move |bounds, _window, _cx| track_bounds.set(Some(bounds)),
                            |_bounds, _state, _window, _cx| {},
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                    })
                    .when(!disabled, |this| {
                        this.on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                                let value = this
                                    .track_bounds
                                    .get()
                                    .map(|b| this.value_from_x(event.position.x, &b));
                                let thumb = value.map(|v| this.nearest_thumb(v));
                                this.dragging_thumb = thumb;
                                this.update_from_position(event.position.x, thumb, cx);
                            }),
                        )
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.dragging_thumb = None;
                                cx.notify();
                            }),
                        )
                        .on_mouse_move(cx.listener(
                            |this, event: &MouseMoveEvent, _, cx| {
                                if event.dragging() || this.dragging_thumb.is_some() {
                                    this.update_from_position(
                                        event.position.x,
                                        this.dragging_thumb,
                                        cx,
                                    );
                                }
                            },
                        ))
                    }),
            )
            .child(
                h_flex()
                    .mt_1()
                    .justify_between()
                    .child(
                        Label::new(format!("{low:.0}"))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .when(range_mode, |this| {
                        this.child(
                            Label::new(format!("{high:.0}"))
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                    }),
            )
            .when(dragging.is_some(), |this| this.cursor_grabbing())
    }
}

impl Component for Slider {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("A slider for selecting a single value or a range via pointer drag.")
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .w(px(280.))
                .child(cx.new(|cx| Slider::new(cx).value(40.)))
                .child(cx.new(|cx| Slider::new(cx).range(25., 75.)))
                .into_any_element(),
        )
    }
}
