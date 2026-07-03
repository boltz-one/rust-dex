use gpui::AnyElement;
use smallvec::SmallVec;

use crate::prelude::*;

/// A fixed top bar: leading content (title/logo) on the left, optional trailing
/// actions on the right. Neutrals are theme-driven.
#[derive(IntoElement, RegisterComponent)]
pub struct Navbar {
    children: SmallVec<[AnyElement; 2]>,
    trailing: Option<AnyElement>,
}

impl Navbar {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            trailing: None,
        }
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }
}

impl Default for Navbar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Navbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Navbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .px_6()
            .py_4()
            .bg(semantic::surface(cx))
            .border_b_1()
            .border_color(semantic::border(cx))
            .shadow_level(Shadow::Sm)
            .child(h_flex().items_center().gap_3().children(self.children))
            .children(self.trailing)
    }
}

impl Component for Navbar {
    fn scope() -> ComponentScope {
        ComponentScope::Navigation
    }

    fn description() -> Option<&'static str> {
        Some("A fixed top bar with leading content and optional trailing actions.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .child(
                    Navbar::new()
                        .trailing(Badge::new("v1.4.2").color(BadgeColor::Primary))
                        .child(Label::new("Application")),
                )
                .child(
                    Navbar::new()
                        .trailing(
                            h_flex()
                                .gap_2()
                                .child(Button::new("navbar-search", "Search"))
                                .child(Button::new("navbar-invite", "Invite").primary()),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(Icon::new(IconName::Folder).size(IconSize::Small))
                                .child(Label::new("Acme Workspace")),
                        ),
                )
                .into_any_element(),
        )
    }
}
