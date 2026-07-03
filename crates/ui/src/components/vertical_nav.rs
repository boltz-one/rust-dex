use gpui::{AnyElement, ClickEvent};
use smallvec::SmallVec;

use crate::prelude::*;

/// A flat list of navigation links for in-page sub-navigation (e.g. a
/// settings page's left-hand link list).
///
/// **Distinct from [`Sidebar`](crate::Sidebar):** `Sidebar` is the app-level,
/// collapsible navigation container that also hosts nested sections.
/// `VerticalNav` is a minimal, flat link list only — it does NOT support
/// collapse/nesting. Use `Sidebar` if you need that.
#[derive(IntoElement, RegisterComponent)]
pub struct VerticalNav {
    children: SmallVec<[AnyElement; 4]>,
}

impl VerticalNav {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for VerticalNav {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for VerticalNav {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for VerticalNav {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        v_flex().w(px(220.)).gap_1().children(self.children)
    }
}

impl Component for VerticalNav {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some(
            "A flat list of navigation links for in-page sub-navigation; distinct from the collapsible Sidebar.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(example_group_with_title(
                    "Basic Usage",
                    vec![single_example(
                        "Default",
                        VerticalNav::new()
                            .child(
                                VerticalNavItem::new("vnav-general", "General")
                                    .icon(IconName::Check)
                                    .active(true),
                            )
                            .child(VerticalNavItem::new("vnav-security", "Security"))
                            .child(VerticalNavItem::new("vnav-billing", "Billing"))
                            .into_any_element(),
                    )],
                ))
                .into_any_element(),
        )
    }
}

/// A single clickable link within a [`VerticalNav`].
#[derive(IntoElement)]
pub struct VerticalNavItem {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    active: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl VerticalNavItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            active: false,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
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

impl RenderOnce for VerticalNavItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let text_color = if self.active {
            semantic::text(cx)
        } else {
            semantic::text_muted(cx)
        };
        let hover = semantic::hover_bg(cx);

        h_flex()
            .id(self.id)
            .items_center()
            .gap_2()
            .px_4()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .text_color(text_color)
            .when(self.active, |this| this.bg(semantic::elevated_surface(cx)))
            .hover(move |this| this.bg(hover))
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::Small))
            })
            .child(Label::new(self.label))
            .when_some(self.on_click, |this, handler| this.on_click(handler))
    }
}
