//! No `theme_settings` crate exists in this workspace, so font
//! family/size/weight are not read from a settings global; see
//! `code_editor.rs`'s `CODE_FONT_FAMILY`/`CODE_FONT_SIZE` consts for the
//! established alternative pattern here: stateless, explicit config instead
//! of a settings global. [`MarkdownFontConfig`] carries that config
//! explicitly at construction time; theme *colors*/*syntax* still come from
//! `cx.theme()` (`boltz-theme`).
//!
//! `CodeBlockRenderer` is intentionally not defined here even though it's
//! visually a "style" concern: its `Custom` variant's
//! `CodeBlockRenderFn`/`CodeBlockTransformFn` types reference
//! `ParsedMarkdown`/`AnyDiv`/`Div`, which live in `entity.rs`/`element.rs`.

use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AbsoluteLength, App, BorderStyle, DefiniteLength, EdgesRefinement, FontFallbacks, FontFeatures,
    FontWeight, Hsla, Length, Pixels, Refineable as _, SharedString, StyleRefinement, TextStyle,
    TextStyleRefinement, UnderlineStyle, Window, px, rems,
};
use pulldown_cmark::BlockQuoteKind;
use syntax_theme::SyntaxTheme;
use theme::{ActiveTheme, ThemeColors};

/// A callback function that can be used to customize the style of links
/// based on the destination URL. If the callback returns `None`, the
/// default link style will be used.
type LinkStyleCallback = Rc<dyn Fn(&str, &App) -> Option<TextStyleRefinement>>;

#[derive(Clone, Copy, Default)]
pub struct BlockQuoteKindColors {
    pub note: Hsla,
    pub tip: Hsla,
    pub important: Hsla,
    pub warning: Hsla,
    pub caution: Hsla,
}

impl BlockQuoteKindColors {
    pub(crate) fn for_kind(&self, kind: Option<BlockQuoteKind>, default: Hsla) -> Hsla {
        match kind {
            Some(BlockQuoteKind::Note) => self.note,
            Some(BlockQuoteKind::Tip) => self.tip,
            Some(BlockQuoteKind::Important) => self.important,
            Some(BlockQuoteKind::Warning) => self.warning,
            Some(BlockQuoteKind::Caution) => self.caution,
            None => default,
        }
    }
}

#[derive(Clone, Default)]
pub struct HeadingLevelStyles {
    pub h1: Option<TextStyleRefinement>,
    pub h2: Option<TextStyleRefinement>,
    pub h3: Option<TextStyleRefinement>,
    pub h4: Option<TextStyleRefinement>,
    pub h5: Option<TextStyleRefinement>,
    pub h6: Option<TextStyleRefinement>,
}

#[derive(Clone)]
pub struct MarkdownStyle {
    pub base_text_style: TextStyle,
    pub container_style: StyleRefinement,
    pub code_block: StyleRefinement,
    pub code_block_overflow_x_scroll: bool,
    pub inline_code: TextStyleRefinement,
    pub block_quote: TextStyleRefinement,
    pub link: TextStyleRefinement,
    pub link_callback: Option<LinkStyleCallback>,
    pub rule_color: Hsla,
    pub block_quote_border_color: Hsla,
    pub block_quote_kind_colors: BlockQuoteKindColors,
    pub syntax: Arc<SyntaxTheme>,
    pub selection_background_color: Hsla,
    pub heading: StyleRefinement,
    pub heading_level_styles: Option<HeadingLevelStyles>,
    pub heading_border_color: Option<Hsla>,
    pub height_is_multiple_of_line_height: bool,
    pub prevent_mouse_interaction: bool,
    pub table_columns_min_size: bool,
    pub soft_break_as_hard_break: bool,
}

impl Default for MarkdownStyle {
    fn default() -> Self {
        Self {
            base_text_style: Default::default(),
            container_style: Default::default(),
            code_block: Default::default(),
            code_block_overflow_x_scroll: false,
            inline_code: Default::default(),
            block_quote: Default::default(),
            link: Default::default(),
            link_callback: None,
            rule_color: Default::default(),
            block_quote_border_color: Default::default(),
            block_quote_kind_colors: Default::default(),
            syntax: Arc::new(SyntaxTheme::default()),
            selection_background_color: Default::default(),
            heading: Default::default(),
            heading_level_styles: None,
            heading_border_color: None,
            height_is_multiple_of_line_height: false,
            prevent_mouse_interaction: false,
            table_columns_min_size: false,
            soft_break_as_hard_break: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum MarkdownFont {
    Agent,
    Editor,
    Preview,
}

/// Explicit font configuration, passed in by the caller instead of read from
/// a global `ThemeSettings` (no `theme_settings` crate exists in this
/// workspace — see module docs above).
#[derive(Clone)]
pub struct MarkdownFontConfig {
    pub ui_font_family: SharedString,
    pub ui_font_fallbacks: Option<FontFallbacks>,
    pub ui_font_features: FontFeatures,
    pub ui_font_size: Pixels,
    pub buffer_font_family: SharedString,
    pub buffer_font_fallbacks: Option<FontFallbacks>,
    pub buffer_font_features: FontFeatures,
    pub buffer_font_weight: FontWeight,
    pub buffer_font_size: Pixels,
    pub agent_buffer_font_size: Pixels,
    pub agent_ui_font_size: Pixels,
    pub markdown_preview_font_size: Pixels,
    pub markdown_preview_font_family: SharedString,
    pub markdown_preview_code_font_family: SharedString,
}

impl MarkdownStyle {
    pub fn themed(
        font: MarkdownFont,
        fonts: &MarkdownFontConfig,
        window: &Window,
        cx: &App,
    ) -> Self {
        let colors = cx.theme().colors();
        let syntax = cx.theme().syntax().clone();
        Self::themed_with_overrides(font, colors, &syntax, fonts, window, cx)
    }

    /// Like [`Self::themed`], but takes explicit [`ThemeColors`] and
    /// [`SyntaxTheme`] so callers (e.g. the markdown preview) can render the
    /// markdown using a theme other than the active editor theme.
    pub fn themed_with_overrides(
        font: MarkdownFont,
        colors: &ThemeColors,
        syntax: &Arc<SyntaxTheme>,
        fonts: &MarkdownFontConfig,
        window: &Window,
        cx: &App,
    ) -> Self {
        let is_preview = matches!(font, MarkdownFont::Preview);

        let buffer_font_weight = fonts.buffer_font_weight;
        let (buffer_font_size, ui_font_size) = match font {
            MarkdownFont::Agent => (fonts.agent_buffer_font_size, fonts.agent_ui_font_size),
            MarkdownFont::Editor => (fonts.buffer_font_size, fonts.ui_font_size),
            MarkdownFont::Preview => (fonts.markdown_preview_font_size, fonts.ui_font_size),
        };

        let body_font_family = if is_preview {
            fonts.markdown_preview_font_family.clone()
        } else {
            fonts.ui_font_family.clone()
        };
        let code_font_family = if is_preview {
            fonts.markdown_preview_code_font_family.clone()
        } else {
            fonts.buffer_font_family.clone()
        };

        let mut text_style = window.text_style();
        let line_height = buffer_font_size * 1.75;

        text_style.refine(&TextStyleRefinement {
            font_family: Some(body_font_family),
            font_fallbacks: fonts.ui_font_fallbacks.clone(),
            font_features: Some(fonts.ui_font_features.clone()),
            font_size: Some(if is_preview {
                rems(1.0).into()
            } else {
                ui_font_size.into()
            }),
            line_height: Some(line_height.into()),
            color: Some(colors.text),
            ..Default::default()
        });

        let style = MarkdownStyle {
            base_text_style: text_style.clone(),
            syntax: syntax.clone(),
            selection_background_color: colors.element_selection_background,
            rule_color: colors.border,
            block_quote_border_color: colors.border,
            block_quote_kind_colors: {
                let status = cx.theme().status();
                BlockQuoteKindColors {
                    note: status.info,
                    tip: status.success,
                    important: status.info,
                    warning: status.warning,
                    caution: status.error,
                }
            },
            code_block_overflow_x_scroll: true,
            code_block: StyleRefinement {
                padding: EdgesRefinement {
                    top: Some(DefiniteLength::Absolute(AbsoluteLength::Pixels(px(8.)))),
                    left: Some(DefiniteLength::Absolute(AbsoluteLength::Pixels(px(8.)))),
                    right: Some(DefiniteLength::Absolute(AbsoluteLength::Pixels(px(8.)))),
                    bottom: Some(DefiniteLength::Absolute(AbsoluteLength::Pixels(px(8.)))),
                },
                margin: EdgesRefinement {
                    top: Some(Length::Definite(px(8.).into())),
                    left: Some(Length::Definite(px(0.).into())),
                    right: Some(Length::Definite(px(0.).into())),
                    bottom: Some(Length::Definite(px(12.).into())),
                },
                border_style: Some(BorderStyle::Solid),
                border_widths: EdgesRefinement {
                    top: Some(AbsoluteLength::Pixels(px(1.))),
                    left: Some(AbsoluteLength::Pixels(px(1.))),
                    right: Some(AbsoluteLength::Pixels(px(1.))),
                    bottom: Some(AbsoluteLength::Pixels(px(1.))),
                },
                border_color: Some(colors.border_variant),
                background: Some(colors.editor_background.into()),
                text: TextStyleRefinement {
                    font_family: Some(code_font_family.clone()),
                    font_fallbacks: fonts.buffer_font_fallbacks.clone(),
                    font_features: Some(fonts.buffer_font_features.clone()),
                    font_size: Some(buffer_font_size.into()),
                    font_weight: Some(buffer_font_weight),
                    ..Default::default()
                },
                ..Default::default()
            },
            inline_code: TextStyleRefinement {
                font_family: Some(code_font_family),
                font_fallbacks: fonts.buffer_font_fallbacks.clone(),
                font_features: Some(fonts.buffer_font_features.clone()),
                font_size: Some(buffer_font_size.into()),
                font_weight: Some(buffer_font_weight),
                background_color: Some(colors.editor_foreground.opacity(0.08)),
                ..Default::default()
            },
            link: TextStyleRefinement {
                background_color: Some(colors.editor_foreground.opacity(0.025)),
                color: Some(colors.text_accent),
                underline: Some(UnderlineStyle {
                    color: Some(colors.text_accent.opacity(0.5)),
                    thickness: px(1.),
                    ..Default::default()
                }),
                ..Default::default()
            },
            soft_break_as_hard_break: matches!(font, MarkdownFont::Agent),
            heading_level_styles: matches!(font, MarkdownFont::Agent).then_some(
                HeadingLevelStyles {
                    h1: Some(TextStyleRefinement {
                        font_size: Some(rems(1.15).into()),
                        ..Default::default()
                    }),
                    h2: Some(TextStyleRefinement {
                        font_size: Some(rems(1.1).into()),
                        ..Default::default()
                    }),
                    h3: Some(TextStyleRefinement {
                        font_size: Some(rems(1.05).into()),
                        ..Default::default()
                    }),
                    h4: Some(TextStyleRefinement {
                        font_size: Some(rems(1.).into()),
                        ..Default::default()
                    }),
                    h5: Some(TextStyleRefinement {
                        font_size: Some(rems(0.95).into()),
                        ..Default::default()
                    }),
                    h6: Some(TextStyleRefinement {
                        font_size: Some(rems(0.875).into()),
                        ..Default::default()
                    }),
                },
            ),
            ..Default::default()
        };

        if is_preview {
            style.with_preview_overrides(colors)
        } else {
            style
        }
    }

    fn with_preview_overrides(mut self, colors: &ThemeColors) -> Self {
        let body_font_size = rems(0.92);
        self.base_text_style.font_size = body_font_size.into();
        self.container_style.text.font_size = Some(body_font_size.into());

        self.base_text_style.color = colors.text_muted.blend(colors.text.opacity(0.25));
        self.inline_code.color = Some(colors.text);
        self.heading.text.color = Some(colors.text);

        self.heading_level_styles = Some(HeadingLevelStyles {
            h1: Some(TextStyleRefinement {
                font_size: Some(rems(1.45).into()),
                ..Default::default()
            }),
            h2: Some(TextStyleRefinement {
                font_size: Some(rems(1.3).into()),
                ..Default::default()
            }),
            h3: Some(TextStyleRefinement {
                font_size: Some(rems(1.1).into()),
                ..Default::default()
            }),
            h4: Some(TextStyleRefinement {
                font_size: Some(rems(1.01).into()),
                ..Default::default()
            }),
            h5: Some(TextStyleRefinement {
                font_size: Some(rems(0.95).into()),
                ..Default::default()
            }),
            h6: Some(TextStyleRefinement {
                font_size: Some(rems(0.85).into()),
                ..Default::default()
            }),
        });

        self.heading_border_color = Some(colors.border_variant);

        self
    }

    pub fn with_buffer_font(mut self, fonts: &MarkdownFontConfig) -> Self {
        self.base_text_style.font_family = fonts.buffer_font_family.clone();
        self.base_text_style.font_fallbacks = fonts.buffer_font_fallbacks.clone();
        self.base_text_style.font_features = fonts.buffer_font_features.clone();
        self.base_text_style.font_weight = fonts.buffer_font_weight;
        self
    }

    pub fn with_muted_text(mut self, cx: &App) -> Self {
        let colors = cx.theme().colors();
        self.base_text_style.color = colors.text_muted;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CopyButtonVisibility {
    Hidden,
    AlwaysVisible,
    VisibleOnHover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapButtonVisibility {
    Hidden,
    AlwaysVisible,
    VisibleOnHover,
}
