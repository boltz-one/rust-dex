use std::rc::Rc;

use crate::prelude::*;

/// Prev/next + numbered page navigation.
///
/// Presentational only: the caller owns the current page and is notified of
/// page changes via [`Pagination::on_change`]; this component does not hold
/// any page state itself.
#[derive(IntoElement, RegisterComponent)]
pub struct Pagination {
    id: ElementId,
    current_page: usize,
    total_pages: usize,
    on_change: Option<Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl Pagination {
    /// `current_page` and `total_pages` are both 1-indexed.
    pub fn new(id: impl Into<ElementId>, current_page: usize, total_pages: usize) -> Self {
        Self {
            id: id.into(),
            current_page: current_page.max(1),
            total_pages: total_pages.max(1),
            on_change: None,
        }
    }

    /// Called with the newly selected page whenever the user clicks a page
    /// number, or the prev/next buttons.
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Pagination {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let total = self.total_pages;
        let current = self.current_page.min(total);
        let on_change = self.on_change;

        let prev_disabled = current <= 1;
        let next_disabled = current >= total;

        let prev_handler = on_change.clone();
        let next_handler = on_change.clone();

        h_flex()
            .id(self.id)
            .items_center()
            .gap_1()
            .child(
                IconButton::new("pagination-prev", IconName::ChevronLeft)
                    .disabled(prev_disabled)
                    .when(!prev_disabled, |this| {
                        this.on_click(move |_, window, cx| {
                            if let Some(handler) = prev_handler.as_ref() {
                                handler(current - 1, window, cx);
                            }
                        })
                    }),
            )
            .children((1..=total).map(|page| {
                let handler = on_change.clone();
                let is_active = page == current;

                let button = Button::new(("pagination-page", page), page.to_string())
                    .label_size(LabelSize::Small);
                let button = if is_active {
                    button.primary()
                } else {
                    button.style(ButtonStyle::Transparent)
                };

                button.when(!is_active, |this| {
                    this.on_click(move |_, window, cx| {
                        if let Some(handler) = handler.as_ref() {
                            handler(page, window, cx);
                        }
                    })
                })
            }))
            .child(
                IconButton::new("pagination-next", IconName::ChevronRight)
                    .disabled(next_disabled)
                    .when(!next_disabled, |this| {
                        this.on_click(move |_, window, cx| {
                            if let Some(handler) = next_handler.as_ref() {
                                handler(current + 1, window, cx);
                            }
                        })
                    }),
            )
    }
}

impl Component for Pagination {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some("Prev/next and numbered page navigation; caller owns the current page.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .children(vec![
                    example_group_with_title(
                        "Middle Page",
                        vec![single_example(
                            "Page 3 of 5",
                            Pagination::new("pagination_middle", 3, 5).into_any_element(),
                        )],
                    ),
                    example_group_with_title(
                        "Bounds",
                        vec![
                            single_example(
                                "First Page",
                                Pagination::new("pagination_first", 1, 5).into_any_element(),
                            ),
                            single_example(
                                "Last Page",
                                Pagination::new("pagination_last", 5, 5).into_any_element(),
                            ),
                        ],
                    ),
                ])
                .into_any_element(),
        )
    }
}
