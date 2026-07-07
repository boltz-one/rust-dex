//! `boltz-markdown`: CommonMark/GFM parsing + rendering primitives.
//!
//! Layering: this crate must never depend on `boltz-ui` (see `controls.rs`
//! module docs) so that `boltz-ui` can depend on `boltz-markdown` without
//! creating a dependency cycle.

mod builder;
mod controls;
mod element;
mod entity;
mod html_rendering;
mod path_range;
mod rendered;
mod style;

pub mod parser;

pub use controls::*;
pub use element::{CodeSpanLinkCallback, MarkdownElement};
pub use entity::{
    AutoscrollBehavior, CodeBlockRenderFn, CodeBlockRenderer, CodeBlockTransformFn, Copy,
    CopyAsMarkdown, Markdown, MarkdownOptions, ParsedMarkdown,
};
pub use path_range::{LineCol, PathWithRange};
pub use style::*;
