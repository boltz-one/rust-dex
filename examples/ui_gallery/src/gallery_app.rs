use gpui::{Context, Entity, Render, ScrollHandle, Window};
use theme::{Appearance, SystemAppearance};
use ui::prelude::*;
use ui::{
    Calendar, Carousel, Combobox, Command, CommandItem, ContextMenu, DatePicker, InputOtp, Menubar,
    MultiSelect, NavigationMenu, NavigationMenuItemDef, NavigationMenuSubItem, ResizablePanelGroup,
    SearchInput, SonnerStack, TabSwitcher, TabSwitcherItem, TerminalView,
};

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
    Examples,
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
            GalleryPage::Examples => "Examples",
        }
    }
}

pub(crate) const PAGES: [GalleryPage; 8] = [
    GalleryPage::Elements,
    GalleryPage::Forms,
    GalleryPage::Feedback,
    GalleryPage::Navigation,
    GalleryPage::Data,
    GalleryPage::Overlays,
    GalleryPage::Layout,
    GalleryPage::Examples,
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
    /// Scroll offset for the main content area; persisted across frames so
    /// scrolling any page actually moves the viewport instead of resetting.
    pub(crate) scroll: ScrollHandle,
    /// Active segment index for the Forms page's `SegmentedControl` demo.
    pub(crate) forms_segment: usize,
    pub(crate) multi_select: Entity<MultiSelect>,
    pub(crate) combobox: Entity<Combobox>,
    pub(crate) search_input: Entity<SearchInput>,
    /// Active status-filter segment for the Examples page's table+toolbar
    /// demo (0 = All, 1 = Active, 2 = Archived).
    pub(crate) examples_status_filter: usize,
    /// Whether the Examples page's settings-form demo was last "saved" (vs.
    /// cancelled/untouched); drives a visible Badge confirmation.
    pub(crate) examples_settings_saved: bool,
    /// Active tab index for the Navigation page's `TabBar`/`Tab` demo.
    pub(crate) nav_tab: usize,
    /// Command palette demo (Overlays page). Created once here — never in a
    /// `preview()`/render body — so typed queries and selection persist.
    pub(crate) command: Entity<Command>,
    /// Tab switcher demo (Overlays page). Created once here — same reason as
    /// `command` above — so ↑/↓ selection persists across re-renders instead
    /// of resetting to index 0 every time `GalleryApp` re-renders for an
    /// unrelated reason (e.g. the theme toggle).
    pub(crate) tab_switcher: Entity<TabSwitcher>,
    /// Real PTY terminal demo (Layout page). Created once here — unlike
    /// every other `Entity` field, recreating this one on each render would
    /// not just lose UI state but spawn a brand-new real shell child process
    /// every time `GalleryApp` re-renders for an unrelated reason (e.g. the
    /// theme toggle), since `TerminalView::new` spawns a PTY. `Drop` on the
    /// old entity does clean up its shell, so this wouldn't literally leak
    /// processes — but it would repeatedly kill and respawn a real shell,
    /// which is real waste for zero benefit.
    pub(crate) terminal_view: Entity<TerminalView>,
    /// OTP input demo. Created once here so slot focus/typed digits persist
    /// across re-renders (see `Forms` page usage of this component pattern).
    pub(crate) input_otp: Entity<InputOtp>,
    /// Calendar demo (Layout page) — created once so the selected day and
    /// visible month persist across re-renders.
    pub(crate) calendar: Entity<Calendar>,
    /// Date picker demo (Layout page) — created once so open/closed state
    /// and the picked date persist across re-renders.
    pub(crate) date_picker: Entity<DatePicker>,
    /// Carousel demo (Layout page) — created once so the active slide
    /// persists across re-renders.
    pub(crate) carousel: Entity<Carousel>,
    /// Navigation menu demo (Navigation + Data pages) — created once so the
    /// open submenu persists across re-renders.
    pub(crate) nav_menu: Entity<NavigationMenu>,
    /// Sonner-style toast queue demo (Overlays page) — created once so
    /// queued/dismissed toasts persist across re-renders.
    pub(crate) sonner: Entity<SonnerStack>,
    /// Resizable split-panel demo (Layout + Data pages) — created once so
    /// the dragged split fraction persists across re-renders.
    pub(crate) resizable: Entity<ResizablePanelGroup>,
    /// Menubar demo (Navigation + Overlays pages). Unlike the other entities
    /// above, this can't be created in `new()`: its dropdown `ContextMenu`s
    /// require `&mut Window` (via `ContextMenu::build`), and `GalleryApp::new`
    /// only receives `&mut Context<Self>`. It is instead created lazily on
    /// first render via `ensure_menubar` — the `Option` guard means it is
    /// still only ever constructed once, not on every frame, which is what
    /// actually fixes the recreate-per-render bug for this component.
    pub(crate) menubar: Option<Entity<Menubar>>,
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
            select: cx.new(|_| {
                Select::new(["Low", "Medium", "High", "Urgent", "Critical"]).placeholder("Priority")
            }),
            modal_open: false,
            toasts: Vec::new(),
            next_toast_id: 0,
            scroll: ScrollHandle::new(),
            forms_segment: 0,
            multi_select: cx.new(|_| {
                MultiSelect::new(["Design", "Engineering", "Marketing", "Sales", "Support"])
                    .selected_indices([0, 2])
            }),
            combobox: cx
                .new(|cx| Combobox::new(cx, ["Apple", "Banana", "Cherry", "Date", "Elderberry"])),
            search_input: cx.new(|cx| SearchInput::new(cx, "Search…")),
            examples_status_filter: 0,
            examples_settings_saved: false,
            nav_tab: 0,
            command: cx.new(|cx| {
                Command::new(
                    cx,
                    vec![
                        CommandItem::Group("Suggestions".into()),
                        CommandItem::Entry {
                            id: "cal".into(),
                            label: "Calendar".into(),
                        },
                        CommandItem::Entry {
                            id: "sr".into(),
                            label: "Search Emoji".into(),
                        },
                        CommandItem::Entry {
                            id: "calc".into(),
                            label: "Calculator".into(),
                        },
                        CommandItem::Group("Settings".into()),
                        CommandItem::Entry {
                            id: "prof".into(),
                            label: "Profile".into(),
                        },
                        CommandItem::Entry {
                            id: "bill".into(),
                            label: "Billing".into(),
                        },
                    ],
                )
            }),
            tab_switcher: cx.new(|cx| {
                TabSwitcher::new(
                    cx,
                    vec![
                        TabSwitcherItem::new("main.rs", |_, _| {}).icon(IconName::File),
                        TabSwitcherItem::new("lib.rs", |_, _| {})
                            .icon(IconName::File)
                            .subtitle("crates/ui/src"),
                        TabSwitcherItem::new("Cargo.toml", |_, _| {}).icon(IconName::File),
                    ],
                )
            }),
            terminal_view: cx.new(|cx| TerminalView::new(cx)),
            input_otp: cx.new(|cx| InputOtp::new(cx, 6)),
            calendar: cx.new(|_| Calendar::new()),
            date_picker: cx.new(DatePicker::new),
            carousel: cx.new(|_| {
                Carousel::new([
                    ("Slide 1", palette::primary(100)),
                    ("Slide 2", palette::success(100)),
                    ("Slide 3", palette::warning(100)),
                ])
            }),
            nav_menu: cx.new(|_| {
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
            }),
            sonner: cx.new(SonnerStack::new),
            resizable: cx.new(|_| {
                ResizablePanelGroup::new(
                    |_, _| {
                        v_flex()
                            .gap_2()
                            .child(Label::new("Left panel").weight(gpui::FontWeight::SEMIBOLD))
                            .child(Label::new("Drag the handle to resize.").color(Color::Muted))
                            .into_any_element()
                    },
                    |_, _| {
                        v_flex()
                            .gap_2()
                            .child(Label::new("Right panel").weight(gpui::FontWeight::SEMIBOLD))
                            .child(Label::new("Clamped between 20% and 80%.").color(Color::Muted))
                            .into_any_element()
                    },
                )
                .min_left_fraction(0.2)
                .max_left_fraction(0.8)
            }),
            menubar: None,
        }
    }

    /// Lazily creates (once) and returns the shared `Menubar` demo entity.
    ///
    /// Building the menubar's dropdown `ContextMenu`s requires `&mut Window`
    /// (see `ContextMenu::build`), which `GalleryApp::new` does not receive —
    /// so this entity can't be constructed alongside the others above. The
    /// `Option` guard ensures it is still only ever created the first time
    /// this is called, never on subsequent re-renders, preserving open/closed
    /// menu state exactly like the entities created in `new()`.
    pub(crate) fn ensure_menubar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<Menubar> {
        if self.menubar.is_none() {
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
            self.menubar = Some(cx.new(|cx| {
                Menubar::new(cx)
                    .item("File", file_menu)
                    .item("Edit", edit_menu)
                    .item("View", view_menu)
            }));
        }
        self.menubar
            .clone()
            .expect("menubar was just initialized above")
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
            GalleryPage::Navigation => self.render_navigation(window, cx),
            GalleryPage::Data => self.render_data(window, cx),
            GalleryPage::Overlays => self.render_overlays(window, cx),
            GalleryPage::Layout => self.render_layout(window, cx),
            GalleryPage::Examples => self.render_examples(window, cx),
        };

        // NOTE: a plain flex-row (default `align-items: stretch`), NOT `h_flex()`
        // — `h_flex()` bakes in `.items_center()`, which sizes the sidebar/content
        // columns to their own content height and centers them, so the content
        // column never fills the window and its inner `flex_1().overflow_y_scroll()`
        // has nothing to overflow (scroll offset stayed pinned at 0). Stretching
        // the columns to full height is what makes the content area actually scroll.
        div()
            .flex()
            .flex_row()
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
                    .child(
                        v_flex()
                            .id("gallery-content")
                            .flex_1()
                            .p_6()
                            .gap_8()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .child(content),
                    ),
            )
    }
}
