use gpui::Entity;
use markdown::{Markdown, MarkdownElement, MarkdownFont, MarkdownFontConfig, MarkdownStyle};

use crate::prelude::*;

/// Builds the font configuration `boltz-markdown` needs to size/family its
/// text, sourced from this workspace's `theme::theme_settings` global (no
/// per-crate settings global exists here — see `code_editor.rs`'s
/// `CODE_FONT_FAMILY` module docs for the established stateless-config
/// alternative). Agent/preview font sizes reuse the same UI/buffer sizes,
/// since this workspace has no distinct agent-panel-specific size setting.
pub fn agent_markdown_font_config(cx: &App) -> MarkdownFontConfig {
    let settings = theme::theme_settings(cx);
    let ui_font = settings.ui_font(cx).clone();
    let buffer_font = settings.buffer_font(cx).clone();
    let ui_font_size = settings.ui_font_size(cx);
    let buffer_font_size = settings.buffer_font_size(cx);

    MarkdownFontConfig {
        ui_font_family: ui_font.family.clone(),
        ui_font_fallbacks: ui_font.fallbacks.clone(),
        ui_font_features: ui_font.features.clone(),
        ui_font_size,
        buffer_font_family: buffer_font.family.clone(),
        buffer_font_fallbacks: buffer_font.fallbacks.clone(),
        buffer_font_features: buffer_font.features.clone(),
        buffer_font_weight: buffer_font.weight,
        buffer_font_size,
        agent_buffer_font_size: buffer_font_size,
        agent_ui_font_size: ui_font_size,
        markdown_preview_font_size: ui_font_size,
        markdown_preview_font_family: ui_font.family,
        markdown_preview_code_font_family: buffer_font.family,
    }
}

/// The [`MarkdownStyle`] used to render agent chat markdown bodies
/// (messages, thinking blocks, tool-call text content).
pub fn agent_markdown_style(window: &Window, cx: &App) -> MarkdownStyle {
    let fonts = agent_markdown_font_config(cx);
    MarkdownStyle::themed(MarkdownFont::Agent, &fonts, window, cx)
}

/// Creates a freshly parsed [`Markdown`] entity from `text`.
///
/// `Markdown` parses in a background task and only shows content once that
/// task completes and updates the entity — an entity with no remaining
/// strong reference is dropped before that update can land, so its content
/// never reaches the screen. Callers that render the same logical message
/// across multiple frames (e.g. a streaming assistant turn) must create this
/// once and keep the returned `Entity` alive for as long as it's displayed,
/// updating its text via `Markdown::reset`/`Markdown::append` as new content
/// arrives, rather than calling this again on every render.
pub fn agent_markdown_entity(text: impl Into<SharedString>, cx: &mut App) -> Entity<Markdown> {
    cx.new(|cx| Markdown::new(text.into(), cx))
}

/// Renders a single agent chat message body as full CommonMark/GFM markdown
/// (tables, footnotes, task lists, syntax-highlighted fenced code, etc.) via
/// `boltz-markdown`.
///
/// Wraps a caller-owned [`Entity<Markdown>`] rather than raw text, since
/// `Markdown` parses asynchronously (see [`agent_markdown_entity`]'s docs for
/// why a persistent entity is required for content to ever render).
#[derive(IntoElement, RegisterComponent)]
pub struct AgentMarkdown {
    id: ElementId,
    markdown: Entity<Markdown>,
}

impl AgentMarkdown {
    pub fn new(id: impl Into<ElementId>, markdown: Entity<Markdown>) -> Self {
        Self {
            id: id.into(),
            markdown,
        }
    }
}

impl RenderOnce for AgentMarkdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = agent_markdown_style(window, cx);
        div()
            .id(self.id)
            .w_full()
            .child(MarkdownElement::new(self.markdown, style))
    }
}

impl Component for AgentMarkdown {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn description() -> Option<&'static str> {
        Some(
            "Renders a single agent chat message body as full CommonMark/GFM \
             markdown (tables, footnotes, task lists, syntax-highlighted \
             fenced code) via boltz-markdown, wrapping a caller-owned \
             Entity<Markdown>.",
        )
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        let source = "# Heading One\n\n\
             Some **bold** text with `inline code` and a [link](https://example.com).\n\n\
             | Left | Right |\n|---|---|\n| a | b |\n\n\
             - [x] Done item\n\
             - [ ] Pending item\n\n\
             1. Step one\n\
             2. Step two\n\n\
             ```rust\nfn main() {\n    println!(\"hi\");\n}\n```\n\n\
             A footnote reference[^1].\n\n[^1]: The footnote body.";
        let entity = agent_markdown_entity(source, cx);

        Some(
            v_flex()
                .w_96()
                .gap_4()
                .child(single_example(
                    "Full document",
                    AgentMarkdown::new("am-preview-1", entity).into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
