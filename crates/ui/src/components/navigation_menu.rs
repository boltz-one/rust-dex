//! Navigation menu with animated viewport for submenu content.
//!
//! Uses simplified hover-intent: submenus open on trigger hover and close when
//! the pointer leaves the menu root (no close-delay timer in this version).

use std::cell::Cell;
use std::rc::Rc;

use gpui::{AnyElement, Bounds, Context, Pixels, Render, anchored, canvas, deferred, point};
use smallvec::SmallVec;

use crate::prelude::*;

/// A single submenu entry shown inside a [`NavigationMenuContent`] viewport.
#[derive(Clone)]
pub struct NavigationMenuSubItem {
    pub label: SharedString,
}

/// Top-level navigation menu item definition.
#[derive(Clone)]
pub struct NavigationMenuItemDef {
    pub label: SharedString,
    pub items: Vec<NavigationMenuSubItem>,
}

/// Root container for a horizontal navigation menu.
#[derive(IntoElement)]
pub struct NavigationMenuList {
    children: SmallVec<[AnyElement; 4]>,
}

impl NavigationMenuList {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for NavigationMenuList {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for NavigationMenuList {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for NavigationMenuList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        h_flex().items_center().gap_1().children(self.children)
    }
}

/// A navigation menu trigger button.
#[derive(IntoElement)]
pub struct NavigationMenuTrigger {
    label: SharedString,
    active: bool,
}

impl NavigationMenuTrigger {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            active: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl RenderOnce for NavigationMenuTrigger {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        if self.active {
            Button::new("nav-menu-trigger", self.label)
                .primary()
                .style(ButtonStyle::Filled)
        } else {
            Button::new("nav-menu-trigger", self.label).style(ButtonStyle::Transparent)
        }
    }
}

/// Viewport panel that displays the active submenu's links.
#[derive(IntoElement)]
pub struct NavigationMenuViewport {
    children: SmallVec<[AnyElement; 6]>,
}

impl NavigationMenuViewport {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for NavigationMenuViewport {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for NavigationMenuViewport {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for NavigationMenuViewport {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .p_4()
            .rounded_md()
            .border_1()
            .border_color(semantic::border(cx))
            .bg(semantic::elevated_surface(cx))
            .shadow_level(Shadow::Md)
            .child(h_flex().gap_6().children(self.children))
    }
}

/// A link-style item inside a navigation menu viewport.
#[derive(IntoElement)]
pub struct NavigationMenuLink {
    label: SharedString,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl NavigationMenuLink {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for NavigationMenuLink {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hover = semantic::hover_bg(cx);
        let handler = self.on_click;
        h_flex()
            .id(ElementId::Name(self.label.clone()))
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .hover(move |s| s.bg(hover))
            .when_some(handler, |this, handler| {
                this.on_click(move |_, window, cx| handler(window, cx))
            })
            .child(Label::new(self.label))
    }
}

/// Content wrapper for a single submenu column inside the viewport.
#[derive(IntoElement)]
pub struct NavigationMenuContent {
    title: SharedString,
    children: SmallVec<[AnyElement; 4]>,
}

impl NavigationMenuContent {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            children: SmallVec::new(),
        }
    }
}

impl ParentElement for NavigationMenuContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for NavigationMenuContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        v_flex()
            .gap_2()
            .min_w(px(160.))
            .child(
                Label::new(self.title)
                    .size(LabelSize::Small)
                    .weight(gpui::FontWeight::SEMIBOLD),
            )
            .children(self.children)
    }
}

/// Stateful navigation menu with hover-driven submenu viewport.
///
/// Create with `cx.new(|_| NavigationMenu::new(items))`.
pub struct NavigationMenu {
    items: Vec<NavigationMenuItemDef>,
    open_index: Option<usize>,
    /// Real screen bounds of the trigger row, captured via an invisible
    /// `canvas()` measurement child every render and read back on the
    /// *next* render to position the floating viewport. Same idiom as
    /// `Combobox::trigger_bounds` / `Command::trigger_bounds`
    /// (`crates/ui/src/components/combobox.rs`,
    /// `crates/ui/src/components/command.rs`).
    trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl NavigationMenu {
    pub fn new(items: Vec<NavigationMenuItemDef>) -> Self {
        Self {
            items,
            open_index: None,
            trigger_bounds: Rc::new(Cell::new(None)),
        }
    }

    pub fn open_index(&self) -> Option<usize> {
        self.open_index
    }
}

impl Render for NavigationMenu {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open_index = self.open_index;

        let triggers = self.items.iter().enumerate().map(|(index, item)| {
            let is_open = open_index == Some(index);
            h_flex()
                .id(("navigation-menu-item", index))
                .on_hover(cx.listener(move |this, &hovered, _, cx| {
                    if hovered {
                        this.open_index = Some(index);
                    }
                    cx.notify();
                }))
                .child(NavigationMenuTrigger::new(item.label.clone()).active(is_open))
        });

        let viewport = open_index
            .and_then(|index| self.items.get(index))
            .map(|item| {
                NavigationMenuViewport::new().child(
                    NavigationMenuContent::new(item.label.clone()).children(
                        item.items
                            .iter()
                            .map(|sub| NavigationMenuLink::new(sub.label.clone())),
                    ),
                )
            });

        // Measure the trigger row's real bounds via an invisible `canvas()`
        // overlay, then float the viewport in a `deferred` overlay pass
        // anchored just below those bounds, instead of an inline flow child
        // — so opening a submenu never pushes sibling content down. Same
        // idiom as `Combobox`/`Command`
        // (`crates/ui/src/components/combobox.rs`,
        // `crates/ui/src/components/command.rs`).
        let trigger_bounds = self.trigger_bounds.clone();
        let trigger_row = div()
            .child(NavigationMenuList::new().children(triggers))
            .child({
                let trigger_bounds = trigger_bounds.clone();
                canvas(
                    move |bounds, _window, _cx| trigger_bounds.set(Some(bounds)),
                    |_bounds, _state, _window, _cx| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full()
            });

        let floating_viewport = viewport.map(|viewport| {
            let mut anchor = anchored().snap_to_window_with_margin(px(8.));
            if let Some(bounds) = self.trigger_bounds.get() {
                anchor = anchor.position(point(
                    bounds.origin.x,
                    bounds.origin.y + bounds.size.height + px(4.),
                ));
            }
            deferred(anchor.child(div().occlude().child(viewport))).with_priority(1)
        });

        v_flex()
            .id("navigation-menu")
            .gap_2()
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                if !hovered {
                    this.open_index = None;
                    cx.notify();
                }
            }))
            .child(trigger_row)
            .when_some(floating_viewport, |this, floating_viewport| {
                this.child(floating_viewport)
            })
    }
}

/// Gallery catalog entry for [`NavigationMenu`].
#[derive(IntoElement, RegisterComponent)]
pub struct NavigationMenuPreview;

impl RenderOnce for NavigationMenuPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        cx.new(|_| {
            NavigationMenu::new(vec![
                NavigationMenuItemDef {
                    label: "Getting Started".into(),
                    items: vec![
                        NavigationMenuSubItem {
                            label: "Introduction".into(),
                        },
                        NavigationMenuSubItem {
                            label: "Installation".into(),
                        },
                    ],
                },
                NavigationMenuItemDef {
                    label: "Components".into(),
                    items: vec![
                        NavigationMenuSubItem {
                            label: "Button".into(),
                        },
                        NavigationMenuSubItem {
                            label: "Dialog".into(),
                        },
                        NavigationMenuSubItem {
                            label: "Table".into(),
                        },
                    ],
                },
            ])
        })
    }
}

impl Component for NavigationMenuPreview {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some("Horizontal navigation menu with hover-intent submenu viewport.")
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        NavigationMenuPreview
            .render(window, cx)
            .into_any_element()
            .into()
    }
}
