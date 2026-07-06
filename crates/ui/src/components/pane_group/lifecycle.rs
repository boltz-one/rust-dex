//! Pane removal and the subscription that drives it automatically when a
//! [`Pane`] empties itself out (its last tab closed).

use gpui::{Entity, EntityId, Subscription};

use super::tree::{self, RemoveOutcome};
use super::{CannotRemoveLastPane, Member, PaneGroup};
use crate::{Pane, PaneEvent, prelude::*};

impl PaneGroup {
    pub(super) fn remove_pane(
        &mut self,
        target: &Entity<Pane>,
        cx: &mut Context<Self>,
    ) -> Result<(), CannotRemoveLastPane> {
        let id = target.entity_id();
        match &mut self.root {
            Member::Leaf(pane) if pane.entity_id() == id => Err(CannotRemoveLastPane),
            Member::Leaf(_) => Ok(()),
            Member::Split(axis) => {
                match axis.remove(id) {
                    RemoveOutcome::NotFound => {}
                    RemoveOutcome::Removed => self.reassign_active_if_removed(id, cx),
                    RemoveOutcome::Collapse(replacement) => {
                        self.root = replacement;
                        self.reassign_active_if_removed(id, cx);
                    }
                }
                Ok(())
            }
        }
    }

    fn reassign_active_if_removed(&mut self, removed_id: EntityId, cx: &mut Context<Self>) {
        if self.active_pane.entity_id() == removed_id {
            self.active_pane = tree::first_leaf(&self.root);
        }
        cx.notify();
    }

    /// Subscribes to `pane`'s [`PaneEvent::Empty`] so a pane that closes its
    /// own last tab is automatically removed from the tree.
    pub(super) fn watch_pane(pane: &Entity<Pane>, cx: &mut Context<Self>) -> Subscription {
        cx.subscribe(pane, |this, pane, event: &PaneEvent, cx| match event {
            PaneEvent::Empty => {
                let _ = this.remove_pane(&pane, cx);
            }
        })
    }
}
