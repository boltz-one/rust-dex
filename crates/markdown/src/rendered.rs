//! `RenderedLine`/`SourceMapping`/`RenderedMarkdown`/`RenderedText`/
//! `WrappedLineSegment`/`RenderedLink`/`RenderedFootnoteRef` + hit-testing.
//!
//! Design notes:
//! - `RenderedLine` has no `language: Option<Arc<Language>>` field: a
//!   per-language `default_scope()` would only be needed to configure
//!   [`RenderedText::surrounding_word_range`]'s `CharClassifier` (word vs.
//!   punctuation classification tuned per grammar, e.g. `$`/`#` counting as
//!   word characters in some languages). `boltz-markdown`'s minimal
//!   `language` crate (see `entity.rs`'s module docs) has no such per-language
//!   word-character configuration, so this file defines a small
//!   language-agnostic `char_kind` classifier instead (alphanumeric/`_` is a
//!   word char, everything else that isn't whitespace is punctuation).
//! - `WrappedLineSegment` collection uses `Vec` instead of `SmallVec<[_; 1]>`
//!   (avoids adding a `smallvec` dependency for a minor allocation
//!   optimization).

use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{Bounds, Pixels, Point, SharedString, TextAlign, TextLayout, WrappedLineLayout, point};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SourceMapping {
    pub(crate) rendered_index: usize,
    pub(crate) source_index: usize,
}

pub(crate) fn source_range_for_rendered(
    mappings: &[SourceMapping],
    rendered: &Range<usize>,
) -> Option<Range<usize>> {
    if rendered.start >= rendered.end {
        return None;
    }
    let start = source_index_for_rendered(mappings, rendered.start)?;
    let end = source_index_for_rendered(mappings, rendered.end - 1)? + 1;
    Some(start..end)
}

fn source_index_for_rendered(mappings: &[SourceMapping], rendered_index: usize) -> Option<usize> {
    let mut last: Option<&SourceMapping> = None;
    for mapping in mappings {
        if mapping.rendered_index <= rendered_index {
            last = Some(mapping);
        } else {
            break;
        }
    }
    last.map(|m| m.source_index + (rendered_index - m.rendered_index))
}

pub(crate) struct RenderedLine {
    pub(crate) layout: TextLayout,
    pub(crate) source_mappings: Vec<SourceMapping>,
    pub(crate) source_end: usize,
    pub(crate) text_align: TextAlign,
}

impl RenderedLine {
    fn rendered_index_for_source_index(&self, source_index: usize) -> usize {
        if source_index >= self.source_end {
            return self.layout.len();
        }

        let mapping = match self
            .source_mappings
            .binary_search_by_key(&source_index, |probe| probe.source_index)
        {
            Ok(ix) => &self.source_mappings[ix],
            Err(ix) => &self.source_mappings[ix - 1],
        };
        (mapping.rendered_index + (source_index - mapping.source_index)).min(self.layout.len())
    }

    fn source_index_for_rendered_index(&self, rendered_index: usize) -> usize {
        if rendered_index >= self.layout.len() {
            return self.source_end;
        }

        let mapping = match self
            .source_mappings
            .binary_search_by_key(&rendered_index, |probe| probe.rendered_index)
        {
            Ok(ix) => &self.source_mappings[ix],
            Err(ix) => &self.source_mappings[ix - 1],
        };
        mapping.source_index + (rendered_index - mapping.rendered_index)
    }

    /// Returns the source index for use as an exclusive range end at a word/selection boundary.
    /// When the rendered index is exactly at the start of a segment with a gap from the previous
    /// segment (e.g., after stripped markdown syntax like backticks), this returns the end of the
    /// previous segment rather than the start of the current one.
    fn source_index_for_exclusive_rendered_end(&self, rendered_index: usize) -> usize {
        if rendered_index >= self.layout.len() {
            return self.source_end;
        }

        let ix = match self
            .source_mappings
            .binary_search_by_key(&rendered_index, |probe| probe.rendered_index)
        {
            Ok(ix) => ix,
            Err(ix) => {
                return self.source_mappings[ix - 1].source_index
                    + (rendered_index - self.source_mappings[ix - 1].rendered_index);
            }
        };

        // Exact match at the start of a segment. Check if there's a gap from the previous segment.
        if ix > 0 {
            let prev_mapping = &self.source_mappings[ix - 1];
            let mapping = &self.source_mappings[ix];
            let prev_segment_len = mapping.rendered_index - prev_mapping.rendered_index;
            let prev_source_end = prev_mapping.source_index + prev_segment_len;
            if prev_source_end < mapping.source_index {
                return prev_source_end;
            }
        }

        self.source_mappings[ix].source_index
    }

    fn alignment_offset_for_segment(
        &self,
        available_width: Pixels,
        segment_start_x: Pixels,
        segment_end_x: Pixels,
    ) -> Pixels {
        let segment_width = segment_end_x - segment_start_x;
        match self.text_align {
            TextAlign::Left => Pixels::ZERO,
            TextAlign::Center => ((available_width - segment_width) / 2.).max(Pixels::ZERO),
            TextAlign::Right => (available_width - segment_width).max(Pixels::ZERO),
        }
    }

    pub(crate) fn source_index_for_position(
        &self,
        position: Point<Pixels>,
    ) -> Result<usize, usize> {
        let adjusted_position = (|| {
            if self.text_align == TextAlign::Left {
                return None;
            }

            let wrapped_line = self.layout.line_layout_for_index(0)?;

            let bounds = self.layout.bounds();
            let line_height = self.layout.line_height();
            let relative_y = (position.y - bounds.top()).max(Pixels::ZERO);
            let wrapped_row_ix = (relative_y / line_height) as usize;
            let boundaries = wrapped_line.wrap_boundaries();

            let segment_start_x = if wrapped_row_ix == 0 {
                Pixels::ZERO
            } else {
                boundaries
                    .get(wrapped_row_ix - 1)
                    .map(|b| {
                        wrapped_line.unwrapped_layout.runs[b.run_ix].glyphs[b.glyph_ix]
                            .position
                            .x
                    })
                    .unwrap_or(Pixels::ZERO)
            };
            let segment_end_x = boundaries
                .get(wrapped_row_ix)
                .map(|b| {
                    wrapped_line.unwrapped_layout.runs[b.run_ix].glyphs[b.glyph_ix]
                        .position
                        .x
                })
                .unwrap_or(wrapped_line.unwrapped_layout.width);

            let alignment_offset = self.alignment_offset_for_segment(
                bounds.size.width,
                segment_start_x,
                segment_end_x,
            );
            Some(point(position.x - alignment_offset, position.y))
        })()
        .unwrap_or(position);

        let line_rendered_index;
        let out_of_bounds;
        match self.layout.index_for_position(adjusted_position) {
            Ok(ix) => {
                line_rendered_index = ix;
                out_of_bounds = false;
            }
            Err(ix) => {
                line_rendered_index = ix;
                out_of_bounds = true;
            }
        };
        let source_index = self.source_index_for_rendered_index(line_rendered_index);
        if out_of_bounds {
            Err(source_index)
        } else {
            Ok(source_index)
        }
    }
}

pub struct RenderedMarkdown {
    pub(crate) element: gpui::AnyElement,
    pub(crate) text: RenderedText,
}

#[derive(Clone)]
pub(crate) struct RenderedText {
    pub(crate) lines: Rc<[RenderedLine]>,
    pub(crate) links: Rc<[RenderedLink]>,
    pub(crate) footnote_refs: Rc<[RenderedFootnoteRef]>,
}

struct WrappedLineSegment {
    start: usize,
    end: usize,
    row_top: Pixels,
    layout: Arc<WrappedLineLayout>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct RenderedLink {
    pub(crate) source_range: Range<usize>,
    pub(crate) destination_url: SharedString,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct RenderedFootnoteRef {
    pub(crate) source_range: Range<usize>,
    pub(crate) label: SharedString,
}

/// Language-agnostic word/punctuation classifier used by
/// [`RenderedText::surrounding_word_range`]. See module docs for why this
/// doesn't consult a per-language `word_characters` set.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum CharKind {
    Whitespace,
    Punctuation,
    Word,
}

fn char_kind(c: char) -> CharKind {
    if c.is_whitespace() {
        CharKind::Whitespace
    } else if c.is_alphanumeric() || c == '_' {
        CharKind::Word
    } else {
        CharKind::Punctuation
    }
}

impl RenderedText {
    pub(crate) fn bounds_for_source_range(&self, range: Range<usize>) -> Vec<Bounds<Pixels>> {
        self.bounds_for_sorted_source_ranges([(0, range)])
            .into_iter()
            .map(|(_, bounds)| bounds)
            .collect()
    }

    pub(crate) fn bounds_for_sorted_source_ranges(
        &self,
        ranges: impl IntoIterator<Item = (usize, Range<usize>)>,
    ) -> Vec<(usize, Bounds<Pixels>)> {
        let ranges = ranges.into_iter().collect::<Vec<_>>();
        let mut all_bounds = Vec::new();
        let mut first_possible_range_ix = 0;

        for line in self.lines.iter() {
            let line_source_start = line.source_mappings.first().unwrap().source_index;
            while ranges
                .get(first_possible_range_ix)
                .is_some_and(|(_, range)| range.end <= line_source_start)
            {
                first_possible_range_ix += 1;
            }

            let Some((_, first_possible_range)) = ranges.get(first_possible_range_ix) else {
                break;
            };
            if first_possible_range.start >= line.source_end {
                continue;
            }

            let wrapped_line_segments = Self::wrapped_line_segments(line);
            if wrapped_line_segments.is_empty() {
                continue;
            }

            let mut range_ix = first_possible_range_ix;
            while let Some((highlight_ix, range)) = ranges.get(range_ix) {
                if range.start >= line.source_end {
                    break;
                }
                Self::push_bounds_for_line_source_range(
                    &mut all_bounds,
                    *highlight_ix,
                    line,
                    &wrapped_line_segments,
                    range.start.max(line_source_start)..range.end.min(line.source_end),
                );
                range_ix += 1;
            }
        }

        all_bounds
    }

    fn wrapped_line_segments(line: &RenderedLine) -> Vec<WrappedLineSegment> {
        let layout = &line.layout;
        let line_height = layout.line_height();
        let mut row_top = layout.bounds().top();
        let mut wrapped_line_start = 0;
        let mut segments = Vec::new();

        for wrapped_line in layout.line_layouts() {
            let wrapped_line_end = wrapped_line_start + wrapped_line.len();
            let wrapped_line_height = wrapped_line.size(line_height).height;
            segments.push(WrappedLineSegment {
                start: wrapped_line_start,
                end: wrapped_line_end,
                row_top,
                layout: wrapped_line,
            });
            row_top += wrapped_line_height;
            wrapped_line_start = wrapped_line_end + 1;
        }

        segments
    }

    fn push_bounds_for_line_source_range(
        all_bounds: &mut Vec<(usize, Bounds<Pixels>)>,
        highlight_ix: usize,
        line: &RenderedLine,
        wrapped_line_segments: &[WrappedLineSegment],
        range: Range<usize>,
    ) {
        if range.start >= range.end {
            return;
        }

        let layout = &line.layout;
        let line_bounds = layout.bounds();
        let line_height = layout.line_height();

        let rendered_start = line.rendered_index_for_source_index(range.start);
        let rendered_end = line.rendered_index_for_source_index(range.end);

        for wrapped_line_segment in wrapped_line_segments {
            if wrapped_line_segment.start >= rendered_end {
                break;
            }
            if wrapped_line_segment.end <= rendered_start {
                continue;
            }

            let wrapped_line = &wrapped_line_segment.layout;
            let unwrapped_layout = &wrapped_line.unwrapped_layout;
            let wrapped_line_start = wrapped_line_segment.start;
            let wrapped_line_end = wrapped_line_segment.end;
            let mut row_top = wrapped_line_segment.row_top;

            let row_ends = wrapped_line
                .wrap_boundaries()
                .iter()
                .map(|wrap_boundary| {
                    let glyph =
                        &unwrapped_layout.runs[wrap_boundary.run_ix].glyphs[wrap_boundary.glyph_ix];
                    (wrapped_line_start + glyph.index, glyph.position.x)
                })
                .chain([(wrapped_line_end, unwrapped_layout.width)]);

            let mut row_start = wrapped_line_start;
            let mut row_start_x = Pixels::ZERO;

            for (row_end, row_end_x) in row_ends {
                let selection_start = rendered_start.max(row_start);
                let selection_end = rendered_end.min(row_end);

                if selection_start < selection_end {
                    let alignment_offset = line.alignment_offset_for_segment(
                        line_bounds.size.width,
                        row_start_x,
                        row_end_x,
                    );
                    let x_for_index = |index| {
                        line_bounds.left()
                            + alignment_offset
                            + unwrapped_layout.x_for_index(index - wrapped_line_start)
                            - row_start_x
                    };
                    all_bounds.push((
                        highlight_ix,
                        Bounds::from_corners(
                            point(x_for_index(selection_start), row_top),
                            point(x_for_index(selection_end), row_top + line_height),
                        ),
                    ));
                }

                row_start = row_end;
                row_start_x = row_end_x;
                row_top += line_height;
            }
        }
    }

    pub(crate) fn source_index_for_position(
        &self,
        position: Point<Pixels>,
    ) -> Result<usize, usize> {
        let mut lines = self.lines.iter().peekable();
        let mut fallback_line: Option<&RenderedLine> = None;

        while let Some(line) = lines.next() {
            let line_bounds = line.layout.bounds();

            // Exact match: position is within bounds (handles overlapping bounds like table columns)
            if line_bounds.contains(&position) {
                return line.source_index_for_position(position);
            }

            // Track fallback for Y-coordinate based matching
            if position.y <= line_bounds.bottom() && fallback_line.is_none() {
                fallback_line = Some(line);
            }

            // Handle gap between lines
            if position.y > line_bounds.bottom()
                && let Some(next_line) = lines.peek()
                && position.y < next_line.layout.bounds().top()
            {
                return Err(line.source_end);
            }
        }

        // Fall back to Y-coordinate matched line
        if let Some(line) = fallback_line {
            return line.source_index_for_position(position);
        }

        Err(self.lines.last().map_or(0, |line| line.source_end))
    }

    pub(crate) fn position_for_source_index(
        &self,
        source_index: usize,
    ) -> Option<(Point<Pixels>, Pixels)> {
        for line in self.lines.iter() {
            let line_source_start = line.source_mappings.first().unwrap().source_index;
            if source_index < line_source_start {
                break;
            } else if source_index > line.source_end {
                continue;
            } else {
                let line_height = line.layout.line_height();
                let rendered_index_within_line = line.rendered_index_for_source_index(source_index);
                let position = line.layout.position_for_index(rendered_index_within_line)?;
                return Some((position, line_height));
            }
        }
        None
    }

    pub(crate) fn surrounding_word_range(&self, source_index: usize) -> Range<usize> {
        for line in self.lines.iter() {
            if source_index > line.source_end {
                continue;
            }

            let line_rendered_start = line.source_mappings.first().unwrap().rendered_index;
            let rendered_index_in_line =
                line.rendered_index_for_source_index(source_index) - line_rendered_start;
            let text = line.layout.text();

            let mut prev_chars = text[..rendered_index_in_line].chars().rev().peekable();
            let mut next_chars = text[rendered_index_in_line..].chars().peekable();

            let word_kind = std::cmp::max(
                prev_chars.peek().map(|&c| char_kind(c)),
                next_chars.peek().map(|&c| char_kind(c)),
            );

            let mut start = rendered_index_in_line;
            for c in prev_chars {
                if Some(char_kind(c)) == word_kind {
                    start -= c.len_utf8();
                } else {
                    break;
                }
            }

            let mut end = rendered_index_in_line;
            for c in next_chars {
                if Some(char_kind(c)) == word_kind {
                    end += c.len_utf8();
                } else {
                    break;
                }
            }

            return line.source_index_for_rendered_index(line_rendered_start + start)
                ..line.source_index_for_exclusive_rendered_end(line_rendered_start + end);
        }

        source_index..source_index
    }

    pub(crate) fn surrounding_line_range(&self, source_index: usize) -> Range<usize> {
        for line in self.lines.iter() {
            if source_index > line.source_end {
                continue;
            }
            let line_source_start = line.source_mappings.first().unwrap().source_index;
            return line_source_start..line.source_end;
        }

        source_index..source_index
    }

    pub(crate) fn text_for_range(&self, range: Range<usize>) -> String {
        let mut accumulator = String::new();

        for line in self.lines.iter() {
            if range.start > line.source_end {
                continue;
            }
            let line_source_start = line.source_mappings.first().unwrap().source_index;
            if range.end < line_source_start {
                break;
            }

            let text = line.layout.text();

            let start = if range.start < line_source_start {
                0
            } else {
                line.rendered_index_for_source_index(range.start)
            };
            let end = if range.end > line.source_end {
                line.rendered_index_for_source_index(line.source_end)
            } else {
                line.rendered_index_for_source_index(range.end)
            }
            .min(text.len());

            accumulator.push_str(&text[start..end]);
            accumulator.push('\n');
        }
        // Remove trailing newline
        accumulator.pop();
        accumulator
    }

    pub(crate) fn link_for_source_index(&self, source_index: usize) -> Option<&RenderedLink> {
        self.links
            .iter()
            .find(|link| link.source_range.contains(&source_index))
    }

    pub(crate) fn footnote_ref_for_source_index(
        &self,
        source_index: usize,
    ) -> Option<&RenderedFootnoteRef> {
        self.footnote_refs
            .iter()
            .find(|fref| fref.source_range.contains(&source_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::MarkdownElement;
    use crate::entity::{Markdown, MarkdownOptions};
    use crate::style::MarkdownStyle;
    use gpui::{
        AppContext as _, Context, IntoElement, Render, TestAppContext, Window, div, px, size,
    };

    struct TestWindow;

    impl Render for TestWindow {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    fn ensure_theme_initialized(cx: &mut TestAppContext) {
        cx.update(|cx| {
            if !cx.has_global::<theme::GlobalTheme>() {
                theme::init(theme::LoadThemes::JustBase, cx);
            }
        });
    }

    fn render_markdown(markdown: &str, cx: &mut TestAppContext) -> RenderedText {
        render_markdown_with_options(markdown, MarkdownOptions::default(), cx)
    }

    fn render_markdown_with_options(
        markdown: &str,
        options: MarkdownOptions,
        cx: &mut TestAppContext,
    ) -> RenderedText {
        ensure_theme_initialized(cx);
        let (_, cx) = cx.add_window_view(|_, _| TestWindow);
        let markdown =
            cx.new(|cx| Markdown::new_with_options(markdown.to_string().into(), options, cx));
        cx.run_until_parked();
        let (rendered, _) = cx.draw(
            Default::default(),
            size(px(600.0), px(600.0)),
            |_window, _cx| MarkdownElement::new(markdown, MarkdownStyle::default()),
        );
        rendered.text
    }

    #[track_caller]
    fn assert_mappings(rendered: &RenderedText, expected: Vec<Vec<(usize, usize)>>) {
        assert_eq!(rendered.lines.len(), expected.len(), "line count mismatch");
        for (line_ix, line_mappings) in expected.into_iter().enumerate() {
            let line = &rendered.lines[line_ix];

            assert!(
                line.source_mappings.windows(2).all(|mappings| {
                    mappings[0].source_index < mappings[1].source_index
                        && mappings[0].rendered_index < mappings[1].rendered_index
                }),
                "line {} has duplicate mappings: {:?}",
                line_ix,
                line.source_mappings
            );

            for (rendered_ix, source_ix) in line_mappings {
                assert_eq!(
                    line.source_index_for_rendered_index(rendered_ix),
                    source_ix,
                    "line {}, rendered_ix {}",
                    line_ix,
                    rendered_ix
                );

                assert_eq!(
                    line.rendered_index_for_source_index(source_ix),
                    rendered_ix,
                    "line {}, source_ix {}",
                    line_ix,
                    source_ix
                );
            }
        }
    }

    #[gpui::test]
    fn test_mappings(cx: &mut TestAppContext) {
        // Formatting.
        assert_mappings(
            &render_markdown("He*l*lo", cx),
            vec![vec![(0, 0), (1, 1), (2, 3), (3, 5), (4, 6), (5, 7)]],
        );

        // Multiple lines.
        assert_mappings(
            &render_markdown("Hello\n\nWorld", cx),
            vec![
                vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5)],
                vec![(0, 7), (1, 8), (2, 9), (3, 10), (4, 11), (5, 12)],
            ],
        );

        // Multi-byte characters.
        assert_mappings(
            &render_markdown("αβγ\n\nδεζ", cx),
            vec![
                vec![(0, 0), (2, 2), (4, 4), (6, 6)],
                vec![(0, 8), (2, 10), (4, 12), (6, 14)],
            ],
        );

        // Smart quotes.
        assert_mappings(&render_markdown("\"", cx), vec![vec![(0, 0), (3, 1)]]);
        assert_mappings(
            &render_markdown("\"hey\"", cx),
            vec![vec![(0, 0), (3, 1), (4, 2), (5, 3), (6, 4), (9, 5)]],
        );

        // HTML Comments are ignored
        assert_mappings(
            &render_markdown(
                "<!--\nrdoc-file=string.c\n- str.intern   -> symbol\n- str.to_sym   -> symbol\n-->\nReturns",
                cx,
            ),
            vec![vec![
                (0, 78),
                (1, 79),
                (2, 80),
                (3, 81),
                (4, 82),
                (5, 83),
                (6, 84),
            ]],
        );
    }

    #[gpui::test]
    fn test_surrounding_word_range(cx: &mut TestAppContext) {
        let rendered = render_markdown("Hello world tesεζ", cx);

        let word_range = rendered.surrounding_word_range(2);
        assert_eq!(rendered.text_for_range(word_range), "Hello");

        let word_range = rendered.surrounding_word_range(7);
        assert_eq!(rendered.text_for_range(word_range), "world");

        let word_range = rendered.surrounding_word_range(14);
        assert_eq!(rendered.text_for_range(word_range), "tesεζ");

        let word_range = rendered.surrounding_word_range(5);
        assert_eq!(rendered.text_for_range(word_range), "Hello");
    }

    #[gpui::test]
    fn test_surrounding_line_range(cx: &mut TestAppContext) {
        let rendered = render_markdown("First line\n\nSecond line\n\nThird lineεζ", cx);

        let line_range = rendered.surrounding_line_range(5);
        assert_eq!(rendered.text_for_range(line_range), "First line");

        let line_range = rendered.surrounding_line_range(13);
        assert_eq!(rendered.text_for_range(line_range), "Second line");

        let line_range = rendered.surrounding_line_range(37);
        assert_eq!(rendered.text_for_range(line_range), "Third lineεζ");
    }

    #[gpui::test]
    fn test_table_column_selection(cx: &mut TestAppContext) {
        let rendered = render_markdown("| a | b |\n|---|---|\n| c | d |", cx);

        assert!(rendered.lines.len() >= 2);
        let first_bounds = rendered.lines[0].layout.bounds();
        let second_bounds = rendered.lines[1].layout.bounds();

        let first_index = match rendered.source_index_for_position(first_bounds.center()) {
            Ok(index) | Err(index) => index,
        };
        let second_index = match rendered.source_index_for_position(second_bounds.center()) {
            Ok(index) | Err(index) => index,
        };

        let first_word = rendered.text_for_range(rendered.surrounding_word_range(first_index));
        let second_word = rendered.text_for_range(rendered.surrounding_word_range(second_index));

        assert_eq!(first_word, "a");
        assert_eq!(second_word, "b");
    }

    #[test]
    fn test_source_range_for_rendered_handles_split_chunks() {
        let mappings = vec![
            SourceMapping {
                rendered_index: 0,
                source_index: 20,
            },
            SourceMapping {
                rendered_index: 1,
                source_index: 21,
            },
            SourceMapping {
                rendered_index: 2,
                source_index: 22,
            },
        ];

        let range = source_range_for_rendered(&mappings, &(0..3)).unwrap();
        assert_eq!(range, 20..23);

        let range = source_range_for_rendered(&mappings, &(1..2)).unwrap();
        assert_eq!(range, 21..22);

        assert_eq!(source_range_for_rendered(&mappings, &(2..2)), None);
    }

    #[gpui::test]
    fn test_inline_code_word_selection_excludes_backticks(cx: &mut TestAppContext) {
        let rendered = render_markdown("use `blah` here", cx);
        let word_range = rendered.surrounding_word_range(6);
        assert_eq!(rendered.text_for_range(word_range.clone()), "blah");
        assert_eq!(word_range, 5..9);
    }

    #[gpui::test]
    fn test_surrounding_word_range_respects_word_characters(cx: &mut TestAppContext) {
        let rendered = render_markdown("foo.bar() baz", cx);

        let word_range = rendered.surrounding_word_range(0);
        assert_eq!(rendered.text_for_range(word_range), "foo");

        let word_range = rendered.surrounding_word_range(4);
        assert_eq!(rendered.text_for_range(word_range), "bar");

        let word_range = rendered.surrounding_word_range(10);
        assert_eq!(rendered.text_for_range(word_range), "baz");
    }

    #[gpui::test]
    fn test_link_detected_for_source_index(cx: &mut TestAppContext) {
        let rendered = render_markdown("[Click here](https://example.com)", cx);

        assert_eq!(rendered.links.len(), 1);
        assert_eq!(rendered.links[0].destination_url, "https://example.com");

        let link = rendered.link_for_source_index(1);
        assert!(link.is_some());
        assert_eq!(link.unwrap().destination_url, "https://example.com");

        let past_end = rendered.links[0].source_range.end;
        assert!(rendered.link_for_source_index(past_end).is_none());
    }

    #[gpui::test]
    fn test_link_for_source_index_ignores_plain_text(cx: &mut TestAppContext) {
        let rendered = render_markdown("Hello world", cx);

        assert!(rendered.links.is_empty());
        assert!(rendered.link_for_source_index(0).is_none());
        assert!(rendered.link_for_source_index(5).is_none());
    }

    #[gpui::test]
    fn test_bounds_for_source_range_skips_gaps_between_rendered_lines(cx: &mut TestAppContext) {
        let source = "First\n\nSecond";
        let rendered = render_markdown(source, cx);
        let highlight_bounds = rendered.bounds_for_source_range(0..source.len());
        assert_eq!(highlight_bounds.len(), rendered.lines.len());

        for (line, highlight_bounds) in rendered.lines.iter().zip(highlight_bounds.iter()) {
            let line_bounds = line.layout.bounds();
            assert_eq!(highlight_bounds.top(), line_bounds.top());
            assert_eq!(
                highlight_bounds.bottom(),
                line_bounds.top() + line.layout.line_height()
            );
        }
    }

    #[gpui::test]
    fn test_heading_font_sizes_are_distinct(cx: &mut TestAppContext) {
        let rendered = render_markdown("# H1\n\n## H2\n\n### H3\n\nBody text", cx);

        assert!(
            rendered.lines.len() >= 4,
            "expected at least 4 rendered lines, got {}",
            rendered.lines.len()
        );

        let h1_line_height = rendered.lines[0].layout.line_height();
        let h2_line_height = rendered.lines[1].layout.line_height();
        let h3_line_height = rendered.lines[2].layout.line_height();
        let body_line_height = rendered.lines[3].layout.line_height();

        assert!(h1_line_height > h2_line_height);
        assert!(h2_line_height > h3_line_height);
        assert!(h3_line_height > body_line_height);
    }
}
