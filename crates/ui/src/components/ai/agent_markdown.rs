use std::ops::Range;

use gpui::{AnyElement, FontWeight, HighlightStyle, SharedString, StyledText, UnderlineStyle};

use crate::CodeEditor;
use crate::prelude::*;

/// Renders a single agent chat message body as a minimal Markdown subset.
///
/// Supported blocks: headings (`#` .. `######`), unordered lists (`- ` / `* `),
/// ordered lists (`1. `), fenced code blocks (```` ```lang ```` .. ```` ``` ````)
/// and paragraphs. Supported inline spans (within paragraphs/list items/
/// headings): `**bold**`, `` `inline code` `` and `[text](url)`.
///
/// Non-goals (intentionally unsupported): tables, blockquotes, nested lists,
/// images, raw HTML passthrough. Links are display-only — the URL is
/// discarded after parsing and never auto-navigated or executed.
#[derive(IntoElement, RegisterComponent)]
pub struct AgentMarkdown {
    id: ElementId,
    text: SharedString,
}

impl AgentMarkdown {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }
}

impl RenderOnce for AgentMarkdown {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let blocks = parse_blocks(&self.text);
        v_flex()
            .id(self.id)
            .gap_2()
            .children(blocks.into_iter().map(|block| render_block(block, cx)))
    }
}

// ---------------------------------------------------------------------
// Block-level parsing
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Block {
    Heading { level: u8, text: String },
    UnorderedList(Vec<String>),
    OrderedList(Vec<String>),
    CodeFence { lang: Option<String>, code: String },
    Paragraph(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ListKind {
    Unordered,
    Ordered,
}

fn heading_level(line: &str) -> Option<u8> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if hashes >= 1 && hashes <= 6 && line.as_bytes().get(hashes) == Some(&b' ') {
        Some(hashes as u8)
    } else {
        None
    }
}

fn ordered_list_item(line: &str) -> Option<String> {
    let dot = line.find(". ")?;
    let (num, rest) = line.split_at(dot);
    if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
        Some(rest[2..].to_string())
    } else {
        None
    }
}

fn parse_blocks(input: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut paragraph_buf: Vec<&str> = Vec::new();
    let mut list_buf: Vec<String> = Vec::new();
    let mut list_kind: Option<ListKind> = None;

    let flush_paragraph = |blocks: &mut Vec<Block>, buf: &mut Vec<&str>| {
        if !buf.is_empty() {
            blocks.push(Block::Paragraph(buf.join(" ")));
            buf.clear();
        }
    };
    let flush_list =
        |blocks: &mut Vec<Block>, kind: &mut Option<ListKind>, buf: &mut Vec<String>| {
            if let Some(kind) = kind.take() {
                let items = std::mem::take(buf);
                blocks.push(match kind {
                    ListKind::Unordered => Block::UnorderedList(items),
                    ListKind::Ordered => Block::OrderedList(items),
                });
            }
        };

    let mut lines = input.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();

        if let Some(fence_lang) = trimmed.strip_prefix("```") {
            flush_paragraph(&mut blocks, &mut paragraph_buf);
            flush_list(&mut blocks, &mut list_kind, &mut list_buf);
            let lang = (!fence_lang.trim().is_empty()).then(|| fence_lang.trim().to_string());
            let mut code_lines = Vec::new();
            for code_line in lines.by_ref() {
                if code_line.trim_start().starts_with("```") {
                    break;
                }
                code_lines.push(code_line);
            }
            blocks.push(Block::CodeFence {
                lang,
                code: code_lines.join("\n"),
            });
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph_buf);
            flush_list(&mut blocks, &mut list_kind, &mut list_buf);
            continue;
        }

        if let Some(level) = heading_level(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_buf);
            flush_list(&mut blocks, &mut list_kind, &mut list_buf);
            let text = trimmed[level as usize + 1..].trim().to_string();
            blocks.push(Block::Heading { level, text });
            continue;
        }

        if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            flush_paragraph(&mut blocks, &mut paragraph_buf);
            if list_kind != Some(ListKind::Unordered) {
                flush_list(&mut blocks, &mut list_kind, &mut list_buf);
                list_kind = Some(ListKind::Unordered);
            }
            list_buf.push(item.to_string());
            continue;
        }

        if let Some(item) = ordered_list_item(trimmed) {
            flush_paragraph(&mut blocks, &mut paragraph_buf);
            if list_kind != Some(ListKind::Ordered) {
                flush_list(&mut blocks, &mut list_kind, &mut list_buf);
                list_kind = Some(ListKind::Ordered);
            }
            list_buf.push(item);
            continue;
        }

        flush_list(&mut blocks, &mut list_kind, &mut list_buf);
        paragraph_buf.push(trimmed);
    }

    flush_paragraph(&mut blocks, &mut paragraph_buf);
    flush_list(&mut blocks, &mut list_kind, &mut list_buf);
    blocks
}

// ---------------------------------------------------------------------
// Inline-level parsing
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum InlineStyle {
    Bold,
    Code,
    Link,
}

/// Strips `**bold**`, `` `code` `` and `[text](url)` markup out of `text`,
/// returning the display string plus the byte ranges (in that display
/// string) that need styling. The URL of a link is intentionally not
/// returned — links are rendered display-only, never navigated.
fn parse_inline(text: &str) -> (String, Vec<(Range<usize>, InlineStyle)>) {
    let mut output = String::with_capacity(text.len());
    let mut spans = Vec::new();
    let len = text.len();
    let mut i = 0;

    while i < len {
        if text[i..].starts_with("**") {
            if let Some(end) = text[i + 2..].find("**") {
                let inner = &text[i + 2..i + 2 + end];
                let start = output.len();
                output.push_str(inner);
                spans.push((start..output.len(), InlineStyle::Bold));
                i += 2 + end + 2;
                continue;
            }
        }

        if text.as_bytes()[i] == b'`' {
            if let Some(end) = text[i + 1..].find('`') {
                let inner = &text[i + 1..i + 1 + end];
                let start = output.len();
                output.push_str(inner);
                spans.push((start..output.len(), InlineStyle::Code));
                i += 1 + end + 1;
                continue;
            }
        }

        if text.as_bytes()[i] == b'[' {
            if let Some(bracket_rel) = text[i..].find(']') {
                let bracket_end = i + bracket_rel;
                if text.as_bytes().get(bracket_end + 1) == Some(&b'(') {
                    if let Some(paren_rel) = text[bracket_end + 2..].find(')') {
                        let paren_end = bracket_end + 2 + paren_rel;
                        let link_text = &text[i + 1..bracket_end];
                        let start = output.len();
                        output.push_str(link_text);
                        spans.push((start..output.len(), InlineStyle::Link));
                        i = paren_end + 1;
                        continue;
                    }
                }
            }
        }

        let ch = text[i..].chars().next().expect("i < len implies a char");
        output.push(ch);
        i += ch.len_utf8();
    }

    (output, spans)
}

/// Builds a single `StyledText` from a run of Markdown inline text — one
/// element per paragraph/list item/heading, never one element per character.
fn render_inline_text(text: &str, cx: &App) -> StyledText {
    let (stripped, spans) = parse_inline(text);
    if spans.is_empty() {
        return StyledText::new(stripped);
    }

    let buffer_font_family = theme::theme_settings(cx).buffer_font(cx).family.clone();
    let code_bg = cx.theme().colors().element_background;
    let link_color = Color::Accent.color(cx);

    let highlights: Vec<(Range<usize>, HighlightStyle)> = spans
        .iter()
        .map(|(range, style)| {
            let highlight = match style {
                InlineStyle::Bold => HighlightStyle {
                    font_weight: Some(FontWeight::BOLD),
                    ..Default::default()
                },
                InlineStyle::Code => HighlightStyle {
                    background_color: Some(code_bg),
                    ..Default::default()
                },
                InlineStyle::Link => HighlightStyle {
                    color: Some(link_color),
                    underline: Some(UnderlineStyle::default()),
                    ..Default::default()
                },
            };
            (range.clone(), highlight)
        })
        .collect();

    let font_overrides: Vec<(Range<usize>, SharedString)> = spans
        .iter()
        .filter(|(_, style)| *style == InlineStyle::Code)
        .map(|(range, _)| (range.clone(), buffer_font_family.clone()))
        .collect();

    StyledText::new(stripped)
        .with_highlights(highlights)
        .with_font_family_overrides(font_overrides)
}

// ---------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------

/// Maps a fenced code block's language tag to a `CodeEditor` extension.
/// Unrecognized/absent tags fall back to plain (unhighlighted) rendering.
fn extension_for_fence_lang(lang: &str) -> Option<&'static str> {
    match lang.to_ascii_lowercase().as_str() {
        "rust" | "rs" => Some("rs"),
        "javascript" | "js" | "jsx" => Some("js"),
        "typescript" | "ts" | "tsx" => Some("ts"),
        "json" => Some("json"),
        "markdown" | "md" => Some("md"),
        _ => None,
    }
}

fn render_block(block: Block, cx: &mut App) -> AnyElement {
    match block {
        Block::Heading { level, text } => {
            let heading = div()
                .font_weight(FontWeight::BOLD)
                .child(render_inline_text(&text, cx));
            match level {
                1 => heading.text_xl(),
                2 => heading.text_lg(),
                _ => heading,
            }
            .into_any_element()
        }
        Block::UnorderedList(items) => v_flex()
            .gap_0p5()
            .children(items.into_iter().map(|item| {
                h_flex()
                    .gap_1()
                    .items_start()
                    .child(Label::new("•").color(Color::Muted))
                    .child(div().child(render_inline_text(&item, cx)))
            }))
            .into_any_element(),
        Block::OrderedList(items) => v_flex()
            .gap_0p5()
            .children(items.into_iter().enumerate().map(|(index, item)| {
                h_flex()
                    .gap_1()
                    .items_start()
                    .child(Label::new(format!("{}.", index + 1)).color(Color::Muted))
                    .child(div().child(render_inline_text(&item, cx)))
            }))
            .into_any_element(),
        Block::CodeFence { lang, code } => {
            let extension = lang.as_deref().and_then(extension_for_fence_lang);
            cx.new(|cx| {
                let mut editor = CodeEditor::new(cx);
                if let Some(extension) = extension {
                    editor = editor.language(extension);
                }
                editor.set_text(code, cx);
                editor.read_only(cx, true)
            })
            .into_any_element()
        }
        Block::Paragraph(text) => div()
            .child(render_inline_text(&text, cx))
            .into_any_element(),
    }
}

impl Component for AgentMarkdown {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn description() -> Option<&'static str> {
        Some(
            "Renders a single agent chat message as a minimal Markdown subset: \
             headings, lists, fenced code blocks and paragraphs with bold/inline \
             code/link inline spans.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let container = || v_flex().w_96().gap_4();

        Some(
            container()
                .child(single_example(
                    "Full document",
                    AgentMarkdown::new(
                        "am-preview-1",
                        "# Heading One\n\n\
                         Some **bold** text with `inline code` and a [link](https://example.com).\n\n\
                         - First item\n\
                         - Second **bold** item\n\
                         - Third item with `code`\n\n\
                         1. Step one\n\
                         2. Step two\n\n\
                         ```rust\nfn main() {\n    println!(\"hi\");\n}\n```",
                    )
                    .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_levels_parsed() {
        let blocks = parse_blocks("# H1\n\n## H2\n\n###### H6");
        assert_eq!(
            blocks,
            vec![
                Block::Heading {
                    level: 1,
                    text: "H1".into()
                },
                Block::Heading {
                    level: 2,
                    text: "H2".into()
                },
                Block::Heading {
                    level: 6,
                    text: "H6".into()
                },
            ]
        );
    }

    #[test]
    fn hash_without_space_is_paragraph() {
        let blocks = parse_blocks("#nospace");
        assert_eq!(blocks, vec![Block::Paragraph("#nospace".into())]);
    }

    #[test]
    fn unordered_list_grouped() {
        let blocks = parse_blocks("- one\n- two\n* three");
        assert_eq!(
            blocks,
            vec![Block::UnorderedList(vec![
                "one".into(),
                "two".into(),
                "three".into(),
            ])]
        );
    }

    #[test]
    fn ordered_list_grouped() {
        let blocks = parse_blocks("1. one\n2. two\n10. ten");
        assert_eq!(
            blocks,
            vec![Block::OrderedList(vec![
                "one".into(),
                "two".into(),
                "ten".into(),
            ])]
        );
    }

    #[test]
    fn fenced_code_with_lang() {
        let blocks = parse_blocks("```rust\nfn main() {}\n```");
        assert_eq!(
            blocks,
            vec![Block::CodeFence {
                lang: Some("rust".into()),
                code: "fn main() {}".into(),
            }]
        );
    }

    #[test]
    fn fenced_code_without_lang() {
        let blocks = parse_blocks("```\nplain\ntext\n```");
        assert_eq!(
            blocks,
            vec![Block::CodeFence {
                lang: None,
                code: "plain\ntext".into(),
            }]
        );
    }

    #[test]
    fn paragraph_lines_joined() {
        let blocks = parse_blocks("line one\nline two\n\nnew paragraph");
        assert_eq!(
            blocks,
            vec![
                Block::Paragraph("line one line two".into()),
                Block::Paragraph("new paragraph".into()),
            ]
        );
    }

    #[test]
    fn mixed_blocks_in_order() {
        let blocks =
            parse_blocks("# Title\n\nIntro text.\n\n- item a\n- item b\n\n```js\nlet x = 1;\n```");
        assert_eq!(
            blocks,
            vec![
                Block::Heading {
                    level: 1,
                    text: "Title".into()
                },
                Block::Paragraph("Intro text.".into()),
                Block::UnorderedList(vec!["item a".into(), "item b".into()]),
                Block::CodeFence {
                    lang: Some("js".into()),
                    code: "let x = 1;".into(),
                },
            ]
        );
    }

    #[test]
    fn inline_bold() {
        let (text, spans) = parse_inline("say **hello** now");
        assert_eq!(text, "say hello now");
        assert_eq!(spans, vec![(4..9, InlineStyle::Bold)]);
    }

    #[test]
    fn inline_code() {
        let (text, spans) = parse_inline("run `cargo test` please");
        assert_eq!(text, "run cargo test please");
        assert_eq!(spans, vec![(4..14, InlineStyle::Code)]);
    }

    #[test]
    fn inline_link() {
        let (text, spans) = parse_inline("see [docs](https://example.com) now");
        assert_eq!(text, "see docs now");
        assert_eq!(spans, vec![(4..8, InlineStyle::Link)]);
    }

    #[test]
    fn inline_mixed() {
        let (text, spans) = parse_inline("**bold** and `code` and [link](url)");
        assert_eq!(text, "bold and code and link");
        assert_eq!(
            spans,
            vec![
                (0..4, InlineStyle::Bold),
                (9..13, InlineStyle::Code),
                (18..22, InlineStyle::Link),
            ]
        );
    }

    #[test]
    fn inline_plain_text_no_spans() {
        let (text, spans) = parse_inline("just plain text");
        assert_eq!(text, "just plain text");
        assert!(spans.is_empty());
    }

    #[test]
    fn extension_mapping() {
        assert_eq!(extension_for_fence_lang("rust"), Some("rs"));
        assert_eq!(extension_for_fence_lang("TypeScript"), Some("ts"));
        assert_eq!(extension_for_fence_lang("cobol"), None);
    }
}
