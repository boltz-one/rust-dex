use gpui::{AnyElement, ClickEvent, ElementId};
use smallvec::SmallVec;

use crate::prelude::*;

/// A vertical navigation rail. Holds [`SidebarItem`]s (or any children).
#[derive(IntoElement, RegisterComponent)]
pub struct Sidebar {
    children: SmallVec<[AnyElement; 2]>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Sidebar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w(px(256.))
            .h_full()
            .flex_shrink_0()
            .gap_1()
            .p_2()
            .bg(semantic::surface(cx))
            .border_r_1()
            .border_color(semantic::border(cx))
            .children(self.children)
    }
}

impl Component for Sidebar {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some("A vertical navigation rail holding nav items.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            Sidebar::new()
                .child(SidebarItem::new("nav-home", "Home").active(true))
                .child(SidebarItem::new("nav-settings", "Settings"))
                .into_any_element(),
        )
    }
}

/// A single clickable navigation row inside a [`Sidebar`].
#[derive(IntoElement)]
pub struct SidebarItem {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    active: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl SidebarItem {
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

impl RenderOnce for SidebarItem {
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
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .text_color(text_color)
            .when(self.active, |this| this.bg(hover))
            .hover(|this| this.bg(hover))
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::Small))
            })
            .child(Label::new(self.label))
            .when_some(self.on_click, |this, handler| this.on_click(handler))
    }
}
