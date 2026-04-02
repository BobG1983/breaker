//! Shared helpers for end-to-end desugaring tests.

use bevy::prelude::*;

use crate::effect::{BoundEffects, StagedEffects, Trigger, TriggerContext};

/// System that evaluates `NodeStart` trigger on all entities with `BoundEffects`.
/// Mirrors `bridge_node_start` from `effect::triggers::node_start` (which is
/// module-private), using the public(crate) evaluate helpers.
pub(super) fn sys_evaluate_node_start(
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    use crate::effect::triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects};

    for (entity, bound, mut staged) in &mut query {
        evaluate_bound_effects(
            &Trigger::NodeStart,
            entity,
            bound,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
        evaluate_staged_effects(
            &Trigger::NodeStart,
            entity,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}
