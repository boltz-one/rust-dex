use gpui::{Context, Entity, Render, Window};
use theme::{Appearance, SystemAppearance};
use ui::prelude::*;

use crate::pages;

/// Component groups shown in the gallery sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryPage {
    Elements,
    Forms,
    Feedback,
    Navigation,
    Data,
    Overlays,
    Layout,
}

impl GalleryPage {
    fn title(self) -> &'static str {
        match self {
            GalleryPage::Elements => "Elements",
            GalleryPage::Forms => "Forms",
            GalleryPage::Feedback => "Feedback",
            GalleryPage::Navigation => "Navigation",
            GalleryPage::Data => "Data",
            GalleryPage::Overlays => "Overlays",
            GalleryPage::Layout => "Layout",
        }
    }
}

const PAGES: [GalleryPage; 7] = [
    GalleryPage::Elements,
    GalleryPage::Forms,
    GalleryPage::Feedback,
    GalleryPage::Navigation,
    GalleryPage::Data,
    GalleryPage::Overlays,
    GalleryPage::Layout,
];

/// A single toast entry owned by the gallery (mirrors `ToastStack`'s
/// caller-owns-the-list contract), driving the Overlays page's "Show toast" /
/// dismiss demo with real state instead of a static mock.
pub(crate) struct ToastItem {
    pub(crate) id: u64,
    pub(crate) severity: Severity,
    pub(crate) heading: SharedString,
    pub(crate) description: SharedString,
}

/// The root gallery view: a sidebar of component groups plus a content area
/// showing the selected group's showcase.
pub struct GalleryApp {
    pub(crate) page: GalleryPage,
    pub(crate) text_input: Entity<TextInput>,
    pub(crate) textarea: Entity<TextInput>,
    pub(crate) select: Entity<Select>,
    pub(crate) modal_open: bool,
    pub(crate) toasts: Vec<ToastItem>,
    pub(crate) next_toast_id: u64,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            page: GalleryPage::Elements,
            text_input: cx.new(|cx| TextInput::new(cx).placeholder("you@example.com")),
            textarea: cx.new(|cx| {
                TextInput::new(cx)
                    .multiline(true)
                    .placeholder("Your message…")
            }),
            select: cx.new(|_| Select::new(["Low", "Medium", "High"]).placeholder("Priority")),
            modal_open: false,
            toasts: Vec::new(),
            next_toast_id: 0,
        }
    }
}

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.page;
        let is_light = SystemAppearance::global(cx).0 == Appearance::Light;

        let mut sidebar = Sidebar::new();
        for page in PAGES {
            sidebar = sidebar.child(
                SidebarItem::new(page.title(), page.title())
                    .active(current == page)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.page = page;
                        cx.notify();
                    })),
            );
        }

        let content = match current {
            GalleryPage::Elements => pages::elements::render(window, cx),
            GalleryPage::Forms => self.render_forms(window, cx),
            GalleryPage::Feedback => pages::feedback::render(window, cx),
            GalleryPage::Navigation => pages::navigation::render(window, cx),
            GalleryPage::Data => pages::data::render(window, cx),
            GalleryPage::Overlays => self.render_overlays(window, cx),
            GalleryPage::Layout => pages::layout::render(window, cx),
        };

        h_flex()
            .size_full()
            .bg(semantic::background(cx))
            .text_color(semantic::text(cx))
            .child(sidebar)
            .child(
                v_flex()
                    .flex_1()
                    .child(
                        Navbar::new()
                            .trailing(
                                Button::new(
                                    "theme-toggle",
                                    if is_light { "Dark mode" } else { "Light mode" },
                                )
                                .on_click(cx.listener(
                                    |_, _, _, cx| {
                                        let next = if SystemAppearance::global(cx).0
                                            == Appearance::Light
                                        {
                                            Appearance::Dark
                                        } else {
                                            Appearance::Light
                                        };
                                        theme::set_appearance(next, cx);
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(Label::new(current.title().to_string())),
                    )
                    .child(v_flex().flex_1().p_6().gap_8().child(content)),
            )
    }
}
