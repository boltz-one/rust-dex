//! Stateful, tabbed pane rendered as a leaf of [`crate::PaneGroup`]'s
//! recursive split tree.
//!
//! Deliberately generic over tab content via [`TabContent`] rather than
//! hardcoding e.g. a text editor or terminal view — `boltz-ui` is a
//! reusable component library, so `Pane` only owns tab bookkeeping
//! (add/close/activate/reorder) and delegates the actual per-tab content to
//! whatever the consumer supplies.

mod render;

use std::cell::Cell;
use std::rc::Rc;

use gpui::{AnyElement, Bounds, EventEmitter, Pixels};

use crate::prelude::*;

/// Content rendered inside a single tab of a [`Pane`].
///
/// Object-safe by design (no generics, no `Self` return type) so `Pane` can
/// hold a heterogeneous `Vec<Box<dyn TabContent>>`.
///
/// # Example
/// ```ignore
/// struct FileTab { path: SharedString }
/// impl TabContent for FileTab {
///     fn render(&self, _focused: bool, _window: &mut Window, _cx: &mut App) -> AnyElement {
///         div().child(self.path.clone()).into_any_element()
///     }
///     fn title(&self) -> SharedString { self.path.clone() }
/// }
/// ```
pub trait TabContent: 'static {
    /// Renders this tab's body. `focused` is true when this tab is the
    /// active tab of a [`Pane`] that is itself `PaneGroup`'s active pane.
    fn render(&self, focused: bool, window: &mut Window, cx: &mut App) -> AnyElement;
    /// The label shown on the tab strip.
    fn title(&self) -> SharedString;

    /// Fired when this tab becomes the active tab of a focused [`Pane`]:
    /// activation ([`Pane::activate`]), being added ([`Pane::add_tab`]),
    /// inheriting focus after the active tab is closed, or its pane regaining
    /// focus ([`Pane::set_focused`]). A terminal implementor would e.g. resume
    /// cursor blink / mark the PTY focused. Default: no-op, so existing
    /// implementors need no changes.
    ///
    /// NOT fired for a pane's initial tab seeded via [`Pane::with_tab`] (that
    /// builder runs before a [`Context`] exists, so no hook can fire) — an
    /// implementor needing initial-focus state should set it at construction
    /// or have the mounting code trigger focus explicitly after mount.
    ///
    /// Takes `&mut App` (not `&mut Window`) because most fire sites
    /// (`activate`/`add_tab`/`set_focused`) run from a `Context`-only path
    /// with no `Window` in scope; focus-driven tab behaviour (blink toggle,
    /// PTY focus flag) needs no window geometry.
    fn on_focus_in(&mut self, _cx: &mut App) {}

    /// Fired when this tab stops being the active tab of a focused [`Pane`]
    /// (another tab activated, or its pane losing focus). A terminal
    /// implementor would e.g. pause cursor blink. Default: no-op.
    fn on_focus_out(&mut self, _cx: &mut App) {}

    /// Fired when the pane's tab-content area changes size (measured via a
    /// `canvas()` in [`Pane`]'s render, one frame after the layout change).
    /// A terminal implementor would recompute rows/cols and `resize` its PTY
    /// (SIGWINCH). Default: no-op.
    fn on_resize(&mut self, _bounds: Bounds<Pixels>, _cx: &mut App) {}

    /// Fired exactly once, right before this tab is removed — whether via
    /// closing the single tab or via its whole pane being removed from the
    /// tree ([`Pane::close_all_tabs`]). A terminal implementor MUST shut its
    /// PTY down here: `Drop` timing is not guaranteed to coincide with tree
    /// removal (other live `Entity` handles can outlive it), so relying on
    /// `Drop` would leak the child process. Default: no-op.
    fn on_close(&mut self, _cx: &mut App) {}
}

/// Stable identifier for a tab within one [`Pane`] (unique per-`Pane` only —
/// a `u64` counter is sufficient since reorder/close are always scoped to a
/// single `Pane`, never compared across panes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u64);

/// Events emitted by [`Pane`] for [`crate::PaneGroup`] to react to.
pub enum PaneEvent {
    /// The last tab was just closed; `PaneGroup` should remove this pane
    /// from the split tree.
    Empty,
    /// The pane's header close ("x") button was pressed; `PaneGroup` should
    /// remove this whole pane (no-op if it is the last remaining pane).
    CloseRequested,
}

/// Default content for tabs created via the "+" button when no factory was
/// supplied through [`Pane::with_new_tab_factory`].
struct PlaceholderTab;

impl TabContent for PlaceholderTab {
    fn render(&self, _focused: bool, _window: &mut Window, _cx: &mut App) -> AnyElement {
        div()
            .p_4()
            .child(Label::new("Empty tab").color(Color::Muted))
            .into_any_element()
    }

    fn title(&self) -> SharedString {
        "Untitled".into()
    }
}

/// A single pane holding an ordered list of tabs, exactly one of which is
/// active. Renders its own [`TabBar`]/[`Tab`] strip (with close "x" and add
/// "+" affordances) above the active tab's content.
///
/// Create with `cx.new(|_| Pane::new())`, mount as a [`crate::PaneGroup`]
/// leaf.
pub struct Pane {
    tabs: Vec<(TabId, Box<dyn TabContent>)>,
    active_idx: usize,
    next_tab_id: u64,
    new_tab_factory: Rc<dyn Fn() -> Box<dyn TabContent>>,
    /// Whether this pane is the active pane of its [`crate::PaneGroup`]. Only
    /// the focused pane's active tab is drawn "selected" (accent), so the
    /// window shows exactly one active tab. Kept in sync by `PaneGroup`.
    focused: bool,
    /// Tab-content-area bounds, written by a `canvas()` child during paint and
    /// read back at the START of the next render (same one-frame-lag pattern
    /// `TerminalView`/`ResizablePanelGroup` use) to detect size changes.
    content_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// The `(tab, bounds)` pair last delivered via `on_resize`. Keyed by
    /// [`TabId`] — not just bounds — so a tab that becomes active without a
    /// physical size change (e.g. `add_tab`/`activate`) still gets an initial
    /// `on_resize` with the current bounds (each tab owns an independently
    /// sized PTY). Plain `Cell` (not `Rc`) — only touched inside `Pane`.
    notified_resize: Cell<Option<(TabId, Bounds<Pixels>)>>,
}

impl Pane {
    /// Starts with zero tabs. The "+" button uses [`PlaceholderTab`] until
    /// [`Pane::with_new_tab_factory`] is called.
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_idx: 0,
            next_tab_id: 0,
            new_tab_factory: Rc::new(|| Box::new(PlaceholderTab)),
            focused: true,
            content_bounds: Rc::new(Cell::new(None)),
            notified_resize: Cell::new(None),
        }
    }

    /// Sets whether this pane is its group's active pane (drives whether its
    /// active tab is shown as selected). Called by [`crate::PaneGroup`].
    pub fn set_focused(&mut self, focused: bool, cx: &mut Context<Self>) {
        if self.focused != focused {
            self.focused = focused;
            // Fire the focus lifecycle hook on the active tab so its content
            // (e.g. a terminal) can toggle its focused state.
            if let Some((_, content)) = self.tabs.get_mut(self.active_idx) {
                if focused {
                    content.on_focus_in(cx);
                } else {
                    content.on_focus_out(cx);
                }
            }
            cx.notify();
        }
    }

    /// Sets the factory used to create content for tabs opened via the "+"
    /// button. Additive over the phase-01 design (not explicitly listed
    /// there) — needed because `Pane` renders its own "+" button and must
    /// therefore know how to produce new tab content itself.
    pub fn with_new_tab_factory(
        mut self,
        factory: impl Fn() -> Box<dyn TabContent> + 'static,
    ) -> Self {
        self.new_tab_factory = Rc::new(factory);
        self
    }

    /// Builder that seeds an initial tab without a [`Context`] — usable
    /// inside a [`crate::PaneGroup`] pane factory (which runs before the
    /// pane's own context exists) or any `cx.new(|_| Pane::new().with_tab(..))`
    /// construction. The seeded tab becomes active.
    pub fn with_tab(mut self, content: Box<dyn TabContent>) -> Self {
        let id = TabId(self.next_tab_id);
        self.next_tab_id += 1;
        self.tabs.push((id, content));
        self.active_idx = self.tabs.len() - 1;
        self
    }

    /// Appends a tab and activates it. Returns the new tab's stable id.
    pub fn add_tab(&mut self, content: Box<dyn TabContent>, cx: &mut Context<Self>) -> TabId {
        // Previous active tab (if any) loses focus to the incoming one.
        let prev_active = if self.tabs.is_empty() {
            None
        } else {
            Some(self.active_idx)
        };
        let id = TabId(self.next_tab_id);
        self.next_tab_id += 1;
        self.tabs.push((id, content));
        self.active_idx = self.tabs.len() - 1;
        if self.focused {
            if let Some(prev) = prev_active {
                self.tabs[prev].1.on_focus_out(cx);
            }
            let new_idx = self.active_idx;
            self.tabs[new_idx].1.on_focus_in(cx);
        }
        cx.notify();
        id
    }

    /// Removes the tab at `idx`, reassigning the active index if needed.
    /// Returns `true` if the pane is now empty (in which case
    /// [`PaneEvent::Empty`] is emitted for `PaneGroup` to remove this pane).
    pub fn close_tab(&mut self, idx: usize, cx: &mut Context<Self>) -> bool {
        if idx >= self.tabs.len() {
            return self.tabs.is_empty();
        }
        // Whether the tab being closed is the currently active one — if so, a
        // different tab inherits focus below and must be told.
        let closing_active = idx == self.active_idx;
        // Let the tab release resources (e.g. shut down its PTY) before it is
        // dropped — `Drop` timing alone is not a reliable close signal.
        self.tabs[idx].1.on_close(cx);
        self.tabs.remove(idx);
        if self.active_idx >= self.tabs.len() {
            self.active_idx = self.tabs.len().saturating_sub(1);
        } else if idx < self.active_idx {
            self.active_idx -= 1;
        }
        // Closing the active tab hands focus to whichever tab took its place;
        // that new active tab must receive `on_focus_in` (mirrors `activate`).
        if closing_active && self.focused && !self.tabs.is_empty() {
            let new_active = self.active_idx;
            self.tabs[new_active].1.on_focus_in(cx);
        }
        let now_empty = self.tabs.is_empty();
        if now_empty {
            cx.emit(PaneEvent::Empty);
        }
        cx.notify();
        now_empty
    }

    /// Activates the tab at `idx` (no-op if out of range).
    pub fn activate(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx < self.tabs.len() && idx != self.active_idx {
            let old = self.active_idx;
            self.active_idx = idx;
            if self.focused {
                self.tabs[old].1.on_focus_out(cx);
                self.tabs[idx].1.on_focus_in(cx);
            }
            cx.notify();
        }
    }

    /// Closes every tab (firing each one's [`TabContent::on_close`] in order)
    /// and empties the pane. Called by [`crate::PaneGroup`] when a whole pane
    /// is removed from the tree, so PTY-owning tabs shut down deterministically
    /// instead of relying on `Entity`/`Box` drop timing.
    pub fn close_all_tabs(&mut self, cx: &mut Context<Self>) {
        for (_, content) in self.tabs.iter_mut() {
            content.on_close(cx);
        }
        self.tabs.clear();
        self.active_idx = 0;
        cx.notify();
    }

    /// Moves the tab at `from` to `to`, preserving which tab is active.
    pub fn reorder(&mut self, from: usize, to: usize, cx: &mut Context<Self>) {
        if from == to || from >= self.tabs.len() || to >= self.tabs.len() {
            return;
        }
        let active_id = self.tabs.get(self.active_idx).map(|(id, _)| *id);
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);
        if let Some(active_id) = active_id
            && let Some(ix) = self.tabs.iter().position(|(id, _)| *id == active_id)
        {
            self.active_idx = ix;
        }
        cx.notify();
    }

    /// Number of open tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Index of the active tab.
    pub fn active_index(&self) -> usize {
        self.active_idx
    }

    /// Titles of every open tab, in order — mainly useful for tests
    /// asserting reorder results.
    pub fn titles(&self) -> Vec<SharedString> {
        self.tabs
            .iter()
            .map(|(_, content)| content.title())
            .collect()
    }
}

impl Default for Pane {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter<PaneEvent> for Pane {}
