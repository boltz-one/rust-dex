//! Pure tree operations on [`Member`]/[`PaneAxis`]: split-insertion,
//! removal-with-collapse, and directional neighbor lookup. Kept separate
//! from `mod.rs`'s [`super::PaneGroup`] state/API so each file stays under
//! the crate's 200-line guideline.

use gpui::{Axis, Entity, EntityId};

use super::{Member, PaneAxis, SplitDirection};
use crate::Pane;

/// Outcome of removing an entity from a [`PaneAxis`].
pub(super) enum RemoveOutcome {
    /// `id` was not found in this subtree.
    NotFound,
    /// `id` was removed and this axis still has 2+ members.
    Removed,
    /// `id` was removed and this axis shrank to a single member, which the
    /// caller must splice in to replace the axis itself.
    Collapse(Member),
}

impl PaneAxis {
    pub(super) fn remove(&mut self, id: EntityId) -> RemoveOutcome {
        if let Some(ix) = self
            .members
            .iter()
            .position(|m| matches!(m, Member::Leaf(pane) if pane.entity_id() == id))
        {
            self.members.remove(ix);
            self.flexes.remove(ix);
            rebalance(&mut self.flexes);
            return if self.members.len() == 1 {
                RemoveOutcome::Collapse(self.members.remove(0))
            } else {
                RemoveOutcome::Removed
            };
        }
        for member in self.members.iter_mut() {
            if let Member::Split(axis) = member {
                match axis.remove(id) {
                    RemoveOutcome::NotFound => continue,
                    RemoveOutcome::Removed => return RemoveOutcome::Removed,
                    RemoveOutcome::Collapse(replacement) => {
                        *member = replacement;
                        return RemoveOutcome::Removed;
                    }
                }
            }
        }
        RemoveOutcome::NotFound
    }

    pub(super) fn find_neighbor(
        &self,
        active_id: EntityId,
        dir: SplitDirection,
    ) -> Option<Entity<Pane>> {
        if self.axis == dir.axis()
            && let Some(ix) = self
                .members
                .iter()
                .position(|m| matches!(m, Member::Leaf(pane) if pane.entity_id() == active_id))
        {
            let neighbor_ix = if dir.inserts_before() {
                ix.checked_sub(1)
            } else {
                (ix + 1 < self.members.len()).then_some(ix + 1)
            };
            if let Some(neighbor_ix) = neighbor_ix {
                return Some(first_leaf(&self.members[neighbor_ix]));
            }
        }
        self.members.iter().find_map(|member| match member {
            Member::Split(axis) => axis.find_neighbor(active_id, dir),
            Member::Leaf(_) => None,
        })
    }
}

impl Member {
    /// Inserts `new_pane` next to the leaf matching `active_id`: appended as
    /// a sibling if the immediate parent axis already matches `axis`
    /// (N-way split), otherwise the leaf is wrapped in a fresh two-child
    /// [`PaneAxis`]. Returns `true` once handled.
    pub(super) fn split_active(
        &mut self,
        active_id: EntityId,
        axis: Axis,
        before: bool,
        new_pane: Entity<Pane>,
    ) -> bool {
        match self {
            Member::Leaf(pane) => {
                if pane.entity_id() != active_id {
                    return false;
                }
                let old = pane.clone();
                let members = if before {
                    vec![Member::Leaf(new_pane), Member::Leaf(old)]
                } else {
                    vec![Member::Leaf(old), Member::Leaf(new_pane)]
                };
                *self = Member::Split(PaneAxis::new(axis, members, vec![0.5, 0.5]));
                true
            }
            Member::Split(pane_axis) => {
                if pane_axis.axis == axis
                    && let Some(ix) = pane_axis.members.iter().position(
                        |m| matches!(m, Member::Leaf(pane) if pane.entity_id() == active_id),
                    )
                {
                    let insert_ix = if before { ix } else { ix + 1 };
                    pane_axis.members.insert(insert_ix, Member::Leaf(new_pane));
                    let n = pane_axis.members.len();
                    pane_axis.flexes = vec![1.0 / n as f32; n];
                    return true;
                }
                pane_axis
                    .members
                    .iter_mut()
                    .any(|member| member.split_active(active_id, axis, before, new_pane.clone()))
            }
        }
    }
}

impl Member {
    /// Collects every leaf pane in this subtree (used to sync per-pane focus).
    pub(super) fn collect_leaves(&self, out: &mut Vec<Entity<Pane>>) {
        match self {
            Member::Leaf(pane) => out.push(pane.clone()),
            Member::Split(axis) => {
                for member in &axis.members {
                    member.collect_leaves(out);
                }
            }
        }
    }
}

fn rebalance(flexes: &mut [f32]) {
    if flexes.is_empty() {
        return;
    }
    let even = 1.0 / flexes.len() as f32;
    flexes.fill(even);
}

/// Descends into the first child of nested splits to find a leaf pane.
pub(super) fn first_leaf(member: &Member) -> Entity<Pane> {
    match member {
        Member::Leaf(pane) => pane.clone(),
        Member::Split(axis) => first_leaf(&axis.members[0]),
    }
}

/// Navigates from `root` through `path` (child indices at each level) to
/// the [`PaneAxis`] living there — used by divider-drag handlers to find
/// which axis's `flexes` to mutate without storing raw pointers.
pub(super) fn axis_at_mut<'a>(root: &'a mut Member, path: &[usize]) -> Option<&'a mut PaneAxis> {
    let mut current = root;
    for &ix in path {
        current = match current {
            Member::Split(axis) => axis.members.get_mut(ix)?,
            Member::Leaf(_) => return None,
        };
    }
    match current {
        Member::Split(axis) => Some(axis),
        Member::Leaf(_) => None,
    }
}
