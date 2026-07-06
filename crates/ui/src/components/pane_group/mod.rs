//! Recursive split-tree layout: [`PaneGroup`] owns a tree of [`Pane`]s
//! joined by [`PaneAxis`] splits, tracks which pane is active, and exposes
//! [`SplitDirection`]-driven split/close/focus operations plus the default
//! [`crate::register_pane_keybindings`] that drive them.
//!
//! Deliberately reuses [`gpui::Axis`] (exactly `Horizontal`/`Vertical`)
//! rather than a redundant parallel enum, and keeps the tree shape
//! (`Member`/`PaneAxis`, per-axis `flexes`) close to the proven shape used
//! by editors with recursive pane splitting — reused as an *idea*, not code
//! (no shared dependency, no copied source).

mod divider;
mod lifecycle;
mod preview;
mod render;
mod tree;

use std::cell::Cell;
use std::rc::Rc;

use gpui::{Axis, Bounds, Entity, FocusHandle, Focusable, Subscription};

pub use preview::PaneGroupPreview;

use crate::{Pane, prelude::*};

/// Direction a new pane is inserted relative to the currently active pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Left,
    Right,
    Up,
    Down,
}

impl SplitDirection {
    /// The layout axis a split in this direction runs along.
    pub fn axis(self) -> Axis {
        match self {
            SplitDirection::Left | SplitDirection::Right => Axis::Horizontal,
            SplitDirection::Up | SplitDirection::Down => Axis::Vertical,
        }
    }

    /// Whether the new pane is inserted before (`Left`/`Up`) or after
    /// (`Right`/`Down`) the active pane along [`Self::axis`].
    fn inserts_before(self) -> bool {
        matches!(self, SplitDirection::Left | SplitDirection::Up)
    }
}

/// A node in [`PaneGroup`]'s recursive split tree: either a leaf pane or a
/// nested split along some [`Axis`].
pub enum Member {
    Leaf(Entity<Pane>),
    Split(PaneAxis),
}

/// A row (`Horizontal`) or column (`Vertical`) of `N` [`Member`]s, each
/// sized by the parallel `flexes` fraction (sums to `1.0`).
pub struct PaneAxis {
    pub axis: Axis,
    pub members: Vec<Member>,
    pub flexes: Vec<f32>,
    /// Last-measured container bounds, populated by a `canvas()` during
    /// render and read back by the divider-drag handler to convert pointer
    /// deltas into fraction deltas. Persists on the node across re-renders.
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl PaneAxis {
    fn new(axis: Axis, members: Vec<Member>, flexes: Vec<f32>) -> Self {
        Self {
            axis,
            members,
            flexes,
            bounds: Rc::new(Cell::new(None)),
        }
    }
}

/// Returned by [`PaneGroup::close_active`] when the active pane is the only
/// leaf left in the tree — a `PaneGroup` always keeps at least one pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CannotRemoveLastPane;

/// Stateful owner of a recursive pane split tree: which pane is active, how
/// new panes' content is created, and (via its [`Render`] impl) the
/// split/close/focus keybinding wiring and active-pane highlight.
///
/// Create with `cx.new(|cx| PaneGroup::new(cx, pane))`, optionally
/// `.with_pane_factory(..)` to control what split-created panes contain.
pub struct PaneGroup {
    root: Member,
    active_pane: Entity<Pane>,
    pane_factory: Box<dyn Fn(&mut Context<Pane>) -> Pane>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl PaneGroup {
    /// Boots a single-leaf tree containing `pane`.
    ///
    /// Takes `cx: &mut Context<Self>` (a refinement over phase-01's
    /// `new(pane: Entity<Pane>)` pseudocode) — needed to allocate this
    /// group's own [`FocusHandle`] and subscribe to `pane`'s
    /// [`PaneEvent::Empty`].
    pub fn new(cx: &mut Context<Self>, pane: Entity<Pane>) -> Self {
        let subscription = Self::watch_pane(&pane, cx);
        Self {
            root: Member::Leaf(pane.clone()),
            active_pane: pane,
            pane_factory: Box::new(|_| Pane::new()),
            focus_handle: cx.focus_handle(),
            _subscriptions: vec![subscription],
        }
    }

    /// Sets the factory used to construct the [`Pane`] for every future
    /// split. Defaults to an empty [`Pane::new`].
    pub fn with_pane_factory(
        mut self,
        factory: impl Fn(&mut Context<Pane>) -> Pane + 'static,
    ) -> Self {
        self.pane_factory = Box::new(factory);
        self
    }

    /// The currently active (keyboard/action target) pane.
    pub fn active_pane(&self) -> &Entity<Pane> {
        &self.active_pane
    }

    /// Sets the active pane directly, for callers holding a specific
    /// `Entity<Pane>` handle (e.g. programmatically laying out an initial
    /// grid, or a "reveal this file's pane" flow) rather than a keyboard
    /// direction. No tree-membership validation: setting a pane outside
    /// this group's tree only makes subsequent `split`/`close_active`
    /// target a detached pane, which is inert (not a crash risk) but almost
    /// certainly not what the caller wants.
    pub fn set_active_pane(&mut self, pane: Entity<Pane>, cx: &mut Context<Self>) {
        self.active_pane = pane;
        cx.notify();
    }

    /// Splits the active pane in `dir`: appends a sibling into the active
    /// pane's parent axis if it already runs along `dir`'s axis (N-way
    /// split), otherwise wraps the active pane in a new two-child axis. The
    /// new pane becomes active.
    pub fn split(&mut self, dir: SplitDirection, cx: &mut Context<Self>) {
        let new_pane = cx.new(|pane_cx| (self.pane_factory)(pane_cx));
        let subscription = Self::watch_pane(&new_pane, cx);
        self._subscriptions.push(subscription);

        let active_id = self.active_pane.entity_id();
        let inserted = self.root.split_active(
            active_id,
            dir.axis(),
            dir.inserts_before(),
            new_pane.clone(),
        );
        debug_assert!(
            inserted,
            "PaneGroup's active pane must exist in its own tree"
        );

        self.active_pane = new_pane;
        cx.notify();
    }

    /// Removes the active pane from the tree. Errs if it is the last pane.
    pub fn close_active(&mut self, cx: &mut Context<Self>) -> Result<(), CannotRemoveLastPane> {
        let target = self.active_pane.clone();
        self.remove_pane(&target, cx)
    }

    /// Moves the active pane to its neighbor in `dir`, if one exists in the
    /// same axis as the active pane's immediate parent split.
    pub fn focus(&mut self, dir: SplitDirection, cx: &mut Context<Self>) {
        let active_id = self.active_pane.entity_id();
        let neighbor = match &self.root {
            Member::Leaf(_) => None,
            Member::Split(axis) => axis.find_neighbor(active_id, dir),
        };
        if let Some(neighbor) = neighbor {
            self.active_pane = neighbor;
            cx.notify();
        }
    }
}

impl Focusable for PaneGroup {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
