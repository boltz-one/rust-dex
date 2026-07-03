use gpui::AnyElement;

use crate::prelude::*;

/// Composes an already-styled [`Navbar`] (top) and [`Sidebar`] (left) around a
/// main content slot. Pure composition — no new styling logic; `Navbar` and
/// `Sidebar` own all of their own visuals (colors/borders/width).
#[derive(IntoElement, RegisterComponent)]
pub struct AppShell {
    navbar: Option<AnyElement>,
    sidebar: Option<AnyElement>,
    content: Option<AnyElement>,
}

impl AppShell {
    pub fn new() -> Self {
        Self {
            navbar: None,
            sidebar: None,
            content: None,
        }
    }

    pub fn navbar(mut self, navbar: impl IntoElement) -> Self {
        self.navbar = Some(navbar.into_any_element());
        self
    }

    pub fn sidebar(mut self, sidebar: impl IntoElement) -> Self {
        self.sidebar = Some(sidebar.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

impl Default for AppShell {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for AppShell {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        v_flex().w_full().h_full().children(self.navbar).child(
            h_flex()
                .flex_1()
                .items_stretch()
                .children(self.sidebar)
                .child(div().flex_1().h_full().children(self.content)),
        )
    }
}

impl Component for AppShell {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some("Composes a Navbar (top) and Sidebar (left) around a main content slot.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            div()
                .h(px(320.))
                .child(
                    AppShell::new()
                        .navbar(Navbar::new().child(Label::new("Application")))
                        .sidebar(
                            Sidebar::new()
                                .child(SidebarItem::new("shell-home", "Home").active(true))
                                .child(SidebarItem::new("shell-settings", "Settings")),
                        )
                        .content(div().p_6().child(Label::new("Main content area"))),
                )
                .into_any_element(),
        )
    }
}
