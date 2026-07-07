mod html_minifier;
pub(crate) mod html_parser;

// Note: HTML block rendering into `MarkdownElementBuilder`/`Div` output lives
// in the crate-level `html_rendering` module (`crate::html_rendering`), not
// here, since it depends on `element.rs`/`builder.rs`.
