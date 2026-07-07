//! Minimal, raw-`gpui` replacements for `ui::{Checkbox, CopyButton,
//! ScrollAxes, Scrollbars, Tooltip, WithScrollbar}`-style components. This
//! crate cannot depend on `boltz-ui` (`boltz-ui` depends on `boltz-markdown`,
//! so a `boltz-ui` dependency here would be a cycle), so these are
//! deliberately thin: just enough for `entity.rs`/`element.rs` to render
//! checkboxes, a code-block copy button, hover tooltips, and scrollable
//! containers. They are not meant to visually match `boltz-ui`'s richer
//! components pixel-for-pixel.

use gpui::{
    AnyView, App, AppContext as _, ClickEvent, ClipboardItem, Div, ElementId, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px, rgb,
};
use icons::IconName;

/// A minimal checkbox: a bordered square that renders a checkmark glyph when
/// `checked`, and invokes `on_click` (toggling is the caller's
/// responsibility — this has no internal state, matching every other
/// stateless-config component in this crate) on click.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            disabled: false,
            on_click: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `on_click` receives the checkbox's *new* intended state (`!checked`).
    pub fn on_click(mut self, on_click: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let disabled = self.disabled;
        let on_click = self.on_click;

        div()
            .id(self.id)
            .size(px(14.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_xs()
            .border_1()
            .border_color(rgb(0x4A5568))
            .when(checked, |this| this.bg(rgb(0x3B82F6)))
            .when(!disabled, |this| this.cursor_pointer())
            .when_some(on_click.filter(|_| !disabled), |this, on_click| {
                this.on_click(move |_: &ClickEvent, window, cx| {
                    on_click(&!checked, window, cx);
                })
            })
            .when(checked, |this| {
                this.child(
                    div()
                        .size(px(8.))
                        .text_color(rgb(0xFFFFFF))
                        .child(SharedString::from("\u{2713}")),
                )
            })
    }
}

/// A small icon button that copies `text` to the clipboard on click. No
/// internal "copied" state (that would require this to be a stateful
/// `Entity`) — callers that want a "copied!" flash can track that externally
/// and swap `icon`/`tooltip`.
#[derive(IntoElement)]
pub struct CopyButton {
    id: ElementId,
    text: SharedString,
    icon: IconName,
}

impl CopyButton {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon: IconName::Copy,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }
}

impl RenderOnce for CopyButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let text = self.text;

        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .size(px(20.))
            .rounded_xs()
            .cursor_pointer()
            .hover(|this| this.bg(rgb(0x2D3748)))
            .child(icon_svg(self.icon, px(14.), rgb(0xA0AEC0)))
            .on_click(move |_: &ClickEvent, _window, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            })
    }
}

/// Renders a `boltz-icons` glyph via `gpui::svg()` directly — this crate
/// deliberately avoids `boltz-ui`'s `Icon` component (see module docs).
pub fn icon_svg(
    icon: IconName,
    size: impl Into<gpui::Pixels>,
    color: impl Into<gpui::Hsla>,
) -> gpui::Svg {
    gpui::svg()
        .path(icon.path())
        .size(size.into())
        .text_color(color.into())
}

/// A minimal hover tooltip carrying a single line of text, built the same
/// way `ui::Tooltip::text` does (`cx.new(|_| ...).into()`) but without the
/// `KeyBinding`/`meta` extras that would require depending on `boltz-menu`.
struct SimpleTooltip {
    text: SharedString,
}

impl gpui::Render for SimpleTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .rounded_xs()
            .bg(rgb(0x1A202C))
            .text_color(rgb(0xE2E8F0))
            .px_2()
            .py_1()
            .text_sm()
            .child(self.text.clone())
    }
}

/// Builds a `.tooltip(...)` callback showing `text` on hover, usable as
/// `div().tooltip(simple_tooltip("Copy"))`.
pub fn simple_tooltip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let text = text.into();
    move |_window, cx| cx.new(|_| SimpleTooltip { text: text.clone() }).into()
}

/// Which axes a scrollable container should scroll along. Shaped closely
/// enough to `ui::ScrollAxes` for `element.rs`/`rendered.rs` to swap in
/// later, but only wraps gpui's own `overflow_{x,y}_scroll` + `track_scroll`
/// (no custom scrollbar thumb/track rendering: `boltz-gpui` exposes no
/// primitive lower-level than `boltz-ui`'s `Scrollbars` for that, so this is
/// a minimal passthrough fallback).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxes {
    Vertical,
    Horizontal,
    Both,
}

/// Minimal `WithScrollbar` replacement: applies native `overflow_scroll` +
/// `track_scroll` for the requested axes. Does not render a visible
/// scrollbar thumb/track (see [`ScrollAxes`] docs). Takes a `Stateful<Div>`
/// (i.e. after `.id(...)`) because gpui only implements
/// `StatefulInteractiveElement` (which provides `overflow_*_scroll`/
/// `track_scroll`) for stateful elements, not bare `Div`.
pub trait WithScrollbar {
    fn with_scrollbar(self, handle: &gpui::ScrollHandle, axes: ScrollAxes) -> Self;
}

impl WithScrollbar for gpui::Stateful<Div> {
    fn with_scrollbar(self, handle: &gpui::ScrollHandle, axes: ScrollAxes) -> Self {
        let this = match axes {
            ScrollAxes::Vertical => self.overflow_y_scroll(),
            ScrollAxes::Horizontal => self.overflow_x_scroll(),
            ScrollAxes::Both => self.overflow_scroll(),
        };
        this.track_scroll(handle)
    }
}
