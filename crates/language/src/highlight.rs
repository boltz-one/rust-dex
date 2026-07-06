use std::ops::Range;

use crate::Language;

/// Returns a flat, non-overlapping list of (byte range, tree-sitter capture
/// name) spans covering every highlighted region of `source`. Gaps between
/// spans are the caller's responsibility to render in the buffer's default
/// text color — this never claims to cover the whole string.
///
/// Capture ranges from a tree-sitter highlight query can nest (e.g. a
/// `function.method` call's name sits inside the enclosing `function`
/// capture). This picks the SMALLEST enclosing capture per sub-span, which
/// matches the common highlight-query convention where more specific
/// (deeper-AST) patterns produce narrower ranges than general ones.
pub fn highlighted_spans(language: &Language, source: &str) -> Vec<(Range<usize>, String)> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language.grammar).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };

    let capture_names = language.highlight_query.capture_names();
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut captures: Vec<(Range<usize>, String)> = Vec::new();
    let mut matches = cursor.matches(
        &language.highlight_query,
        tree.root_node(),
        source.as_bytes(),
    );
    while let Some(m) = tree_sitter::StreamingIterator::next(&mut matches) {
        for capture in m.captures {
            let name = capture_names[capture.index as usize];
            captures.push((capture.node.byte_range(), name.to_string()));
        }
    }

    resolve_overlaps(captures, source.len())
}

/// Splits `captures` at every distinct boundary point, keeping — for each
/// resulting sub-span — the narrowest capture that fully contains it.
fn resolve_overlaps(
    captures: Vec<(Range<usize>, String)>,
    len: usize,
) -> Vec<(Range<usize>, String)> {
    if captures.is_empty() {
        return Vec::new();
    }

    let mut boundaries: Vec<usize> = captures
        .iter()
        .flat_map(|(r, _)| [r.start, r.end])
        .collect();
    boundaries.push(0);
    boundaries.push(len);
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut spans = Vec::new();
    for window in boundaries.windows(2) {
        let (start, end) = (window[0], window[1]);
        if start >= end {
            continue;
        }
        let mid = start + (end - start) / 2;
        let narrowest = captures
            .iter()
            .filter(|(range, _)| range.start <= mid && mid < range.end)
            .min_by_key(|(range, _)| range.end - range.start);
        if let Some((_, name)) = narrowest {
            spans.push((start..end, name.clone()));
        }
    }
    spans
}

#[cfg(test)]
#[cfg(feature = "lang-rust")]
mod tests {
    use crate::{DefaultLanguageRegistry, LanguageRegistry};

    #[test]
    fn highlights_rust_keywords_and_strings() {
        let registry = DefaultLanguageRegistry::new();
        let rust = registry.language_for_extension("rs").unwrap();
        let source = r#"fn main() { let s = "hi"; }"#;
        let spans = super::highlighted_spans(rust, source);

        assert!(!spans.is_empty(), "expected at least one highlight span");
        assert!(
            spans
                .iter()
                .any(|(range, name)| &source[range.clone()] == "fn" && name.starts_with("keyword")),
            "expected `fn` tagged as a keyword capture, got: {spans:?}"
        );
        assert!(
            spans
                .iter()
                .any(|(range, name)| &source[range.clone()] == "\"hi\""
                    && name.starts_with("string")),
            "expected the string literal tagged as a string capture, got: {spans:?}"
        );
    }

    #[test]
    fn empty_source_has_no_spans() {
        let registry = DefaultLanguageRegistry::new();
        let rust = registry.language_for_extension("rs").unwrap();
        assert!(super::highlighted_spans(rust, "").is_empty());
    }
}
