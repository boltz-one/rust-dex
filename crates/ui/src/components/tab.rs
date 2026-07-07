use std::cmp::Ordering;

use gpui::{AnyElement, Div, IntoElement, Pixels, Stateful, px, transparent_black};
use smallvec::SmallVec;

use crate::TabBarStyle;
use crate::prelude::*;

const START_TAB_SLOT_SIZE: Pixels = px(12.);
const END_TAB_SLOT_SIZE: Pixels = px(14.);

/// The position of a [`Tab`] within a [`TabBar`](crate::TabBar)'s row, used
/// to decide which edges get a border in the [`TabBarStyle::Underline`]
/// style (ported from Zed's `tab.rs`).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TabPosition {
    First,
    Middle(Ordering),
    Last,
}

/// Which side of a [`Tab`]'s content the close button (or other end-slot
/// element) sits on, relative to the fixed-size start/end slots.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TabCloseSide {
    Start,
    End,
}

/// A single tab within a [`TabBar`](crate::TabBar).
///
/// Renders per [`TabBarStyle`] (default [`TabBarStyle::Underline`]); pass the
/// same style used on the parent `TabBar` via [`Tab::style`] so both agree.
#[derive(IntoElement, RegisterComponent)]
pub struct Tab {
    div: Stateful<Div>,
    selected: bool,
    style: TabBarStyle,
    position: TabPosition,
    close_side: TabCloseSide,
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
            position: TabPosition::First,
            close_side: TabCloseSide::End,
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

    /// Sets this tab's position within its row (see [`TabPosition`]).
    pub fn position(mut self, position: TabPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets which side the end slot (e.g. close button) sits on relative to
    /// the fixed-size start/end slots (see [`TabCloseSide`]).
    pub fn close_side(mut self, close_side: TabCloseSide) -> Self {
        self.close_side = close_side;
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
        match self.style {
            TabBarStyle::Underline => {
                let (text_color, tab_bg) = if self.selected {
                    (
                        cx.theme().colors().text,
                        cx.theme().colors().tab_active_background,
                    )
                } else {
                    (
                        cx.theme().colors().text_muted,
                        cx.theme().colors().tab_inactive_background,
                    )
                };

                // Start/end slots are fixed-size wrappers (12px/14px) so the
                // close "x" (end slot) has a stable hit target regardless of
                // title length; hoisted per `close_side` so it can sit on
                // either edge of the content.
                let start = h_flex()
                    .size(START_TAB_SLOT_SIZE)
                    .justify_center()
                    .children(self.start_slot);
                let end = h_flex()
                    .size(END_TAB_SLOT_SIZE)
                    .justify_center()
                    .children(self.end_slot);
                let (start_slot, end_slot) = match self.close_side {
                    TabCloseSide::End => (start, end),
                    TabCloseSide::Start => (end, start),
                };

                self.div
                    .h(Tab::container_height(cx))
                    .bg(tab_bg)
                    .border_color(cx.theme().colors().border)
                    .map(|this| match self.position {
                        TabPosition::First => {
                            if self.selected {
                                this.pl_px().border_r_1().pb_px()
                            } else {
                                this.pl_px().pr_px().border_b_1()
                            }
                        }
                        TabPosition::Last => {
                            if self.selected {
                                this.border_l_1().border_r_1().pb_px()
                            } else {
                                this.pl_px().border_b_1().border_r_1()
                            }
                        }
                        TabPosition::Middle(Ordering::Equal) => {
                            this.border_l_1().border_r_1().pb_px()
                        }
                        TabPosition::Middle(Ordering::Less) => {
                            this.border_l_1().pr_px().border_b_1()
                        }
                        TabPosition::Middle(Ordering::Greater) => {
                            this.border_r_1().pl_px().border_b_1()
                        }
                    })
                    .cursor_pointer()
                    .child(
                        h_flex()
                            .group("")
                            .relative()
                            .h(Tab::content_height(cx))
                            .px(DynamicSpacing::Base04.px(cx))
                            .gap(DynamicSpacing::Base04.rems(cx))
                            .text_color(text_color)
                            .child(start_slot)
                            .children(self.children)
                            .child(end_slot),
                    )
            }
            TabBarStyle::Pills => {
                // Title area grows (`flex_1`) so the end slot (e.g. a close
                // "x") is pinned to the tab's right edge instead of sitting
                // right after the text.
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
                                    .position(TabPosition::First)
                                    .child("Default Tab")
                                    .into_any_element(),
                            ),
                            single_example(
                                "Selected",
                                Tab::new("underline_selected")
                                    .position(TabPosition::Middle(Ordering::Equal))
                                    .toggle_state(true)
                                    .child("Selected Tab")
                                    .into_any_element(),
                            ),
                            single_example(
                                "Last",
                                Tab::new("underline_last")
                                    .position(TabPosition::Last)
                                    .child("Last Tab")
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
