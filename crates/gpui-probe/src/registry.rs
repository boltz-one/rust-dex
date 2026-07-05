//! The shared element-tree data model + the app-wide [`ElementRegistry`] global.
//!
//! Both consumers of this crate — the in-process test driver and the inspector
//! overlay — read the same [`ElementRegistry`]. It is populated by
//! [`crate::track`]'s `canvas()` paint closure (see ADR 0007), one entry per
//! opted-in (`.probe(id)`) element, refreshed every frame.

use std::borrow::Borrow;

use gpui::{Bounds, Global, Pixels, SharedString};
use rustc_hash::FxHashMap;

/// A single tracked element's state as of the frame it was last painted in.
#[derive(Clone, Debug)]
pub struct ElementSnapshot {
    /// The `test_id` the element was `.probe()`-ed with.
    pub id: SharedString,
    /// Real screen-space bounds captured via `gpui::canvas()` on the last paint.
    pub bounds: Bounds<Pixels>,
    /// App-defined enabled flag supplied by the caller (GPUI has no universal
    /// "enabled" concept — see ADR 0007).
    pub enabled: bool,
    /// The frame this snapshot was recorded in; compared against
    /// [`ElementRegistry::current_frame_seq`] to detect staleness.
    frame_seq: u64,
}

/// One node in the shared [`ElementTree`]. For now the tree is flat (children
/// always empty) — hierarchy capture is a documented later extension (ADR 0007
/// / Phase 02 notes); `Locator::TestId` lookups are flat by nature.
#[derive(Clone, Debug)]
pub struct ElementNode {
    pub id: SharedString,
    pub bounds: Bounds<Pixels>,
    pub enabled: bool,
    pub children: Vec<ElementNode>,
}

/// A serializable snapshot of the currently-visible tracked elements.
#[derive(Clone, Debug, Default)]
pub struct ElementTree {
    pub roots: Vec<ElementNode>,
}

/// App-wide registry of tracked elements. Stored as a `gpui::Global` so it
/// composes with GPUI's update/effect cycle and is reachable from any
/// `&mut App`/`&mut Window` callback without extra plumbing.
///
/// Staleness model: [`begin_frame`](Self::begin_frame) advances a monotonic
/// counter; [`upsert`](Self::upsert) stamps the current counter onto each
/// entry as it is (re)painted; reads ([`get`](Self::get)/
/// [`all_visible`](Self::all_visible)) ignore any entry not stamped with the
/// current counter. A reader that wants fresh visibility calls `begin_frame()`
/// before triggering the render pass it intends to observe.
///
/// LIMITATION — staleness only prunes when something calls `begin_frame()`.
/// The [`TestHarness`](crate::driver::TestHarness) does so on every step, so
/// the driver sees correct present/absent results. A PASSIVE reader that never
/// calls `begin_frame()` (e.g. an app that only mounts the inspector overlay)
/// leaves the counter fixed, so every entry reads as "current": unmounted
/// elements are reported with their LAST-KNOWN bounds and are not auto-pruned.
/// GPUI exposes no public per-frame hook to fix this automatically here; real
/// occlusion/liveness is Phase 06's `hit_test` job (ADR 0007). Because probe
/// ids are compile-time literals on a bounded set of call sites, the map size
/// stays bounded (re-probing an id overwrites in place) — it is not a leak for
/// static id sets, but dynamically-generated ids would accumulate.
///
/// LIMITATION — the registry is a single App-wide `Global`, NOT per-`Window`.
/// Two open windows in the same `App` that probe the same id collide: the
/// last painted wins, silently. Keep probe ids unique across all open windows,
/// or scope by `(WindowId, id)` if multi-window support is needed later.
#[derive(Default)]
pub struct ElementRegistry {
    entries: FxHashMap<SharedString, ElementSnapshot>,
    current_frame_seq: u64,
}

impl Global for ElementRegistry {}

impl ElementRegistry {
    /// Advance the frame counter. Call once before a render pass you intend to
    /// observe; any entry not re-stamped during that pass becomes stale.
    pub fn begin_frame(&mut self) {
        self.current_frame_seq = self.current_frame_seq.wrapping_add(1);
    }

    /// Record or refresh a tracked element's bounds for the current frame.
    /// Called from [`crate::track`]'s canvas paint closure.
    pub fn upsert(&mut self, id: SharedString, bounds: Bounds<Pixels>, enabled: bool) {
        let frame_seq = self.current_frame_seq;
        self.entries.insert(
            id.clone(),
            ElementSnapshot {
                id,
                bounds,
                enabled,
                frame_seq,
            },
        );
    }

    /// Current snapshot for `id`, or `None` if it was not painted in the
    /// current frame (stale — e.g. unmounted or conditionally hidden).
    pub fn get(&self, id: &str) -> Option<ElementSnapshot> {
        self.entries
            .get(id)
            .filter(|s| s.frame_seq == self.current_frame_seq)
            .cloned()
    }

    /// Iterator over every entry painted in the current frame.
    pub fn all_visible(&self) -> impl Iterator<Item = &ElementSnapshot> {
        let seq = self.current_frame_seq;
        self.entries.values().filter(move |s| s.frame_seq == seq)
    }

    /// A flat, single-level [`ElementTree`] of the current frame's entries.
    /// Consumed by the snapshot layer (Phase 04). Ordering is unspecified here
    /// (callers that need determinism sort by `id`).
    pub fn snapshot_tree(&self) -> ElementTree {
        let roots = self
            .all_visible()
            .map(|s| ElementNode {
                id: s.id.clone(),
                bounds: s.bounds,
                enabled: s.enabled,
                children: Vec::new(),
            })
            .collect();
        ElementTree { roots }
    }
}

// `SharedString: Borrow<str>` is what makes the `&str` key lookup in `get` work.
const _: fn() = || {
    fn assert_borrow<T: Borrow<str>>() {}
    assert_borrow::<SharedString>();
};
