//! Stateful, tabbed pane rendered as a leaf of [`crate::PaneGroup`]'s
//! recursive split tree.
//!
//! Deliberately generic over tab content via [`TabContent`] rather than
//! hardcoding e.g. a text editor or terminal view — `boltz-ui` is a
//! reusable component library, so `Pane` only owns tab bookkeeping
//! (add/close/activate/reorder) and delegates the actual per-tab content to
//! whatever the consumer supplies.

mod render;

use std::rc::Rc;

use gpui::{AnyElement, EventEmitter};

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
        }
    }

    /// Sets whether this pane is its group's active pane (drives whether its
    /// active tab is shown as selected). Called by [`crate::PaneGroup`].
    pub fn set_focused(&mut self, focused: bool, cx: &mut Context<Self>) {
        if self.focused != focused {
            self.focused = focused;
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
        let id = TabId(self.next_tab_id);
        self.next_tab_id += 1;
        self.tabs.push((id, content));
        self.active_idx = self.tabs.len() - 1;
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
        self.tabs.remove(idx);
        if self.active_idx >= self.tabs.len() {
            self.active_idx = self.tabs.len().saturating_sub(1);
        } else if idx < self.active_idx {
            self.active_idx -= 1;
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
            self.active_idx = idx;
            cx.notify();
        }
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
