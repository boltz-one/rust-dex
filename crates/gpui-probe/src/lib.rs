//! `gpui-probe` — a shared element-tree core for a GPUI inspector overlay and
//! an in-process UI test driver, built on top of `boltz-gpui`.
//!
//! The crate exposes a single, shared representation of the rendered element
//! tree so that two consumers can be built on top of it without duplicating
//! traversal logic:
//!
//! - An **inspector overlay/panel** for interactively browsing the live
//!   element tree of a running app (see the `inspector` feature).
//! - An **in-process UI test driver** for writing assertions and driving
//!   interactions against the same element tree (see the `semantic` feature
//!   for `Role`/`Label` based selectors).
//!
//! # License boundary
//!
//! This crate MUST NOT depend on `boltz-theme`, `boltz-ui`, or `boltz-icons`.
//! It only depends on `boltz-gpui` (and general-purpose utility crates), so
//! that it can be published and reused independently of the app-specific
//! theme/component/icon layers.
//!
//! # Architecture decisions
//!
//! See the ADRs in `base/docs/decisions/`:
//! - `0007-element-tree-access-public-registry-primary.md`
//! - `0008-selector-model-test-id-first-hybrid.md`
//! - `0009-driver-topology-in-process-dual-backend.md`

pub mod driver;
pub mod registry;
pub mod snapshot;
pub mod track;

#[cfg(feature = "inspector")]
pub mod inspector;
#[cfg(feature = "inspector")]
pub use inspector::InspectorOverlay;

pub use driver::{ActionabilityStage, Locator, ProbeError, WaitConfig, find_by_test_id};
#[cfg(any(test, feature = "test-support"))]
pub use driver::{ElementHandle, TestHarness};
pub use registry::{ElementNode, ElementRegistry, ElementSnapshot, ElementTree};
pub use snapshot::{SnapshotRedactions, tree_text};
pub use track::{Trackable, track};
