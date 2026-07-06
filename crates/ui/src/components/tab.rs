use gpui::{AnyElement, Div, IntoElement, Stateful, transparent_black};
use smallvec::SmallVec;

use crate::TabBarStyle;
use crate::prelude::*;

/// A single tab within a [`TabBar`](crate::TabBar).
///
/// Renders per [`TabBarStyle`] (default [`TabBarStyle::Underline`]); pass the
/// same style used on the parent `TabBar` via [`Tab::style`] so both agree.
#[derive(IntoElement, RegisterComponent)]
pub struct Tab {
    div: Stateful<Div>,
    selected: bool,
    style: TabBarStyle,
    start_slot: Option<AnyElement>,
    end_slot: Option<AnyElement>,
    children: SmallVec<[AnyElement; 2]>,
}

impl Tab {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            div: div()
                .id(id.clone())
                .debug_selector(|| format!("TAB-{}", id)),
            selected: false,
            style: TabBarStyle::default(),
            start_slot: None,
            end_slot: None,
            children: SmallVec::new(),
        }
    }

    /// Sets the visual style. Should match the parent [`TabBar`](crate::TabBar)'s style.
    pub fn style(mut self, style: TabBarStyle) -> Self {
        self.style = style;
        self
    }

    pub fn start_slot<E: IntoElement>(mut self, element: impl Into<Option<E>>) -> Self {
        self.start_slot = element.into().map(IntoElement::into_any_element);
        self
    }

    pub fn end_slot<E: IntoElement>(mut self, element: impl Into<Option<E>>) -> Self {
        self.end_slot = element.into().map(IntoElement::into_any_element);
        self
    }

    pub fn content_height(cx: &App) -> Pixels {
        DynamicSpacing::Base32.px(cx) - px(1.)
    }

    pub fn container_height(cx: &App) -> Pixels {
        DynamicSpacing::Base32.px(cx)
    }
}

impl InteractiveElement for Tab {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.div.interactivity()
    }
}

impl StatefulInteractiveElement for Tab {}

impl Toggleable for Tab {
    fn toggle_state(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl ParentElement for Tab {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for Tab {
    #[allow(refining_impl_trait)]
    fn render(self, _: &mut Window, cx: &mut App) -> Stateful<Div> {
        // Title area grows (`flex_1`) so the end slot (e.g. a close "x") is
        // pinned to the tab's right edge instead of sitting right after the
        // text.
        let content = h_flex()
            .w_full()
            .items_center()
            .gap_2()
            .children(self.start_slot)
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .items_center()
                    .gap_2()
                    .children(self.children),
            )
            .children(self.end_slot);

        match self.style {
            TabBarStyle::Underline => {
                // VSCode-style tab: the active tab gets a top accent bar + a
                // slightly elevated background + bright text; inactive tabs are
                // flat/muted. Tabs sit flush, separated by a thin right divider.
                // (Top accent, not a bottom border, per the requested look.)
                let (text_color, bg, accent) = if self.selected {
                    (
                        semantic::text(cx),
                        semantic::elevated_surface(cx),
                        palette::primary(500),
                    )
                } else {
                    (
                        semantic::text_muted(cx),
                        transparent_black(),
                        transparent_black(),
                    )
                };
                let hover_color = semantic::text(cx);

                self.div
                    .h_full()
                    .min_w(px(140.))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .px_3()
                    .border_t_2()
                    .border_color(accent)
                    .bg(bg)
                    .text_color(text_color)
                    .hover(move |this| this.text_color(hover_color))
                    .child(content)
            }
            TabBarStyle::Pills => {
                let (text_color, bg) = if self.selected {
                    (semantic::text(cx), semantic::surface(cx))
                } else {
                    (semantic::text_muted(cx), transparent_black())
                };
                let hover_bg = semantic::hover_bg(cx);

                self.div
                    .cursor_pointer()
                    .px_3()
                    .py_1p5()
                    .rounded_md()
                    .bg(bg)
                    .text_color(text_color)
                    .when(self.selected, |this| this.shadow_level(Shadow::Sm))
                    .when(!self.selected, |this| {
                        this.hover(move |this| this.bg(hover_bg))
                    })
                    .child(content)
            }
        }
    }
}

impl Component for Tab {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some(
            "A tab component that can be used in a tabbed interface, supporting underline and pills styles.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .children(vec![
                    example_group_with_title(
                        "Underline",
                        vec![
                            single_example(
                                "Default",
                                Tab::new("underline_default")
                                    .child("Default Tab")
                                    .into_any_element(),
                            ),
                            single_example(
                                "Selected",
                                Tab::new("underline_selected")
                                    .toggle_state(true)
                                    .child("Selected Tab")
                                    .into_any_element(),
                            ),
                        ],
                    ),
                    example_group_with_title(
                        "Pills",
                        vec![
                            single_example(
                                "Default",
                                Tab::new("pills_default")
                                    .style(TabBarStyle::Pills)
                                    .child("Default Tab")
                                    .into_any_element(),
                            ),
                            single_example(
                                "Selected",
                                Tab::new("pills_selected")
                                    .style(TabBarStyle::Pills)
                                    .toggle_state(true)
                                    .child("Selected Tab")
                                    .into_any_element(),
                            ),
                        ],
                    ),
                ])
                .into_any_element(),
        )
    }
}
