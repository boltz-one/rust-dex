use gpui::{AnyElement, ClickEvent};
use smallvec::SmallVec;

use crate::prelude::*;

/// A single crumb within a [`Breadcrumb`] trail.
///
/// The last item added to a [`Breadcrumb`] is always rendered as the
/// non-interactive "current" item, regardless of whether [`Self::on_click`]
/// was set.
pub struct BreadcrumbItem {
    id: ElementId,
    label: SharedString,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl BreadcrumbItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_click: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// A horizontal trail of links showing the user's location within a hierarchy.
#[derive(IntoElement, RegisterComponent)]
pub struct Breadcrumb {
    items: SmallVec<[BreadcrumbItem; 4]>,
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self {
            items: SmallVec::new(),
        }
    }

    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let last_index = self.items.len().saturating_sub(1);
        let hover_color = semantic::text(cx);

        h_flex()
            .items_center()
            .gap_1()
            .children(self.items.into_iter().enumerate().map(|(index, item)| {
                let is_current = index == last_index;

                let crumb = if is_current {
                    div()
                        .text_color(semantic::text(cx))
                        .child(Label::new(item.label))
                        .into_any_element()
                } else {
                    h_flex()
                        .id(item.id)
                        .cursor_pointer()
                        .text_color(semantic::text_muted(cx))
                        .hover(move |this| this.text_color(hover_color))
                        .child(Label::new(item.label))
                        .when_some(item.on_click, |this, handler| this.on_click(handler))
                        .into_any_element()
                };

                h_flex()
                    .items_center()
                    .gap_1()
                    .when(index > 0, |this| {
                        this.child(
                            Icon::new(IconName::ChevronRight)
                                .size(IconSize::XSmall)
                                .color(Color::Muted),
                        )
                    })
                    .child(crumb)
                    .into_any_element()
            }))
    }
}

impl Component for Breadcrumb {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some("A horizontal trail of links showing the user's location within a hierarchy.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_6()
                .child(example_group_with_title(
                    "Basic Usage",
                    vec![
                        single_example(
                            "Three Levels",
                            Breadcrumb::new()
                                .item(BreadcrumbItem::new("crumb-home", "Home"))
                                .item(BreadcrumbItem::new("crumb-projects", "Projects"))
                                .item(BreadcrumbItem::new("crumb-current", "Current Page"))
                                .into_any_element(),
                        ),
                        single_example(
                            "Deep Hierarchy",
                            Breadcrumb::new()
                                .item(BreadcrumbItem::new("crumb-org", "Acme Corp"))
                                .item(BreadcrumbItem::new("crumb-team", "Engineering"))
                                .item(BreadcrumbItem::new("crumb-repo", "rust-dex"))
                                .item(BreadcrumbItem::new("crumb-file", "gallery_app.rs"))
                                .into_any_element(),
                        ),
                        single_example(
                            "Two Levels",
                            Breadcrumb::new()
                                .item(BreadcrumbItem::new("crumb-root", "Dashboard"))
                                .item(BreadcrumbItem::new("crumb-leaf", "Settings"))
                                .into_any_element(),
                        ),
                    ],
                ))
                .into_any_element(),
        )
    }
}
