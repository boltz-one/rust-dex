//! Minimal tree-sitter language registry + highlight query runner.
//!
//! Deliberately does NOT port Zed's `language` crate (22.9kLOC) — ~70% of it
//! is `LanguageServer`/`Capability`/diagnostics glue for LSP, which is out of
//! scope for every phase of this plan (see
//! `plans/20260705-1722-zed-ui-component-enrichment/plan.md`). Only the
//! rope+tree-sitter binding — the ~30% actually needed for syntax
//! highlighting — is reproduced here, written fresh rather than extracted
//! from Zed's source (that source is entangled with the LSP types being cut).

mod highlight;

pub use highlight::highlighted_spans;

/// A single language: its tree-sitter grammar plus a compiled highlight
/// query. Construction fails only if `highlight_query_source` doesn't parse
/// against `grammar` — a bug in the bundled query, not user input, so a
/// caller-facing `Result` would just be unwrapped everywhere; panicking here
/// surfaces the bug immediately in development instead.
pub struct Language {
    name: &'static str,
    pub(crate) grammar: tree_sitter::Language,
    pub(crate) highlight_query: tree_sitter::Query,
}

impl Language {
    pub fn new(
        name: &'static str,
        grammar: tree_sitter::Language,
        highlight_query_source: &str,
    ) -> Self {
        let highlight_query = tree_sitter::Query::new(&grammar, highlight_query_source)
            .unwrap_or_else(|error| {
                panic!("bundled highlight query for {name} failed to parse: {error}")
            });
        Self {
            name,
            grammar,
            highlight_query,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Resolves a [`Language`] by file extension (no leading dot, e.g. `"rs"`).
/// Intentionally minimal: no shebang detection, no content sniffing — see
/// Zed's `LanguageRegistry` for that scope, out of bounds here.
pub trait LanguageRegistry {
    fn language_for_extension(&self, extension: &str) -> Option<&Language>;
}

/// The default registry: exactly the grammars compiled in via Cargo
/// features (see this crate's `Cargo.toml` — `lang-rust` is on by default
/// as the minimal working example; every other grammar is opt-in so
/// consumers only pay its ~200-300KB binary-size cost if they enable it).
pub struct DefaultLanguageRegistry {
    #[cfg(feature = "lang-rust")]
    rust: Language,
    #[cfg(feature = "lang-javascript")]
    javascript: Language,
    #[cfg(feature = "lang-typescript")]
    typescript: Language,
    #[cfg(feature = "lang-typescript")]
    tsx: Language,
    #[cfg(feature = "lang-markdown")]
    markdown: Language,
    #[cfg(feature = "lang-json")]
    json: Language,
}

impl DefaultLanguageRegistry {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "lang-rust")]
            rust: Language::new(
                "Rust",
                tree_sitter_rust::LANGUAGE.into(),
                tree_sitter_rust::HIGHLIGHTS_QUERY,
            ),
            #[cfg(feature = "lang-javascript")]
            javascript: Language::new(
                "JavaScript",
                tree_sitter_javascript::LANGUAGE.into(),
                tree_sitter_javascript::HIGHLIGHT_QUERY,
            ),
            #[cfg(feature = "lang-typescript")]
            typescript: Language::new(
                "TypeScript",
                tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            ),
            #[cfg(feature = "lang-typescript")]
            tsx: Language::new(
                "TSX",
                tree_sitter_typescript::LANGUAGE_TSX.into(),
                tree_sitter_typescript::HIGHLIGHTS_QUERY,
            ),
            // Markdown's tree-sitter grammar splits block/inline parsing into
            // two separate grammars; only the block grammar is wired here
            // (headings, lists, code fences) — inline emphasis/link spans
            // inside a block need the second `inline_language()` grammar
            // layered on top, which is out of scope for this minimal pass.
            #[cfg(feature = "lang-markdown")]
            markdown: Language::new(
                "Markdown",
                tree_sitter_md::LANGUAGE.into(),
                tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            ),
            #[cfg(feature = "lang-json")]
            json: Language::new(
                "JSON",
                tree_sitter_json::LANGUAGE.into(),
                tree_sitter_json::HIGHLIGHTS_QUERY,
            ),
        }
    }
}

impl Default for DefaultLanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRegistry for DefaultLanguageRegistry {
    fn language_for_extension(&self, extension: &str) -> Option<&Language> {
        match extension {
            #[cfg(feature = "lang-rust")]
            "rs" => Some(&self.rust),
            #[cfg(feature = "lang-javascript")]
            "js" | "mjs" | "cjs" | "jsx" => Some(&self.javascript),
            #[cfg(feature = "lang-typescript")]
            "ts" | "mts" | "cts" => Some(&self.typescript),
            #[cfg(feature = "lang-typescript")]
            "tsx" => Some(&self.tsx),
            #[cfg(feature = "lang-markdown")]
            "md" | "markdown" => Some(&self.markdown),
            #[cfg(feature = "lang-json")]
            "json" => Some(&self.json),
            _ => None,
        }
    }
}
