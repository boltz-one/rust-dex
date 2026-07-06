use std::cmp::Ordering;

use gpui::{AnyElement, ScrollHandle};
use smallvec::SmallVec;

use crate::prelude::*;
use crate::{Tab, TabPosition};

/// Visual style for [`TabBar`] and its child [`Tab`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabBarStyle {
    /// Bottom-border container line; the active tab shows a colored
    /// underline. Preserves the existing default look for current callers.
    #[default]
    Underline,
    /// Rounded pill container; the active tab shows a raised pill background.
    Pills,
}

#[derive(IntoElement, RegisterComponent)]
pub struct TabBar {
    id: ElementId,
    style: TabBarStyle,
    start_children: SmallVec<[AnyElement; 2]>,
    children: SmallVec<[AnyElement; 2]>,
    end_children: SmallVec<[AnyElement; 2]>,
    scroll_handle: Option<ScrollHandle>,
}

impl TabBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: TabBarStyle::default(),
            start_children: SmallVec::new(),
            children: SmallVec::new(),
            end_children: SmallVec::new(),
            scroll_handle: None,
        }
    }

    /// Sets the visual style (default [`TabBarStyle::Underline`]).
    pub fn style(mut self, style: TabBarStyle) -> Self {
        self.style = style;
        self
    }

    pub fn track_scroll(mut self, scroll_handle: &ScrollHandle) -> Self {
        self.scroll_handle = Some(scroll_handle.clone());
        self
    }

    pub fn start_children_mut(&mut self) -> &mut SmallVec<[AnyElement; 2]> {
        &mut self.start_children
    }

    pub fn start_child(mut self, start_child: impl IntoElement) -> Self
    where
        Self: Sized,
    {
        self.start_children_mut()
            .push(start_child.into_element().into_any());
        self
    }

    pub fn start_children(
        mut self,
        start_children: impl IntoIterator<Item = impl IntoElement>,
    ) -> Self
    where
        Self: Sized,
    {
        self.start_children_mut().extend(
            start_children
                .into_iter()
                .map(|child| child.into_any_element()),
        );
        self
    }

    pub fn end_children_mut(&mut self) -> &mut SmallVec<[AnyElement; 2]> {
        &mut self.end_children
    }

    pub fn end_child(mut self, end_child: impl IntoElement) -> Self
    where
        Self: Sized,
    {
        self.end_children_mut()
            .push(end_child.into_element().into_any());
        self
    }

    pub fn end_children(mut self, end_children: impl IntoIterator<Item = impl IntoElement>) -> Self
    where
        Self: Sized,
    {
        self.end_children_mut().extend(
            end_children
                .into_iter()
                .map(|child| child.into_any_element()),
        );
        self
    }
}

impl ParentElement for TabBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for TabBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.style;

        let tabs_row = h_flex()
            .id("tabs")
            .flex_grow()
            .overflow_x_scroll()
            // Underline (VSCode-style) tabs sit flush with per-tab dividers;
            // only the Pills style keeps an inter-tab gap.
            .when(style == TabBarStyle::Pills, |this| this.gap_2())
            .when_some(self.scroll_handle, |this, scroll_handle| {
                this.track_scroll(&scroll_handle)
            })
            .children(self.children);

        let middle = match style {
            // No `.overflow_x_hidden()` here: the inner `tabs_row` already owns
            // its horizontal overflow via `.overflow_x_scroll()`, and an outer
            // clip on this `relative` wrapper made the `Tab` children
            // non-hit-testable (clicks at their real bounds silently no-op'd).
            // Bottom border line: painted as an absolute overlay *before*
            // `tabs_row` (so it sits underneath), rather than as a
            // `border_b_1` on this container, because the active `Tab`
            // "cuts through" the line via `pb_px` — that only reads
            // correctly if the line is painted under the tabs, not as part
            // of this wrapper's own border.
            TabBarStyle::Underline => div()
                .relative()
                .flex_1()
                .h_full()
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_b_1()
                        .border_color(cx.theme().colors().border),
                )
                .child(tabs_row)
                .into_any_element(),
            TabBarStyle::Pills => div()
                .flex_1()
                .p_1()
                .rounded_lg()
                .bg(semantic::elevated_surface(cx))
                .child(tabs_row)
                .into_any_element(),
        };

        div()
            .id(self.id)
            .group("tab_bar")
            .flex()
            .flex_none()
            .w_full()
            .h(Tab::container_height(cx))
            .bg(cx.theme().colors().tab_bar_background)
            .when(!self.start_children.is_empty(), |this| {
                this.child(
                    h_flex()
                        .flex_none()
                        .gap(DynamicSpacing::Base04.rems(cx))
                        .px(DynamicSpacing::Base06.rems(cx))
                        .border_b_1()
                        .border_r_1()
                        .border_color(cx.theme().colors().border)
                        .children(self.start_children),
                )
            })
            .child(middle)
            .when(!self.end_children.is_empty(), |this| {
                this.child(
                    h_flex()
                        .flex_none()
                        .gap(DynamicSpacing::Base04.rems(cx))
                        .px(DynamicSpacing::Base06.rems(cx))
                        .border_b_1()
                        .border_l_1()
                        .border_color(cx.theme().colors().border)
                        .children(self.end_children),
                )
            })
    }
}

impl Component for TabBar {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn name() -> &'static str {
        "TabBar"
    }

    fn description() -> Option<&'static str> {
        Some("A horizontal bar containing tabs for navigation between different views or sections.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .children(vec![
                    example_group_with_title(
                        "Underline (default)",
                        vec![single_example(
                            "With Tabs",
                            TabBar::new("underline_tab_bar")
                                .child(
                                    Tab::new("u_tab1")
                                        .position(TabPosition::First)
                                        .toggle_state(true)
                                        .child("Overview"),
                                )
                                .child(
                                    Tab::new("u_tab2")
                                        .position(TabPosition::Middle(Ordering::Greater))
                                        .child("Activity"),
                                )
                                .child(
                                    Tab::new("u_tab3")
                                        .position(TabPosition::Last)
                                        .child("Settings"),
                                )
                                .into_any_element(),
                        )],
                    ),
                    example_group_with_title(
                        "Pills",
                        vec![single_example(
                            "With Tabs",
                            TabBar::new("pills_tab_bar")
                                .style(TabBarStyle::Pills)
                                .child(
                                    Tab::new("p_tab1")
                                        .style(TabBarStyle::Pills)
                                        .toggle_state(true)
                                        .child("Overview"),
                                )
                                .child(
                                    Tab::new("p_tab2")
                                        .style(TabBarStyle::Pills)
                                        .child("Activity"),
                                )
                                .child(
                                    Tab::new("p_tab3")
                                        .style(TabBarStyle::Pills)
                                        .child("Settings"),
                                )
                                .into_any_element(),
                        )],
                    ),
                    example_group_with_title(
                        "With Start and End Children",
                        vec![single_example(
                            "Full TabBar",
                            TabBar::new("full_tab_bar")
                                .start_child(Button::new("start_button", "Start"))
                                .child(Tab::new("tab1"))
                                .child(Tab::new("tab2"))
                                .child(Tab::new("tab3"))
                                .end_child(Button::new("end_button", "End"))
                                .into_any_element(),
                        )],
                    ),
                ])
                .into_any_element(),
        )
    }
}
