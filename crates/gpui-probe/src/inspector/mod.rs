//! Read-only inspector overlay (Chunk C5 / MVP). Entirely gated behind the
//! crate's `inspector` feature — see `lib.rs`, which only declares this
//! module under `#[cfg(feature = "inspector")]`.
//!
//! - [`overlay::InspectorOverlay`] — a `Render`-able overlay that
//!   hover-highlights the innermost tracked element under the mouse, reading
//!   live data from [`crate::registry::ElementRegistry`].
//! - [`panel`] — the side-panel element-list renderer, split out for
//!   readability.
//!
//! This is display-only: no click-through / hit-testing (that's Phase 06,
//! ADR 0007).

pub mod overlay;
pub mod panel;

pub use overlay::InspectorOverlay;
