//! During node evaluator — state-scoped effect application.

use bevy::prelude::*;

use crate::effect_v3::{
    storage::BoundEffects,
    types::{Condition, ScopedTree, Tree, TriggerContext},
};

/// Evaluate a `Tree::During` node: apply inner effects while the condition
/// is true, reverse them when it becomes false.
///
/// When called from inside a When gate (Shape A), this installs the
/// inner During as a top-level `Tree::During` entry in `BoundEffects`
/// so the condition poller can pick it up.
pub fn evaluate_during(
    entity: Entity,
    condition: &Condition,
    inner: &ScopedTree,
    _context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    // Already-installed Durings are managed by the condition poller, not
    // walk_effects. Skip re-installation when the source key indicates
    // this is an installed entry being re-walked.
    if source.contains("#installed") {
        return;
    }
    commands.queue(DuringInstallCommand {
        entity,
        condition: condition.clone(),
        inner: inner.clone(),
        source: source.to_owned(),
    });
}

/// Deferred command that installs a During tree into `BoundEffects` for the
/// condition poller to manage. Idempotent — skips if already installed.
struct DuringInstallCommand {
    entity:    Entity,
    condition: Condition,
    inner:     ScopedTree,
    source:    String,
}

impl Command for DuringInstallCommand {
    fn apply(self, world: &mut World) {
        if world.get_entity(self.entity).is_err() {
            return;
        }

        let install_key = format!("{}#installed[0]", self.source);
        let tree = Tree::During(self.condition, Box::new(self.inner));

        // Idempotent: skip if already installed OR if this During is
        // already a top-level entry (re-walked by walk_effects, not a
        // Shape A installation from inside a When gate).
        if let Some(bound) = world.get::<BoundEffects>(self.entity) {
            if bound.0.iter().any(|(name, _)| name == &install_key) {
                return;
            }
            if bound
                .0
                .iter()
                .any(|(name, tree)| name == &self.source && matches!(tree, Tree::During(..)))
            {
                return;
            }
        }

        // Install the During into BoundEffects for condition poller to manage
        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
            bound.0.push((install_key, tree));
        } else {
            // No BoundEffects component — create one with the installed entry
            world
                .entity_mut(self.entity)
                .insert(BoundEffects(vec![(install_key, tree)]));
        }
    }
}
