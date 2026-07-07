use std::ops::Range;
use std::sync::LazyLock;

use gpui::{AnyElement, HighlightStyle, StyledText};
use language::{DefaultLanguageRegistry, LanguageRegistry};

use crate::prelude::*;

/// Shares the process-wide grammar registry `CodeEditor` uses (see
/// `code_editor.rs`'s `LANGUAGES` module docs) — grammars/queries are
/// immutable after construction, so every `DiffBlock` reuses one registry
/// instead of re-parsing highlight queries per instance.
static LANGUAGES: LazyLock<DefaultLanguageRegistry> = LazyLock::new(DefaultLanguageRegistry::new);

/// Monospace font for diff text (mirrors `code_editor.rs`'s `CODE_FONT_FAMILY`
/// constant, duplicated locally since that constant is private to its
/// module).
const DIFF_FONT_FAMILY: &str = "IBM Plex Mono";
const DIFF_FONT_SIZE: Pixels = px(12.5);

/// Resolves a tree-sitter capture name to a style, falling back to its first
/// dotted segment if the exact name isn't in the theme. Mirrors
/// `code_editor.rs`'s `style_for_capture` helper — duplicated here rather
/// than shared because that helper is private to its module and
/// `boltz-markdown` (which has its own capture-to-style path) cannot depend
/// on `boltz-ui`.
fn style_for_capture(syntax: &syntax_theme::SyntaxTheme, name: &str) -> Option<HighlightStyle> {
    syntax.style_for_name(name).or_else(|| {
        let prefix = name.split('.').next()?;
        (prefix != name)
            .then(|| syntax.style_for_name(prefix))
            .flatten()
    })
}

fn highlighted_text(text: SharedString, extension: Option<&str>, cx: &App) -> AnyElement {
    let language = extension.and_then(|extension| LANGUAGES.language_for_extension(extension));
    match language {
        Some(language) => {
            let syntax = cx.theme().syntax();
            let highlights: Vec<(Range<usize>, HighlightStyle)> =
                language::highlighted_spans(language, &text)
                    .into_iter()
                    .filter_map(|(range, name)| {
                        style_for_capture(syntax, &name).map(|style| (range, style))
                    })
                    .collect();
            StyledText::new(text)
                .with_highlights(highlights)
                .into_any_element()
        }
        None => StyledText::new(text).into_any_element(),
    }
}

fn diff_section(
    label: &'static str,
    text: SharedString,
    extension: Option<&str>,
    bg: gpui::Hsla,
    label_color: Color,
    cx: &App,
) -> AnyElement {
    v_flex()
        .gap_1()
        .p_2()
        .rounded_md()
        .bg(bg)
        .child(Label::new(label).size(LabelSize::Small).color(label_color))
        .child(
            div()
                .font_family(DIFF_FONT_FAMILY)
                .text_size(DIFF_FONT_SIZE)
                .child(highlighted_text(text, extension, cx)),
        )
        .into_any_element()
}

/// Renders a tool-call diff as a plain unified old/new block (Decision #2a:
/// no `MultiBuffer`-based inline-editable diff view — see this component's
/// call sites for that trade-off's rationale). Old/new text is
/// syntax-highlighted via the same `language::highlighted_spans` tree-sitter
/// path `CodeEditor` uses.
#[derive(IntoElement, RegisterComponent)]
pub struct DiffBlock {
    id: ElementId,
    old_text: SharedString,
    new_text: SharedString,
    language: Option<SharedString>,
}

impl DiffBlock {
    pub fn new(
        id: impl Into<ElementId>,
        old_text: impl Into<SharedString>,
        new_text: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            old_text: old_text.into(),
            new_text: new_text.into(),
            language: None,
        }
    }

    /// File extension (no leading dot, e.g. `"rs"`) used to resolve syntax
    /// highlighting via `language::DefaultLanguageRegistry`. Unset or
    /// unrecognized extensions render as plain unhighlighted text.
    pub fn language(mut self, extension: impl Into<SharedString>) -> Self {
        self.language = Some(extension.into());
        self
    }
}

impl RenderOnce for DiffBlock {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let extension = self.language.as_deref();
        let status = cx.theme().status();

        v_flex()
            .id(self.id)
            .w_full()
            .gap_1()
            .child(diff_section(
                "− Old",
                self.old_text,
                extension,
                status.deleted_background,
                Color::Error,
                cx,
            ))
            .child(diff_section(
                "+ New",
                self.new_text,
                extension,
                status.created_background,
                Color::Success,
                cx,
            ))
    }
}

impl Component for DiffBlock {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn description() -> Option<&'static str> {
        Some(
            "Renders a tool-call diff as a plain unified old/new text block, \
             syntax-highlighted via the same tree-sitter path CodeEditor uses.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .w_96()
                .gap_4()
                .child(single_example(
                    "Rust diff",
                    DiffBlock::new(
                        "diff-preview-1",
                        "fn greet() {\n    println!(\"hi\");\n}",
                        "fn greet(name: &str) {\n    println!(\"hi, {name}\");\n}",
                    )
                    .language("rs")
                    .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
