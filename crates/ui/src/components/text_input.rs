use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, Bounds, Context, ElementInputHandler, EntityInputHandler, FocusHandle, Focusable,
    KeyDownEvent, MouseButton, Pixels, Render, UTF16Selection, canvas,
};

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
/// IME composition (e.g. Vietnamese/CJK input methods) is handled via the
/// [`EntityInputHandler`] impl, so composed text commits correctly.
///
/// This is a stateful view: create with `cx.new(|cx| TextInput::new(cx))` and
/// store the resulting `Entity<TextInput>`.
pub struct TextInput {
    content: String,
    placeholder: SharedString,
    focus_handle: FocusHandle,
    multiline: bool,
    submit_on_enter: bool,
    validation: InputValidationState,
    read_only: bool,
    /// Byte range of in-progress IME marked (composition) text within
    /// `content`, if any. `None` when no composition is active.
    marked_range: Option<Range<usize>>,
    on_submit: Option<Rc<dyn Fn(&mut Window, &mut Context<Self>) + 'static>>,
}

impl TextInput {
    pub fn new(cx: &mut App) -> Self {
        Self {
            content: String::new(),
            placeholder: SharedString::default(),
            focus_handle: cx.focus_handle(),
            multiline: false,
            submit_on_enter: false,
            validation: InputValidationState::Neutral,
            read_only: false,
            marked_range: None,
            on_submit: None,
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

    /// When `true`, plain Enter fires [`Self::on_submit`] instead of
    /// inserting a newline; Shift/Ctrl/Cmd+Enter still inserts a newline (in
    /// `multiline` mode). Defaults to `false`, which preserves the previous
    /// behavior of always inserting a newline on Enter in `multiline` mode
    /// (e.g. `CodeEditor`'s free-form multiline input).
    pub fn submit_on_enter(mut self, submit_on_enter: bool) -> Self {
        self.submit_on_enter = submit_on_enter;
        self
    }

    /// Registers a callback fired when the user presses plain Enter while
    /// [`Self::submit_on_enter`] is `true`. Has no effect otherwise.
    pub fn on_submit(
        mut self,
        handler: impl Fn(&mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
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

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.read_only {
            return;
        }
        let keystroke = &event.keystroke;

        if self.submit_on_enter && keystroke.key == "enter" {
            let wants_newline = self.multiline
                && (keystroke.modifiers.shift
                    || keystroke.modifiers.control
                    || keystroke.modifiers.platform);
            if wants_newline {
                self.content.push('\n');
            } else if let Some(on_submit) = self.on_submit.clone() {
                on_submit(window, cx);
            }
            cx.notify();
            return;
        }

        // Ignore keyboard shortcuts (cmd/ctrl chords) — only capture text input.
        if keystroke.modifiers.control || keystroke.modifiers.platform {
            return;
        }
        // Printable text (including spaces and IME-composed characters) is
        // committed through the `EntityInputHandler` impl via
        // `replace_text_in_range` — appending `key_char` here too would double
        // every character. `on_key_down` only owns editing keys that the input
        // handler doesn't synthesize: backspace and (in multiline mode) newline.
        match keystroke.key.as_str() {
            "backspace" => {
                if let Some(range) = self.marked_range.take() {
                    self.content.replace_range(range, "");
                } else {
                    self.content.pop();
                }
            }
            "enter" if self.multiline => self.content.push('\n'),
            _ => {}
        }
        cx.notify();
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let is_empty = self.content.is_empty();
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
        let show_cursor = focused && !is_empty && !self.read_only;
        let cursor = || div().w(px(1.)).h(px(16.)).bg(palette::primary(500));

        // Multiline content is split and rendered one row per line (rather
        // than a single text child carrying embedded `\n`s) so each typed
        // newline reliably produces a new visual row, with the blinking
        // caret appended to the last row.
        let content: AnyElement = if self.multiline {
            let text: SharedString = if is_empty {
                self.placeholder.clone()
            } else {
                self.content.clone().into()
            };
            let lines: Vec<String> = text.split('\n').map(str::to_string).collect();
            let last_ix = lines.len().saturating_sub(1);
            v_flex()
                .w_full()
                .children(lines.into_iter().enumerate().map(|(ix, line)| {
                    h_flex()
                        .min_h(px(20.))
                        .items_center()
                        .gap(DynamicSpacing::Base02.rems(cx))
                        .child(SharedString::from(line))
                        .when(ix == last_ix && show_cursor, |this| this.child(cursor()))
                }))
                .into_any_element()
        } else {
            let display: SharedString = if is_empty {
                self.placeholder.clone()
            } else {
                self.content.clone().into()
            };
            h_flex()
                .flex_wrap()
                .items_center()
                .gap(DynamicSpacing::Base02.rems(cx))
                .child(display)
                .when(show_cursor, |this| this.child(cursor()))
                .into_any_element()
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
            .px(DynamicSpacing::Base12.px(cx))
            .py(DynamicSpacing::Base08.px(cx))
            .rounded_md()
            .bg(semantic::surface(cx))
            .border_1()
            .border_color(border_color)
            .text_color(text_color)
            .child(content)
            .child({
                // Register an IME input handler for this field so platform
                // input methods (Vietnamese/CJK IME, dead-key composition)
                // commit text through `EntityInputHandler` instead of being
                // dropped. `handle_input` only activates while focused.
                let focus_handle = self.focus_handle.clone();
                let entity = cx.entity();
                canvas(
                    move |_bounds, _window, _cx| {},
                    move |bounds, _state, window, cx| {
                        window.handle_input(
                            &focus_handle,
                            ElementInputHandler::new(bounds, entity.clone()),
                            cx,
                        );
                    },
                )
                .absolute()
                .size_full()
            });

        focus_ring(field, focused, ring_color)
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for TextInput {
    fn accepts_text_input(&self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        !self.read_only
    }

    /// No cursor/selection tracking — this is an append-only field. Returning
    /// `None` tells the platform there is no active selection (IME commits at
    /// the end).
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        None
    }

    /// The byte range of in-progress IME marked (composition) text.
    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    /// Returns the text in the given UTF-16 range (used by the platform to
    /// read back what's around the composition).
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let bytes = utf16_range_to_byte_range(&self.content, range_utf16)?;
        Some(self.content[bytes].to_string())
    }

    /// IME committed text (or a plain paste/insert). Replaces any in-progress
    /// marked range, otherwise appends at the end.
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.read_only {
            return;
        }
        if let Some(marked) = self.marked_range.take() {
            self.content.replace_range(marked, text);
        } else if let Some(range) = range {
            let bytes = utf16_range_to_byte_range(&self.content, range);
            if let Some(bytes) = bytes {
                self.content.replace_range(bytes, text);
            } else {
                self.content.push_str(text);
            }
        } else {
            self.content.push_str(text);
        }
        cx.notify();
    }

    /// IME composition in progress — replace the given range (or append) with
    /// `new_text` and mark it as the active composition range.
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.read_only {
            return;
        }
        let start_byte = if let Some(marked) = self.marked_range.clone() {
            self.content.replace_range(marked.clone(), new_text);
            marked.start
        } else if let Some(range) = range {
            let bytes = utf16_range_to_byte_range(&self.content, range);
            if let Some(bytes) = bytes {
                self.content.replace_range(bytes.clone(), new_text);
                bytes.start
            } else {
                self.content.push_str(new_text);
                self.content.len().saturating_sub(new_text.len())
            }
        } else {
            let start = self.content.len();
            self.content.push_str(new_text);
            start
        };
        let end_byte = start_byte + new_text.len();
        self.marked_range = Some(start_byte..end_byte);
        cx.notify();
    }

    /// Composition finalized/abandoned — drop the mark without changing text.
    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.marked_range.take().is_some() {
            cx.notify();
        }
    }

    /// Bounds for a UTF-16 range within the field, for IME candidate placement.
    /// Best-effort: returns the element bounds (candidates appear near the
    /// field, not pixel-perfect per-character).
    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        Some(element_bounds)
    }

    fn character_index_for_point(
        &mut self,
        _point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

/// Maps a UTF-16 code-unit range to a byte range within `text` (Rust strings
/// are UTF-8). Returns `None` if the range is out of bounds.
fn utf16_range_to_byte_range(text: &str, range_utf16: Range<usize>) -> Option<Range<usize>> {
    let mut start_byte = None;
    let mut end_byte = None;
    let mut utf16_index = 0usize;
    for (byte_idx, ch) in text.char_indices() {
        if start_byte.is_none() && utf16_index >= range_utf16.start {
            start_byte = Some(byte_idx);
        }
        utf16_index += ch.len_utf16();
        if end_byte.is_none() && utf16_index >= range_utf16.end {
            end_byte = Some(byte_idx + ch.len_utf8());
            break;
        }
    }
    let start = start_byte.unwrap_or(text.len());
    let end = end_byte.unwrap_or(text.len());
    if start <= end && start <= text.len() && end <= text.len() {
        Some(start..end)
    } else {
        None
    }
}

/// A multi-line text field. Construct with
/// `cx.new(|cx| Textarea::new(cx).multiline(true))`.
pub type Textarea = TextInput;
