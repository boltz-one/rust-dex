//! Horizontal carousel with next/prev controls and drag-to-snap.
//!
//! Snap uses instant positioning on drag-release (no inertia physics).

use gpui::{Context, Empty, Hsla, Render};

use crate::prelude::*;

/// Drag payload for carousel pointer tracking.
#[derive(Debug)]
struct CarouselDrag {
    start_x: Pixels,
}

/// Horizontal carousel with next/prev navigation and drag-to-snap.
///
/// Create with `cx.new(|_| Carousel::new(slides))`.
pub struct Carousel {
    slides: Vec<(SharedString, Hsla)>,
    active_index: usize,
    drag_offset: Pixels,
}

impl Carousel {
    pub fn new(slides: impl IntoIterator<Item = (impl Into<SharedString>, Hsla)>) -> Self {
        Self {
            slides: slides
                .into_iter()
                .map(|(label, color)| (label.into(), color))
                .collect(),
            active_index: 0,
            drag_offset: px(0.),
        }
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    fn snap_to_nearest(&mut self, cx: &mut Context<Self>) {
        if self.slides.is_empty() {
            return;
        }
        let threshold = px(48.);
        if self.drag_offset < -threshold && self.active_index + 1 < self.slides.len() {
            self.active_index += 1;
        } else if self.drag_offset > threshold && self.active_index > 0 {
            self.active_index -= 1;
        }
        self.drag_offset = px(0.);
        cx.notify();
    }
}

impl Render for Carousel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.slides.len();
        let active = self.active_index.min(count.saturating_sub(1));
        let drag_offset = self.drag_offset;
        let (label, color) = self
            .slides
            .get(active)
            .cloned()
            .unwrap_or_else(|| ("No items".into(), palette::neutral(100)));

        let track = div()
            .id("carousel-track")
            .w_full()
            .h(px(200.))
            .overflow_hidden()
            .cursor_pointer()
            .on_drag(
                CarouselDrag {
                    start_x: window.mouse_position().x,
                },
                |_, _, _, cx| cx.new(|_| Empty),
            )
            .on_drag_move::<CarouselDrag>(cx.listener(
                |this, event: &gpui::DragMoveEvent<CarouselDrag>, _, cx| {
                    let drag = event.drag(cx);
                    this.drag_offset = event.event.position.x - drag.start_x;
                    cx.notify();
                },
            ))
            .on_drop::<CarouselDrag>(cx.listener(|this, _, _, cx| {
                this.snap_to_nearest(cx);
            }))
            .child(
                div()
                    .w_full()
                    .h_full()
                    .ml(drag_offset)
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(color)
                    .child(Headline::new(label).size(HeadlineSize::Small)),
            );

        let prev_disabled = active == 0;
        let next_disabled = active + 1 >= count;

        v_flex()
            .id("carousel")
            .w_full()
            .max_w(px(480.))
            .gap_3()
            .child(track)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        // Wrapping `div` only exists so integration tests can
                        // locate the prev button's real rendered pixel bounds
                        // via `VisualTestContext::debug_bounds` (test-only,
                        // no-op in release builds — mirrors the
                        // `ActionPanel`/`Tab` `debug_selector` precedent).
                        // `IconButton::new` already sets its own
                        // `"ICON-{icon:?}"` debug_selector, but `Calendar`'s
                        // prev/next buttons use the same `ChevronLeft`/
                        // `ChevronRight` icons and both components can render
                        // simultaneously on the gallery's Layout page, so a
                        // distinct selector is needed to avoid ambiguity. The
                        // wrapping div does not intercept the click: the
                        // inner `IconButton` still owns the only click
                        // handler.
                        div().debug_selector(|| "CAROUSEL-PREV".into()).child(
                            IconButton::new("carousel-prev", IconName::ChevronLeft)
                                .disabled(prev_disabled)
                                .when(!prev_disabled, |this| {
                                    this.on_click(cx.listener(|this, _, _, cx| {
                                        if this.active_index > 0 {
                                            this.active_index -= 1;
                                            this.drag_offset = px(0.);
                                            cx.notify();
                                        }
                                    }))
                                }),
                        ),
                    )
                    .child(
                        Label::new(format!("{} / {}", active + 1, count.max(1)))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        // See the prev button wrapper's comment above; same
                        // rationale.
                        div().debug_selector(|| "CAROUSEL-NEXT".into()).child(
                            IconButton::new("carousel-next", IconName::ChevronRight)
                                .disabled(next_disabled)
                                .when(!next_disabled, |this| {
                                    this.on_click(cx.listener(|this, _, _, cx| {
                                        if this.active_index + 1 < this.slides.len() {
                                            this.active_index += 1;
                                            this.drag_offset = px(0.);
                                            cx.notify();
                                        }
                                    }))
                                }),
                        ),
                    ),
            )
    }
}

/// Gallery catalog entry for [`Carousel`].
#[derive(IntoElement, RegisterComponent)]
pub struct CarouselPreview;

impl RenderOnce for CarouselPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        cx.new(|_| {
            Carousel::new([
                ("Slide 1", palette::primary(100)),
                ("Slide 2", palette::success(100)),
                ("Slide 3", palette::warning(100)),
            ])
        })
    }
}

impl Component for CarouselPreview {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> Option<&'static str> {
        Some("Horizontal carousel with next/prev navigation and instant drag-snap.")
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        CarouselPreview.render(window, cx).into_any_element().into()
    }
}
