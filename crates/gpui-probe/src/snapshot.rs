//! Deterministic tree-text snapshots of the tracked-element registry (Phase 04).
//!
//! Serialization-free by design: this turns an [`ElementTree`] into a stable,
//! sorted text form. Tests feed the result to `insta::assert_snapshot!` — insta
//! is a `[dev-dependencies]`, kept out of the published library's graph.
//!
//! Bounds are excluded by default (they vary by OS/DPI/font rendering, which is
//! not meaningful signal for a tree-STRUCTURE snapshot); opt in via
//! [`SnapshotRedactions::include_bounds`], which rounds to 10px — never raw floats.

use crate::registry::ElementTree;

/// Controls what a snapshot includes or omits.
#[derive(Clone, Debug, Default)]
pub struct SnapshotRedactions {
    redact: Vec<String>,
    include_bounds: bool,
}

impl SnapshotRedactions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Omit any entry whose id contains `pattern` (e.g. dynamic spinners/timestamps).
    pub fn redact_by_id(mut self, pattern: impl Into<String>) -> Self {
        self.redact.push(pattern.into());
        self
    }

    /// Include coarse bounds (rounded to nearest 10px) for the rare geometry
    /// assertion. Off by default so snapshots stay cross-environment stable.
    pub fn include_bounds(mut self) -> Self {
        self.include_bounds = true;
        self
    }

    fn is_redacted(&self, id: &str) -> bool {
        self.redact.iter().any(|p| id.contains(p.as_str()))
    }
}

/// Render `tree` as deterministic, sorted tree-text. One line per element:
/// `"<id> enabled=<bool>"`, plus ` bounds=<x>,<y>,<w>,<h>` (10px-rounded) when
/// [`SnapshotRedactions::include_bounds`] is set. Sorted by id (registry
/// iteration order is not stable). The tree is flat today (ADR 0007).
pub fn tree_text(tree: &ElementTree, redactions: &SnapshotRedactions) -> String {
    let mut lines: Vec<String> = tree
        .roots
        .iter()
        .filter(|n| !redactions.is_redacted(&n.id))
        .map(|n| {
            let mut line = format!("{} enabled={}", n.id, n.enabled);
            if redactions.include_bounds {
                let b = n.bounds;
                line.push_str(&format!(
                    " bounds={},{},{},{}",
                    round10(b.origin.x),
                    round10(b.origin.y),
                    round10(b.size.width),
                    round10(b.size.height),
                ));
            }
            line
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

fn round10(p: gpui::Pixels) -> i32 {
    ((f32::from(p) / 10.0).round() as i32) * 10
}
