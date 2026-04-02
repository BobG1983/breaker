//! Shared test helper systems for on-resolution tests.

use bevy::prelude::*;

use super::super::super::system::*;
use crate::effect::core::*;

pub(super) fn sys_evaluate_bound_for_node_start(
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::NodeStart;
    for (entity, bound, mut staged) in &mut query {
        evaluate_bound_effects(
            &trigger,
            entity,
            bound,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}

pub(super) fn sys_evaluate_staged_for_node_start(
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::NodeStart;
    for (entity, mut staged) in &mut query {
        evaluate_staged_effects(
            &trigger,
            entity,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}

pub(super) fn sys_evaluate_staged_for_bump(
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::Bump;
    for (entity, mut staged) in &mut query {
        evaluate_staged_effects(
            &trigger,
            entity,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}
