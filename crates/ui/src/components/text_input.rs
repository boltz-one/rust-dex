use gpui::{Context, FocusHandle, KeyDownEvent, Render};

use crate::prelude::*;

/// A focusable single-line (or multi-line) text field backed by a real
/// `String` buffer. Keyboard input is handled via key events (`key_char` +
/// editing keys), so typed characters genuinely appear and backspace deletes.
///
/// This is a stateful view: create with `cx.new(|cx| TextInput::new(cx))` and
/// store the resulting `Entity<TextInput>`.
pub struct TextInput {
    content: String,
    placeholder: SharedString,
    focus_handle: FocusHandle,
    multiline: bool,
    invalid: bool,
}

impl TextInput {
    pub fn new(cx: &mut App) -> Self {
        Self {
            content: String::new(),
            placeholder: SharedString::default(),
            focus_handle: cx.focus_handle(),
            multiline: false,
            invalid: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// The current text content.
    pub fn text(&self) -> &str {
        &self.content
    }

    /// Programmatically sets the text content (e.g. `SearchInput`/`Combobox`
    /// setting the display text after a selection). Notifies for re-render.
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.content = text.into();
        cx.notify();
    }

    /// Clears the text content (e.g. `SearchInput`'s clear button). Notifies
    /// for re-render.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.content.clear();
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        // Ignore keyboard shortcuts (cmd/ctrl chords) — only capture text input.
        if keystroke.modifiers.control || keystroke.modifiers.platform {
            return;
        }
        match keystroke.key.as_str() {
            "backspace" => {
                self.content.pop();
            }
            "enter" if self.multiline => self.content.push('\n'),
            "space" => self.content.push(' '),
            _ => {
                if let Some(text) = &keystroke.key_char {
                    self.content.push_str(text);
                }
            }
        }
        cx.notify();
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let is_empty = self.content.is_empty();
        let display: SharedString = if is_empty {
            self.placeholder.clone()
        } else {
            self.content.clone().into()
        };
        let text_color = if is_empty {
            semantic::text_placeholder(cx)
        } else {
            semantic::text(cx)
        };
        let border_color = if self.invalid {
            palette::danger(500)
        } else {
            semantic::border(cx)
        };
        let ring_color = if self.invalid {
            palette::danger(500)
        } else {
            palette::primary(500)
        };

        let field = div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .w_full()
            .when(self.multiline, |this| this.min_h(px(96.)))
            .flex()
            .flex_wrap()
            .items_center()
            .gap_0p5()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(semantic::surface(cx))
            .border_1()
            .border_color(border_color)
            .text_color(text_color)
            .child(display)
            .when(focused && !is_empty, |this| {
                this.child(div().w(px(1.)).h(px(16.)).bg(palette::primary(500)))
            });

        focus_ring(field, focused, ring_color)
    }
}

/// A multi-line text field. Construct with
/// `cx.new(|cx| Textarea::new(cx).multiline(true))`.
pub type Textarea = TextInput;
