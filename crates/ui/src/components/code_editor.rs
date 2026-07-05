use std::ops::Range;
use std::sync::LazyLock;

use gpui::{
    AnyElement, Context, Entity, Focusable, HighlightStyle, Render, StyledText, rgb, white,
};
use language_core::{DefaultLanguageRegistry, LanguageRegistry};

use crate::TextInput;
use crate::prelude::*;

/// Compiled once per process — grammars/queries are immutable after
/// construction, so every `CodeEditor` shares one registry instead of
/// re-parsing highlight queries per instance.
static LANGUAGES: LazyLock<DefaultLanguageRegistry> = LazyLock::new(DefaultLanguageRegistry::new);

/// Per-digit gutter width at `CODE_FONT_SIZE`/`CODE_FONT_FAMILY` (monospace,
/// ~0.6em advance per digit) — the gutter box grows with the line count
/// instead of clipping once line numbers exceed a fixed digit budget.
const GUTTER_DIGIT_WIDTH: Pixels = px(8.);
/// Gutter never shrinks below a 2-digit budget, so 1-9 line files don't get a
/// visually cramped near-zero-width gutter.
const GUTTER_MIN_DIGITS: usize = 2;
/// Gap between the line numbers and the divider/code content.
const GUTTER_RIGHT_PADDING: Pixels = px(16.);
/// Monospace font used for code content.
const CODE_FONT_FAMILY: &str = "IBM Plex Mono";
/// Code text size.
const CODE_FONT_SIZE: Pixels = px(12.5);
/// Code line height (relative).
const CODE_LINE_HEIGHT: f32 = 1.7;

/// Gutter width scaled to the number of digits in `line_count`.
fn gutter_width(line_count: usize) -> Pixels {
    let digits = line_count.max(1).to_string().len().max(GUTTER_MIN_DIGITS);
    GUTTER_DIGIT_WIDTH * digits as f32
}

/// Resolves a tree-sitter capture name (e.g. `"type.builtin"`) to a style,
/// falling back to its first dotted segment (`"type"`) if the full name
/// isn't in the theme. `SyntaxTheme::style_for_name` only does an exact
/// lookup — the dotted-prefix fallback it documents (`highlight_id`) is a
/// separate method that returns an index, not a style — so this reproduces
/// just enough of that fallback to avoid losing color on every capture a
/// grammar specializes beyond the theme's base vocabulary (confirmed by the
/// coverage tests below: `type.builtin`/`variable.parameter` in TypeScript
/// and `constant.builtin`/`string.special.key` in JSON all need this to
/// resolve against the One Dark theme's flatter capture list).
fn style_for_capture(syntax: &syntax_theme::SyntaxTheme, name: &str) -> Option<HighlightStyle> {
    syntax.style_for_name(name).or_else(|| {
        let prefix = name.split('.').next()?;
        (prefix != name)
            .then(|| syntax.style_for_name(prefix))
            .flatten()
    })
}

/// A multi-line code area with a line-number gutter, composed from a
/// multiline `TextInput`. Real tree-sitter syntax highlighting is available
/// via [`Self::language`], but only while [`Self::read_only`] is set — see
/// that method's doc comment for why live-typing highlighting is a
/// separately-scoped effort.
///
/// Stateful view — create with `cx.new(|cx| CodeEditor::new(cx))` and store
/// the resulting `Entity<CodeEditor>`.
pub struct CodeEditor {
    input: Entity<TextInput>,
    read_only: bool,
    language_extension: Option<&'static str>,
}

impl CodeEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx).multiline(true));
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self {
            input,
            read_only: false,
            language_extension: None,
        }
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
        self.read_only = read_only;
        self.input
            .update(cx, |input, cx| input.set_read_only(read_only, cx));
        cx.notify();
    }

    /// Enables real tree-sitter syntax highlighting for the given file
    /// extension (no leading dot, e.g. `"rs"`) — see
    /// [`language_core::DefaultLanguageRegistry`] for which extensions are
    /// compiled in. Highlighting only renders while [`Self::read_only`] is
    /// set: `TextInput` (which this component wraps for keystroke handling)
    /// has no cursor-position tracking beyond append-at-the-end (see the
    /// current-line-highlight note in `render` below), so there is no way to
    /// keep a caret visually correct inside syntax-highlighted rich text
    /// while the user is actively typing. Read-only code previews — the
    /// primary use case this component documents — get full highlighting;
    /// live editing still renders as plain colored text.
    pub fn language(mut self, extension: &'static str) -> Self {
        self.language_extension = Some(extension);
        self
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text = self.input.read(cx).text().to_string();
        let line_count = text.matches('\n').count() + 1;
        let line_height = CODE_FONT_SIZE * CODE_LINE_HEIGHT;

        // `TextInput` has no cursor-position tracking beyond "append at the
        // end" (see its `on_key_down`: backspace pops the last char, there is
        // no caret movement) — so the only line that can ever be "current" is
        // the last one. This highlight is an honest approximation of that,
        // not a real per-position caret; it only shows while focused so it
        // doesn't imply cursor state on an unfocused/read-only preview.
        let focused = self.input.read(cx).focus_handle(cx).is_focused(window);
        let current_line_top = line_height * (line_count - 1);

        let gutter = v_flex()
            .flex_none()
            .w(gutter_width(line_count))
            .pr(GUTTER_RIGHT_PADDING)
            .text_right()
            // Gutter line-number text color.
            .text_color(rgb(0x3A424E))
            .font_family(CODE_FONT_FAMILY)
            .text_size(CODE_FONT_SIZE)
            .line_height(relative(CODE_LINE_HEIGHT))
            .children((1..=line_count).map(|line| div().child(line.to_string())));

        div()
            .id(("code-editor", cx.entity_id()))
            .relative()
            .w_full()
            .overflow_y_scroll()
            .when(focused, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(current_line_top)
                        .left_0()
                        .w_full()
                        .h(line_height)
                        .bg(white().opacity(0.04)),
                )
            })
            .child({
                let content: AnyElement = self
                    .read_only
                    .then_some(self.language_extension)
                    .flatten()
                    .and_then(|extension| LANGUAGES.language_for_extension(extension))
                    .map(|language| {
                        let syntax = cx.theme().syntax();
                        let highlights: Vec<(Range<usize>, HighlightStyle)> =
                            language_core::highlighted_spans(language, &text)
                                .into_iter()
                                .filter_map(|(range, name)| {
                                    style_for_capture(syntax, &name).map(|style| (range, style))
                                })
                                .collect();
                        StyledText::new(text.clone())
                            .with_highlights(highlights)
                            .into_any_element()
                    })
                    .unwrap_or_else(|| self.input.clone().into_any_element());

                h_flex().w_full().items_start().child(gutter).child(
                    div()
                        .flex_1()
                        .min_w_0()
                        // Code text color (also the base color for
                        // unhighlighted spans/plain-text editing mode).
                        .text_color(rgb(0xB7BEC7))
                        .font_family(CODE_FONT_FAMILY)
                        .text_size(CODE_FONT_SIZE)
                        .line_height(relative(CODE_LINE_HEIGHT))
                        .child(content),
                )
            })
    }
}

/// Standalone gallery preview for `CodeEditor` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching
/// `SearchInput`'s existing convention in this crate). Shows both modes:
/// an editable plain-text buffer, and a read-only buffer with real
/// tree-sitter syntax highlighting (see [`CodeEditor::language`]).
pub fn code_editor_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_4()
        .child(cx.new(|cx| CodeEditor::new(cx)))
        .child(cx.new(|cx| {
            let mut editor = CodeEditor::new(cx).language("rs");
            editor.set_text(
                "fn main() {\n    let greeting = \"hello, world\";\n    println!(\"{greeting}\");\n}",
                cx,
            );
            editor.read_only(cx, true)
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use language_core::{DefaultLanguageRegistry, LanguageRegistry};
    use theme::default_themes;

    /// Audits that every capture name tree-sitter produces for each
    /// registered grammar resolves to *some* style via the real One Dark
    /// fallback theme, through the same [`super::style_for_capture`]
    /// dotted-prefix fallback `render` uses (e.g. `function.method` ->
    /// `function`). This is the "syntax theme mismatch" risk the Phase B
    /// plan flagged as needing verification before/while adding grammars —
    /// run for every grammar added here, not just Rust.
    ///
    /// Unmapped captures aren't a hard failure (`code_editor.rs` just skips
    /// coloring them — see `render`'s `filter_map`), so this asserts a
    /// coverage ratio rather than 100%: a sudden drop would mean a grammar's
    /// query uses a capture-naming convention the theme doesn't anticipate.
    fn assert_theme_covers_language(extension: &str, source: &str, min_coverage: f32) {
        let registry = DefaultLanguageRegistry::new();
        let language = registry
            .language_for_extension(extension)
            .unwrap_or_else(|| panic!("no grammar registered for .{extension}"));
        let theme = &default_themes().themes[0];
        let syntax = theme.syntax();

        let spans = language_core::highlighted_spans(language, source);
        assert!(
            !spans.is_empty(),
            "expected at least one highlight span for .{extension}"
        );

        let distinct_names: std::collections::BTreeSet<_> =
            spans.iter().map(|(_, name)| name.clone()).collect();
        let covered = distinct_names
            .iter()
            .filter(|name| super::style_for_capture(syntax, name).is_some())
            .count();
        let coverage = covered as f32 / distinct_names.len() as f32;

        assert!(
            coverage >= min_coverage,
            ".{extension}: only {covered}/{len} distinct capture names resolved via the \
             One Dark theme (coverage {coverage:.2} < {min_coverage:.2}). Uncovered: {uncovered:?}",
            len = distinct_names.len(),
            uncovered = distinct_names
                .iter()
                .filter(|name| super::style_for_capture(syntax, name).is_none())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn rust_highlights_covered_by_theme() {
        assert_theme_covers_language(
            "rs",
            "fn main() {\n    let s: &str = \"hi\";\n    println!(\"{s}\");\n}\n",
            0.7,
        );
    }

    #[test]
    fn javascript_highlights_covered_by_theme() {
        assert_theme_covers_language(
            "js",
            "function greet(name) {\n  return `hi ${name}`;\n}\nconst x = 1;\n",
            0.6,
        );
    }

    #[test]
    fn typescript_highlights_covered_by_theme() {
        assert_theme_covers_language(
            "ts",
            "interface Point { x: number; y: number }\nfunction f(p: Point): number { return p.x; }\n",
            0.6,
        );
    }

    #[test]
    fn json_highlights_covered_by_theme() {
        assert_theme_covers_language(
            "json",
            "{\"name\": \"base\", \"count\": 3, \"ok\": true}",
            0.6,
        );
    }

    #[test]
    fn markdown_highlights_covered_by_theme() {
        assert_theme_covers_language(
            "md",
            "# Title\n\nSome *text* with a [link](https://example.com).\n",
            0.4,
        );
    }
}
