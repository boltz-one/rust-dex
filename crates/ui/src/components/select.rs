use std::{cell::Cell, rc::Rc};

use gpui::{Bounds, Context, Pixels, Render, anchored, canvas, deferred, point};

use crate::prelude::*;

/// A dropdown select: a trigger styled like a text field plus an expandable
/// option list. Stateful view — create with `cx.new(|_| Select::new(options))`.
pub struct Select {
    options: Vec<SharedString>,
    selected: Option<usize>,
    open: bool,
    placeholder: SharedString,
    /// Real screen bounds of the trigger row, captured via an invisible
    /// `canvas()` measurement child every render (regardless of `open`
    /// state) and read back on the *next* render to position the floating
    /// option list. One-render-frame lag, same idiom as
    /// `ContextMenu::submenu_trigger_bounds` (see
    /// `crates/ui/src/components/context_menu.rs`); converges before the
    /// list is ever visible to the user since the trigger renders (and thus
    /// measures) on every frame, open or closed.
    trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl Select {
    pub fn new(options: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Self {
            options: options.into_iter().map(Into::into).collect(),
            selected: None,
            open: false,
            placeholder: "Select…".into(),
            trigger_bounds: Rc::new(Cell::new(None)),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    /// The currently selected option text, if any.
    pub fn value(&self) -> Option<&SharedString> {
        self.selected.and_then(|i| self.options.get(i))
    }
}

impl Render for Select {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label: SharedString = self
            .selected
            .and_then(|i| self.options.get(i).cloned())
            .unwrap_or_else(|| self.placeholder.clone());
        let has_value = self.selected.is_some();

        let trigger = h_flex()
            .id("select-trigger")
            // Test-only (no-op in release builds, per `debug_selector`'s own
            // doc comment): lets integration tests locate the trigger's real
            // rendered pixel bounds via `VisualTestContext::debug_bounds`.
            // Mirrors the existing `Tab`/`SegmentedControl` precedent.
            .debug_selector(|| "SELECT-TRIGGER".into())
            .w_full()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(semantic::surface(cx))
            .border_1()
            .border_color(if self.open {
                palette::primary(500)
            } else {
                semantic::border(cx)
            })
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.open = !this.open;
                cx.notify();
            }))
            .child(Label::new(label).color(if has_value {
                Color::Default
            } else {
                Color::Placeholder
            }))
            .child(Icon::new(IconName::ChevronDown).size(IconSize::Small))
            .child({
                let trigger_bounds = self.trigger_bounds.clone();
                canvas(
                    move |bounds, _window, _cx| trigger_bounds.set(Some(bounds)),
                    |_bounds, _state, _window, _cx| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full()
            });

        let trigger_width = px(240.);

        v_flex()
            .w(trigger_width)
            .gap_1()
            .child(trigger)
            .when(self.open, |this| {
                let hover = semantic::hover_bg(cx);
                let mut list = v_flex()
                    .w(trigger_width)
                    .p_1()
                    .rounded_md()
                    .bg(semantic::elevated_surface(cx))
                    .border_1()
                    .border_color(semantic::border(cx))
                    .shadow_level(Shadow::Lg);
                for (i, option) in self.options.iter().enumerate() {
                    let option = option.clone();
                    list = list.child(
                        h_flex()
                            .id(("select-option", i))
                            .debug_selector(move || format!("SELECT-OPTION-{i}"))
                            .w_full()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected = Some(i);
                                this.open = false;
                                cx.notify();
                            }))
                            .child(Label::new(option)),
                    );
                }

                // Float the list in a `deferred` overlay pass, anchored just
                // below the trigger's real (previous-frame) bounds, instead
                // of an inline flow child — so it never pushes sibling
                // content down. Same idiom as `PopoverMenu`/`ContextMenu`
                // (`crates/ui/src/components/popover_menu.rs`,
                // `crates/ui/src/components/context_menu.rs`).
                let mut anchor = anchored().snap_to_window_with_margin(px(8.));
                if let Some(bounds) = self.trigger_bounds.get() {
                    anchor = anchor.position(point(
                        bounds.origin.x,
                        bounds.origin.y + bounds.size.height + px(4.),
                    ));
                }
                let floating_list = deferred(
                    anchor.child(
                        div()
                            .occlude()
                            .debug_selector(|| "SELECT-LIST".into())
                            .child(list),
                    ),
                )
                .with_priority(1);
                this.child(floating_list)
            })
    }
}
