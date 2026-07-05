use gpui::{AnyElement, Context, Entity, Render, rgb};

use crate::prelude::*;
use crate::TextInput;

/// Fixed line-number gutter width.
const GUTTER_WIDTH: Pixels = px(44.);
/// Monospace font used for code content.
const CODE_FONT_FAMILY: &str = "IBM Plex Mono";
/// Code text size.
const CODE_FONT_SIZE: Pixels = px(12.5);
/// Code line height (relative).
const CODE_LINE_HEIGHT: f32 = 1.7;

/// A multi-line code area with a line-number gutter, composed from a
/// multiline `TextInput`. No syntax highlighting — full highlighting
/// (grammar integration, incremental parsing, theme mapping) is a
/// materially larger, separately-scoped effort.
///
/// Stateful view — create with `cx.new(|cx| CodeEditor::new(cx))` and store
/// the resulting `Entity<CodeEditor>`.
pub struct CodeEditor {
    input: Entity<TextInput>,
}

impl CodeEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx).multiline(true));
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self { input }
    }

    /// Toggles read-only mode (e.g. for a read-only code preview that reuses
    /// this component rather than a separate one).
    /// Forwards the flag to the wrapped `TextInput` — no key-handling logic
    /// is duplicated here; the `TextInput`'s own `read_only` flag remains the
    /// single source of truth.
    pub fn read_only(mut self, cx: &mut Context<Self>, read_only: bool) -> Self {
        self.set_read_only(read_only, cx);
        self
    }

    /// Dynamically toggles read-only mode after construction.
    pub fn set_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.set_read_only(read_only, cx));
        cx.notify();
    }

    /// The current code content.
    pub fn text(&self, cx: &App) -> String {
        self.input.read(cx).text().to_string()
    }

    /// Programmatically sets the code content (e.g. loading content into a
    /// read-only preview).
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.set_text(text, cx));
        cx.notify();
    }
}

impl Render for CodeEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let line_count = self.input.read(cx).text().matches('\n').count() + 1;

        let gutter = v_flex()
            .flex_none()
            .w(GUTTER_WIDTH)
            .pr(px(16.))
            .text_right()
            // Gutter line-number text color.
            .text_color(rgb(0x3A424E))
            .font_family(CODE_FONT_FAMILY)
            .text_size(CODE_FONT_SIZE)
            .line_height(relative(CODE_LINE_HEIGHT))
            .children((1..=line_count).map(|line| div().child(line.to_string())));

        h_flex()
            .id(("code-editor", cx.entity_id()))
            .w_full()
            .items_start()
            .overflow_y_scroll()
            .child(gutter)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    // Code text color.
                    .text_color(rgb(0xB7BEC7))
                    .font_family(CODE_FONT_FAMILY)
                    .text_size(CODE_FONT_SIZE)
                    .line_height(relative(CODE_LINE_HEIGHT))
                    .child(self.input.clone()),
            )
    }
}

/// Standalone gallery preview for `CodeEditor` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching
/// `SearchInput`'s existing convention in this crate).
pub fn code_editor_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_4()
        .child(cx.new(|cx| CodeEditor::new(cx)))
        .into_any_element()
}
