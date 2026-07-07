//! `Markdown`, `MarkdownOptions`, `CodeBlockRenderer`, `EscapeAction`/
//! `MarkdownEscaper`, `impl Markdown`, `impl Focusable for Markdown`,
//! `SelectMode`, `Selection`, `ParsedMarkdown`, `AutoscrollBehavior`.
//!
//! Design notes (stem from this workspace's minimal `language` crate — see
//! `base/crates/language/src/language.rs`'s module docs — having no
//! `LanguageRegistry`/async loading/`LanguageName` types):
//! - Mermaid support is dropped entirely; fenced ` ```mermaid ` blocks render
//!   as plain code.
//! - `Markdown::new`/`new_with_options` take no `language_registry`/
//!   `fallback_code_block_language` parameters. Fenced code blocks resolve a
//!   language synchronously at *render* time (see
//!   [`resolve_code_block_language`]) against a process-wide
//!   `LazyLock<DefaultLanguageRegistry>` — the same pattern
//!   `base/crates/ui/src/components/code_editor.rs` uses — instead of an
//!   async per-document `LanguageRegistry` lookup populated during
//!   background parsing. `ParsedMarkdown` therefore has no
//!   `languages_by_name`/`languages_by_path` maps.
//! - `first_code_block_language` returns `Option<&'static language::Language>`
//!   (resolved via the same static registry) rather than `Option<Arc<Language>>`.
//! - `images_by_source_offset` is never populated from `data:` URL image
//!   sources (that would require decoding with the `base64` crate, which is
//!   not a workspace dependency here). Base64-embedded images fall through to
//!   `MarkdownElement::image_resolver` like any other image source, same as
//!   remote URLs.
//! - `copied_code_blocks` (a "just copied, show a flash" `HashSet` used so the
//!   copy button can briefly change icon) is dropped: this crate's
//!   `crate::controls::CopyButton` (see its module docs — no `boltz-ui`
//!   dependency allowed) has no `custom_on_click` hook to observe a
//!   post-copy state transition.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::{Arc, LazyLock};

use collections::{HashMap, HashSet};
use gpui::{
    App, AppContext as _, ClipboardItem, Context, Div, FocusHandle, Focusable, Image, Pixels,
    Point, ScrollHandle, SharedString, Task, Window, actions, point,
};
use language::{DefaultLanguageRegistry, Language, LanguageRegistry};

use crate::element::AnyDiv;
use crate::parser::{
    CodeBlockKind, CodeBlockMetadata, MarkdownEvent, MarkdownTag, ParsedMetadataBlock,
    parse_links_only, parse_markdown_with_options,
};
use crate::rendered::{RenderedFootnoteRef, RenderedLink, RenderedText};

actions!(
    markdown,
    [
        /// Copies the selected text to the clipboard.
        Copy,
        /// Copies the selected text as markdown to the clipboard.
        CopyAsMarkdown
    ]
);

/// Process-wide language registry used to resolve fenced code-block
/// languages at render time (see module docs).
static LANGUAGES: LazyLock<DefaultLanguageRegistry> = LazyLock::new(DefaultLanguageRegistry::new);

/// Maps a handful of common markdown fence info-string spellings (language
/// names, as opposed to file extensions) to the extension
/// [`DefaultLanguageRegistry`] indexes by. Intentionally small — only the
/// grammars `DefaultLanguageRegistry` ships are worth aliasing.
fn language_name_to_extension(name: &str) -> Option<&'static str> {
    const ALIASES: &[(&str, &str)] = &[
        ("rust", "rs"),
        ("rs", "rs"),
        ("javascript", "js"),
        ("js", "js"),
        ("jsx", "js"),
        ("mjs", "js"),
        ("cjs", "js"),
        ("typescript", "ts"),
        ("ts", "ts"),
        ("mts", "ts"),
        ("cts", "ts"),
        ("tsx", "tsx"),
        ("json", "json"),
        ("json5", "json"),
        ("jsonc", "json"),
        ("markdown", "md"),
        ("md", "md"),
    ];
    ALIASES
        .iter()
        .find(|(alias, _)| name.eq_ignore_ascii_case(alias))
        .map(|(_, extension)| *extension)
}

/// Resolves a fenced code block's language against the process-wide
/// [`LANGUAGES`] registry (see module docs for why this is a synchronous
/// lookup rather than an async per-document `LanguageRegistry` lookup).
pub(crate) fn resolve_code_block_language(kind: &CodeBlockKind) -> Option<&'static Language> {
    let extension = match kind {
        CodeBlockKind::FencedLang(name) => language_name_to_extension(name.as_ref())?,
        CodeBlockKind::FencedSrc(path_range) => std::path::Path::new(path_range.path.as_ref())
            .extension()
            .and_then(|extension| extension.to_str())?,
        CodeBlockKind::Fenced | CodeBlockKind::Indented => return None,
    };
    LANGUAGES.language_for_extension(extension)
}

pub struct Markdown {
    source: SharedString,
    pub(crate) selection: Selection,
    pub(crate) pressed_link: Option<RenderedLink>,
    pub(crate) pressed_footnote_ref: Option<RenderedFootnoteRef>,
    pub(crate) autoscroll_request: Option<usize>,
    active_root_block: Option<usize>,
    pub(crate) parsed_markdown: ParsedMarkdown,
    pub(crate) images_by_source_offset: HashMap<usize, Arc<Image>>,
    should_reparse: bool,
    pending_parse: Option<Task<()>>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) options: MarkdownOptions,
    wrapped_code_blocks: HashSet<usize>,
    code_block_scroll_handles: BTreeMap<usize, ScrollHandle>,
    context_menu_link: Option<SharedString>,
    context_menu_selected_text: Option<SharedString>,
    context_menu_selected_markdown: Option<SharedString>,
    pub(crate) search_highlights: Vec<Range<usize>>,
    pub(crate) active_search_highlight: Option<usize>,
}

#[derive(Clone, Copy, Default)]
pub struct MarkdownOptions {
    pub parse_links_only: bool,
    pub parse_html: bool,
    pub parse_heading_slugs: bool,
    pub render_metadata_blocks: bool,
}

pub enum CodeBlockRenderer {
    Default {
        copy_button_visibility: crate::style::CopyButtonVisibility,
        wrap_button_visibility: crate::style::WrapButtonVisibility,
        border: bool,
    },
    Custom {
        render: CodeBlockRenderFn,
        /// A function that can modify the parent container after the code block
        /// content has been appended as a child element.
        transform: Option<CodeBlockTransformFn>,
    },
}

pub type CodeBlockRenderFn = Arc<
    dyn Fn(
        &CodeBlockKind,
        &ParsedMarkdown,
        Range<usize>,
        CodeBlockMetadata,
        &mut Window,
        &App,
    ) -> Div,
>;

pub type CodeBlockTransformFn =
    Arc<dyn Fn(AnyDiv, Range<usize>, CodeBlockMetadata, &mut Window, &App) -> AnyDiv>;

enum EscapeAction {
    PassThrough,
    Nbsp(usize),
    DoubleNewline,
    PrefixBackslash,
}

impl EscapeAction {
    fn output_len(&self, c: char) -> usize {
        match self {
            Self::PassThrough => c.len_utf8(),
            Self::Nbsp(count) => count * '\u{00A0}'.len_utf8(),
            Self::DoubleNewline => 2,
            Self::PrefixBackslash => '\\'.len_utf8() + c.len_utf8(),
        }
    }

    fn write_to(&self, c: char, output: &mut String) {
        match self {
            Self::PassThrough => output.push(c),
            Self::Nbsp(count) => {
                for _ in 0..*count {
                    output.push('\u{00A0}');
                }
            }
            Self::DoubleNewline => {
                output.push('\n');
                output.push('\n');
            }
            Self::PrefixBackslash => {
                // '\\' is a single backslash in Rust, e.g. '|' -> '\|'
                output.push('\\');
                output.push(c);
            }
        }
    }
}

struct MarkdownEscaper {
    in_leading_whitespace: bool,
}

impl MarkdownEscaper {
    const TAB_SIZE: usize = 4;

    fn new() -> Self {
        Self {
            in_leading_whitespace: true,
        }
    }

    fn next(&mut self, c: char) -> EscapeAction {
        let action = if self.in_leading_whitespace && c == '\t' {
            EscapeAction::Nbsp(Self::TAB_SIZE)
        } else if self.in_leading_whitespace && c == ' ' {
            EscapeAction::Nbsp(1)
        } else if c == '\n' {
            EscapeAction::DoubleNewline
        } else if c.is_ascii_punctuation() {
            EscapeAction::PrefixBackslash
        } else {
            EscapeAction::PassThrough
        };

        self.in_leading_whitespace =
            c == '\n' || (self.in_leading_whitespace && (c == ' ' || c == '\t'));
        action
    }
}

impl Markdown {
    pub fn new(source: SharedString, cx: &mut Context<Self>) -> Self {
        Self::new_with_options(source, MarkdownOptions::default(), cx)
    }

    pub fn new_with_options(
        source: SharedString,
        options: MarkdownOptions,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();

        let mut this = Self {
            source,
            selection: Selection::default(),
            pressed_link: None,
            pressed_footnote_ref: None,
            autoscroll_request: None,
            active_root_block: None,
            should_reparse: false,
            images_by_source_offset: Default::default(),
            parsed_markdown: ParsedMarkdown::default(),
            pending_parse: None,
            focus_handle,
            options,
            wrapped_code_blocks: HashSet::default(),
            code_block_scroll_handles: BTreeMap::default(),
            context_menu_link: None,
            context_menu_selected_text: None,
            context_menu_selected_markdown: None,
            search_highlights: Vec::new(),
            active_search_highlight: None,
        };
        this.parse(cx);
        this
    }

    pub fn new_text(source: SharedString, cx: &mut Context<Self>) -> Self {
        Self::new_with_options(
            source,
            MarkdownOptions {
                parse_links_only: true,
                ..Default::default()
            },
            cx,
        )
    }

    pub(crate) fn is_code_block_wrapped(&self, id: usize) -> bool {
        self.wrapped_code_blocks.contains(&id)
    }

    pub(crate) fn toggle_code_block_wrap(&mut self, id: usize) {
        if !self.wrapped_code_blocks.remove(&id) {
            self.wrapped_code_blocks.insert(id);
        }
    }

    pub(crate) fn code_block_scroll_handle(&mut self, id: usize) -> Option<ScrollHandle> {
        (!self.is_code_block_wrapped(id)).then(|| {
            self.code_block_scroll_handles
                .entry(id)
                .or_insert_with(ScrollHandle::new)
                .clone()
        })
    }

    pub(crate) fn retain_code_block_scroll_handles(&mut self, ids: &HashSet<usize>) {
        self.code_block_scroll_handles
            .retain(|id, _| ids.contains(id));
    }

    pub(crate) fn clear_code_block_scroll_handles(&mut self) {
        self.code_block_scroll_handles.clear();
    }

    pub(crate) fn autoscroll_code_block(
        &self,
        source_index: usize,
        cursor_position: Point<Pixels>,
    ) {
        let Some((_, scroll_handle)) = self
            .code_block_scroll_handles
            .range(..=source_index)
            .next_back()
        else {
            return;
        };

        let bounds = scroll_handle.bounds();
        if cursor_position.y < bounds.top() || cursor_position.y > bounds.bottom() {
            return;
        }

        let horizontal_delta = if cursor_position.x < bounds.left() {
            bounds.left() - cursor_position.x
        } else if cursor_position.x > bounds.right() {
            bounds.right() - cursor_position.x
        } else {
            return;
        };

        let offset = scroll_handle.offset();
        scroll_handle.set_offset(point(offset.x + horizontal_delta, offset.y));
    }

    pub fn is_parsing(&self) -> bool {
        self.pending_parse.is_some()
    }

    pub fn scroll_to_heading(&mut self, slug: &str, cx: &mut Context<Self>) -> Option<usize> {
        if let Some(source_index) = self.parsed_markdown.heading_slugs.get(slug).copied() {
            self.autoscroll_request = Some(source_index);
            cx.notify();
            Some(source_index)
        } else {
            None
        }
    }

    pub fn source(&self) -> &SharedString {
        &self.source
    }

    /// Resolves the first fenced code block's language, if any (see module
    /// docs for the static-registry resolution this uses instead of an
    /// `Arc<Language>`-based per-document resolution).
    pub fn first_code_block_language(&self) -> Option<&'static Language> {
        self.parsed_markdown.events.iter().find_map(|(_, event)| {
            let MarkdownEvent::Start(MarkdownTag::CodeBlock { kind, .. }) = event else {
                return None;
            };
            resolve_code_block_language(kind)
        })
    }

    pub fn append(&mut self, text: &str, cx: &mut Context<Self>) {
        self.source = SharedString::new(self.source.to_string() + text);
        self.parse(cx);
    }

    pub fn replace(&mut self, source: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.source = source.into();
        self.parse(cx);
    }

    pub fn request_autoscroll_to_source_index(
        &mut self,
        source_index: usize,
        cx: &mut Context<Self>,
    ) {
        self.autoscroll_request = Some(source_index);
        cx.refresh_windows();
    }

    pub(crate) fn footnote_definition_content_start(&self, label: &SharedString) -> Option<usize> {
        self.parsed_markdown
            .footnote_definitions
            .get(label)
            .copied()
    }

    pub fn set_active_root_for_source_index(
        &mut self,
        source_index: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        let active_root_block =
            source_index.and_then(|index| self.parsed_markdown.root_block_for_source_index(index));
        if self.active_root_block == active_root_block {
            return;
        }

        self.active_root_block = active_root_block;
        cx.notify();
    }

    pub(crate) fn active_root_block(&self) -> Option<usize> {
        self.active_root_block
    }

    pub fn reset(&mut self, source: SharedString, cx: &mut Context<Self>) {
        if &source == self.source() {
            return;
        }
        self.source = source;
        self.selection = Selection::default();
        self.autoscroll_request = None;
        self.pending_parse = None;
        self.should_reparse = false;
        self.search_highlights.clear();
        self.active_search_highlight = None;
        // Don't clear parsed_markdown here - keep existing content visible until new parse completes
        self.parse(cx);
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn parsed_markdown(&self) -> &ParsedMarkdown {
        &self.parsed_markdown
    }

    pub fn escape(s: &str) -> Cow<'_, str> {
        let output_len: usize = {
            let mut escaper = MarkdownEscaper::new();
            s.chars().map(|c| escaper.next(c).output_len(c)).sum()
        };

        if output_len == s.len() {
            return s.into();
        }

        let mut escaper = MarkdownEscaper::new();
        let mut output = String::with_capacity(output_len);
        for c in s.chars() {
            escaper.next(c).write_to(c, &mut output);
        }
        output.into()
    }

    pub fn selected_text(&self) -> Option<String> {
        if self.selection.end <= self.selection.start {
            None
        } else {
            Some(self.source[self.selection.start..self.selection.end].to_string())
        }
    }

    pub fn set_search_highlights(
        &mut self,
        highlights: Vec<Range<usize>>,
        active: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        debug_assert!(
            highlights
                .windows(2)
                .all(|ranges| (ranges[0].start, ranges[0].end) <= (ranges[1].start, ranges[1].end))
        );
        self.search_highlights = highlights;
        self.active_search_highlight =
            active.filter(|active| *active < self.search_highlights.len());
        cx.notify();
    }

    pub fn clear_search_highlights(&mut self, cx: &mut Context<Self>) {
        if !self.search_highlights.is_empty() || self.active_search_highlight.is_some() {
            self.search_highlights.clear();
            self.active_search_highlight = None;
            cx.notify();
        }
    }

    pub fn set_active_search_highlight(&mut self, active: Option<usize>, cx: &mut Context<Self>) {
        let active = active.filter(|active| *active < self.search_highlights.len());
        if self.active_search_highlight != active {
            self.active_search_highlight = active;
            cx.notify();
        }
    }

    pub fn search_highlights(&self) -> &[Range<usize>] {
        &self.search_highlights
    }

    pub fn active_search_highlight(&self) -> Option<usize> {
        self.active_search_highlight
    }

    pub(crate) fn copy(&self, text: &RenderedText, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection.end <= self.selection.start {
            return;
        }
        let text = text.text_for_range(self.selection.start..self.selection.end);
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    pub(crate) fn copy_as_markdown(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = self.context_menu_selected_markdown.take() {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            return;
        }
        if self.selection.end <= self.selection.start {
            return;
        }
        let text = self.source[self.selection.start..self.selection.end].to_string();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    pub(crate) fn capture_for_context_menu(
        &mut self,
        link: Option<SharedString>,
        rendered_text: Option<&RenderedText>,
    ) {
        let range = self.selection.start..self.selection.end;
        if range.end > range.start {
            self.context_menu_selected_markdown =
                Some(SharedString::new(&self.source[range.clone()]));
            self.context_menu_selected_text = rendered_text
                .map(|text| text.text_for_range(range))
                .map(SharedString::new)
                .or_else(|| self.context_menu_selected_markdown.clone());
        } else {
            self.context_menu_selected_markdown = None;
            self.context_menu_selected_text = None;
        }
        self.context_menu_link = link;
    }

    /// Returns the URL of the link that was most recently right-clicked, if any.
    /// This is set during a right-click mouse-down event and can be read by parent
    /// views to include a "Copy Link" item in their context menus.
    pub fn context_menu_link(&self) -> Option<&SharedString> {
        self.context_menu_link.as_ref()
    }

    /// Returns the rendered (plain) text that was selected when the most recent
    /// context menu invocation happened.
    pub fn context_menu_selected_text(&self) -> Option<&SharedString> {
        self.context_menu_selected_text.as_ref()
    }

    /// Returns the raw markdown source that was selected when the most recent
    /// context menu invocation happened.
    pub fn context_menu_selected_markdown(&self) -> Option<&SharedString> {
        self.context_menu_selected_markdown.as_ref()
    }

    fn parse(&mut self, cx: &mut Context<Self>) {
        if self.source.is_empty() {
            self.should_reparse = false;
            self.pending_parse.take();
            self.parsed_markdown = ParsedMarkdown {
                source: self.source.clone(),
                ..Default::default()
            };
            self.active_root_block = None;
            self.images_by_source_offset.clear();
            cx.notify();
            cx.refresh_windows();
            return;
        }

        if self.pending_parse.is_some() {
            self.should_reparse = true;
            return;
        }
        self.should_reparse = false;
        self.pending_parse = Some(self.start_background_parse(cx));
    }

    fn start_background_parse(&self, cx: &Context<Self>) -> Task<()> {
        let source = self.source.clone();
        let should_parse_links_only = self.options.parse_links_only;
        let should_parse_html = self.options.parse_html;
        let should_parse_heading_slugs = self.options.parse_heading_slugs;
        let should_parse_metadata_blocks = self.options.render_metadata_blocks;

        let parsed = cx.background_spawn(async move {
            if should_parse_links_only {
                return ParsedMarkdown {
                    events: Arc::from(parse_links_only(source.as_ref())),
                    source,
                    root_block_starts: Arc::default(),
                    html_blocks: BTreeMap::default(),
                    metadata_blocks: BTreeMap::default(),
                    heading_slugs: HashMap::default(),
                    footnote_definitions: HashMap::default(),
                };
            }

            let parsed = parse_markdown_with_options(
                &source,
                should_parse_html,
                should_parse_heading_slugs,
                should_parse_metadata_blocks,
            );

            ParsedMarkdown {
                source,
                events: Arc::from(parsed.events),
                root_block_starts: Arc::from(parsed.root_block_starts),
                html_blocks: parsed.html_blocks,
                metadata_blocks: parsed.metadata_blocks,
                heading_slugs: parsed.heading_slugs,
                footnote_definitions: parsed.footnote_definitions,
            }
        });

        cx.spawn(async move |this, cx| {
            let parsed = parsed.await;

            this.update(cx, |this, cx| {
                this.parsed_markdown = parsed;
                if this.active_root_block.is_some_and(|block_index| {
                    block_index >= this.parsed_markdown.root_block_starts.len()
                }) {
                    this.active_root_block = None;
                }
                this.pending_parse.take();
                if this.should_reparse {
                    this.parse(cx);
                }
                cx.notify();
                cx.refresh_windows();
            })
            .ok();
        })
    }
}

impl Focusable for Markdown {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) enum SelectMode {
    #[default]
    Character,
    Word(Range<usize>),
    Line(Range<usize>),
    All,
}

#[derive(Clone, Default)]
pub(crate) struct Selection {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) reversed: bool,
    pub(crate) pending: bool,
    pub(crate) mode: SelectMode,
}

impl Selection {
    pub(crate) fn set_head(&mut self, head: usize, rendered_text: &RenderedText) {
        match &self.mode {
            SelectMode::Character => {
                if head < self.tail() {
                    if !self.reversed {
                        self.end = self.start;
                        self.reversed = true;
                    }
                    self.start = head;
                } else {
                    if self.reversed {
                        self.start = self.end;
                        self.reversed = false;
                    }
                    self.end = head;
                }
            }
            SelectMode::Word(original_range) | SelectMode::Line(original_range) => {
                let head_range = if matches!(self.mode, SelectMode::Word(_)) {
                    rendered_text.surrounding_word_range(head)
                } else {
                    rendered_text.surrounding_line_range(head)
                };

                if head < original_range.start {
                    self.start = head_range.start;
                    self.end = original_range.end;
                    self.reversed = true;
                } else if head >= original_range.end {
                    self.start = original_range.start;
                    self.end = head_range.end;
                    self.reversed = false;
                } else {
                    self.start = original_range.start;
                    self.end = original_range.end;
                    self.reversed = false;
                }
            }
            SelectMode::All => {
                self.start = 0;
                self.end = rendered_text
                    .lines
                    .last()
                    .map(|line| line.source_end)
                    .unwrap_or(0);
                self.reversed = false;
            }
        }
    }

    pub(crate) fn tail(&self) -> usize {
        if self.reversed { self.end } else { self.start }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParsedMarkdown {
    pub source: SharedString,
    pub events: Arc<[(Range<usize>, MarkdownEvent)]>,
    pub root_block_starts: Arc<[usize]>,
    pub(crate) html_blocks: BTreeMap<usize, crate::parser::html::html_parser::ParsedHtmlBlock>,
    pub(crate) metadata_blocks: BTreeMap<usize, ParsedMetadataBlock>,
    pub heading_slugs: HashMap<SharedString, usize>,
    pub footnote_definitions: HashMap<SharedString, usize>,
}

impl ParsedMarkdown {
    pub fn source(&self) -> &SharedString {
        &self.source
    }

    pub fn events(&self) -> &Arc<[(Range<usize>, MarkdownEvent)]> {
        &self.events
    }

    pub fn root_block_starts(&self) -> &Arc<[usize]> {
        &self.root_block_starts
    }

    pub fn root_block_for_source_index(&self, source_index: usize) -> Option<usize> {
        if self.root_block_starts.is_empty() {
            return None;
        }

        let partition = self
            .root_block_starts
            .partition_point(|block_start| *block_start <= source_index);

        Some(partition.saturating_sub(1))
    }
}

pub enum AutoscrollBehavior {
    /// Propagate the request up the element tree for the nearest
    /// scrollable ancestor (e.g. `List`) to handle.
    Propagate,
    /// Directly control a specific scroll handle.
    Controlled(ScrollHandle),
}
