use gpui::{Context, FocusHandle, Focusable, KeyDownEvent, MouseButton, Render};

use crate::prelude::*;

/// Visual validation state of a [`TextInput`], reflected in its border/focus
/// ring color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputValidationState {
    #[default]
    Neutral,
    Error,
    Success,
    Warning,
}

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
    validation: InputValidationState,
    read_only: bool,
}

impl TextInput {
    pub fn new(cx: &mut App) -> Self {
        Self {
            content: String::new(),
            placeholder: SharedString::default(),
            focus_handle: cx.focus_handle(),
            multiline: false,
            validation: InputValidationState::Neutral,
            read_only: false,
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

    /// When `true`, the input no longer accepts keyboard edits (used for
    /// read-only code previews). Focus/selection styling is unaffected.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Sets the error validation state (red border/ring) when `invalid` is
    /// true, otherwise clears back to [`InputValidationState::Neutral`].
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.validation = if invalid {
            InputValidationState::Error
        } else {
            InputValidationState::Neutral
        };
        self
    }

    /// Sets the success validation state (green border/ring) when `success`
    /// is true, otherwise clears back to [`InputValidationState::Neutral`].
    pub fn success(mut self, success: bool) -> Self {
        self.validation = if success {
            InputValidationState::Success
        } else {
            InputValidationState::Neutral
        };
        self
    }

    /// Sets the warning validation state (amber border/ring) when `warning`
    /// is true, otherwise clears back to [`InputValidationState::Neutral`].
    pub fn warning(mut self, warning: bool) -> Self {
        self.validation = if warning {
            InputValidationState::Warning
        } else {
            InputValidationState::Neutral
        };
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

    /// Dynamically toggles read-only mode after construction (e.g.
    /// `CodeEditor` switching an already-created input between editable and
    /// preview modes). Notifies for re-render.
    pub fn set_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        self.read_only = read_only;
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.read_only {
            return;
        }
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
        let border_color = match self.validation {
            InputValidationState::Error => palette::danger(500),
            InputValidationState::Success => palette::success(500),
            InputValidationState::Warning => palette::warning(500),
            InputValidationState::Neutral => semantic::border(cx),
        };
        let ring_color = match self.validation {
            InputValidationState::Error => palette::danger(500),
            InputValidationState::Success => palette::success(500),
            InputValidationState::Warning => palette::warning(500),
            InputValidationState::Neutral => palette::primary(500),
        };

        let field = div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, window, cx| {
                    window.focus(&this.focus_handle, cx);
                    cx.notify();
                }),
            )
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
            .when(focused && !is_empty && !self.read_only, |this| {
                this.child(div().w(px(1.)).h(px(16.)).bg(palette::primary(500)))
            });

        focus_ring(field, focused, ring_color)
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// A multi-line text field. Construct with
/// `cx.new(|cx| Textarea::new(cx).multiline(true))`.
pub type Textarea = TextInput;
