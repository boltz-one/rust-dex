use gpui::{AnyElement, App, Context, Entity, Render, Window};
use theme::{Appearance, SystemAppearance};
use ui::prelude::*;

/// Component groups shown in the gallery sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryPage {
    Elements,
    Forms,
    Feedback,
    Navigation,
}

impl GalleryPage {
    fn title(self) -> &'static str {
        match self {
            GalleryPage::Elements => "Elements",
            GalleryPage::Forms => "Forms",
            GalleryPage::Feedback => "Feedback",
            GalleryPage::Navigation => "Navigation",
        }
    }
}

const PAGES: [GalleryPage; 4] = [
    GalleryPage::Elements,
    GalleryPage::Forms,
    GalleryPage::Feedback,
    GalleryPage::Navigation,
];

/// The root gallery view: a sidebar of component groups plus a content area
/// showing the selected group's showcase.
pub struct GalleryApp {
    page: GalleryPage,
    text_input: Entity<TextInput>,
    textarea: Entity<TextInput>,
    select: Entity<Select>,
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
        }
    }

    fn render_forms(&self) -> AnyElement {
        v_flex()
            .gap_6()
            .w(px(360.))
            .child(field("Email", self.text_input.clone().into_any_element()))
            .child(field("Message", self.textarea.clone().into_any_element()))
            .child(field("Priority", self.select.clone().into_any_element()))
            .child(field(
                "Plan",
                v_flex()
                    .gap_2()
                    .child(RadioButton::new("plan-free").label("Free").checked(true))
                    .child(RadioButton::new("plan-pro").label("Pro"))
                    .into_any_element(),
            ))
            .child(field(
                "Preferences",
                v_flex()
                    .gap_2()
                    .child(Checkbox::new("chk-updates", ToggleState::Selected))
                    .child(Checkbox::new("chk-marketing", ToggleState::Unselected))
                    .into_any_element(),
            ))
            .child(field(
                "Notifications",
                h_flex()
                    .gap_3()
                    .child(Switch::new("sw-on", ToggleState::Selected))
                    .child(Switch::new("sw-off", ToggleState::Unselected))
                    .into_any_element(),
            ))
            .into_any_element()
    }
}

fn field(label: &str, control: AnyElement) -> AnyElement {
    v_flex()
        .gap_1()
        .child(Label::new(label.to_string()).size(LabelSize::Small))
        .child(control)
        .into_any_element()
}

fn section(title: &str, body: Option<AnyElement>) -> AnyElement {
    v_flex()
        .gap_3()
        .child(Label::new(title.to_string()).size(LabelSize::Large))
        .children(body)
        .into_any_element()
}

fn render_static_page(page: GalleryPage, window: &mut Window, cx: &mut App) -> AnyElement {
    match page {
        GalleryPage::Elements => v_flex()
            .gap_8()
            .child(section("Buttons", Button::preview(window, cx)))
            .child(section("Badges", Badge::preview(window, cx)))
            .child(section("Cards", Card::preview(window, cx)))
            .into_any_element(),
        GalleryPage::Feedback => v_flex()
            .gap_8()
            .child(section("Alerts", Alert::preview(window, cx)))
            .into_any_element(),
        GalleryPage::Navigation => v_flex()
            .gap_8()
            .child(section("Navbar", Navbar::preview(window, cx)))
            .child(section("Sidebar", Sidebar::preview(window, cx)))
            .into_any_element(),
        GalleryPage::Forms => div().into_any_element(),
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

        let content = if current == GalleryPage::Forms {
            self.render_forms()
        } else {
            render_static_page(current, window, cx)
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
