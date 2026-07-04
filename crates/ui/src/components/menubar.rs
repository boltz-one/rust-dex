use gpui::{Anchor, Context, Entity, Render};

use crate::{ContextMenu, PopoverMenu, PopoverMenuHandle, prelude::*};

struct MenubarItem {
    label: SharedString,
    menu: Entity<ContextMenu>,
    handle: PopoverMenuHandle<ContextMenu>,
}

/// A horizontal menubar with dropdown menus. Only one menu is open at a time;
/// left/right arrow keys move between adjacent top-level menus while any menu
/// is open.
///
/// Stateful view — create with `cx.new(|cx| Menubar::new(cx))` and add menus
/// via [`Menubar::item`].
#[derive(RegisterComponent)]
pub struct Menubar {
    items: Vec<MenubarItem>,
    open_index: Option<usize>,
    focus_index: usize,
}

impl Menubar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            items: Vec::new(),
            open_index: None,
            focus_index: 0,
        }
    }

    pub fn item(mut self, label: impl Into<SharedString>, menu: Entity<ContextMenu>) -> Self {
        self.items.push(MenubarItem {
            label: label.into(),
            menu,
            handle: PopoverMenuHandle::default(),
        });
        self
    }

    fn close_all(&mut self, cx: &mut App) {
        for item in &self.items {
            item.handle.hide(cx);
        }
        self.open_index = None;
    }

    fn open_at(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.items.len() {
            return;
        }
        self.close_all(cx);
        self.focus_index = index;
        self.open_index = Some(index);
        self.items[index].handle.show(window, cx);
        cx.notify();
    }

    fn move_focus(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        let len = self.items.len() as isize;
        let next = (self.focus_index as isize + delta).rem_euclid(len) as usize;
        if self.open_index.is_some() {
            self.open_at(next, window, cx);
        } else {
            self.focus_index = next;
            cx.notify();
        }
    }
}

impl Render for Menubar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut bar = h_flex()
            .id("menubar")
            .w_full()
            .items_center()
            .gap_1()
            .px_3()
            .py_1()
            .bg(semantic::surface(cx))
            .border_b_1()
            .border_color(semantic::border(cx))
            .on_action(cx.listener(|this, _: &menu::SelectPrevious, window, cx| {
                this.move_focus(-1, window, cx);
            }))
            .on_action(cx.listener(|this, _: &menu::SelectNext, window, cx| {
                this.move_focus(1, window, cx);
            }));

        for (i, item) in self.items.iter().enumerate() {
            let label = item.label.clone();
            let menu = item.menu.clone();
            let handle = item.handle.clone();
            let is_open = self.open_index == Some(i);
            let is_focused = self.focus_index == i;

            bar = bar.child(
                PopoverMenu::new(("menubar", i))
                    .attach(Anchor::BottomLeft)
                    .with_handle(handle)
                    .menu(move |_, _| Some(menu.clone()))
                    .trigger(
                        Button::new(("menubar-trigger", i), label)
                            .style(ButtonStyle::Transparent)
                            .toggle_state(is_open)
                            .when(is_focused, |this| {
                                this.color(Color::Custom(palette::primary(600)))
                            }),
                    ),
            );
        }

        bar
    }
}

impl Component for Menubar {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some("A horizontal menubar with dropdown menus and cross-menu keyboard navigation.")
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let file_menu = ContextMenu::build(window, cx, |this, _, _| {
            this.entry("New Tab", None, |_, _| {})
                .entry("New Window", None, |_, _| {})
                .separator()
                .entry("Close Tab", None, |_, _| {})
        });
        let edit_menu = ContextMenu::build(window, cx, |this, _, _| {
            this.entry("Undo", None, |_, _| {})
                .entry("Redo", None, |_, _| {})
                .separator()
                .entry("Cut", None, |_, _| {})
                .entry("Copy", None, |_, _| {})
                .entry("Paste", None, |_, _| {})
        });
        let view_menu = ContextMenu::build(window, cx, |this, _, _| {
            this.entry("Sidebar", None, |_, _| {})
                .entry("Panel", None, |_, _| {})
        });

        Some(
            cx.new(|cx| {
                Menubar::new(cx)
                    .item("File", file_menu)
                    .item("Edit", edit_menu)
                    .item("View", view_menu)
            })
            .into_any_element(),
        )
    }
}
