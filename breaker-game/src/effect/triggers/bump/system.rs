//! Bridge system for the `bump` trigger.
use bevy::prelude::*;

use crate::{
    breaker::sets::BreakerSystems,
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    prelude::*,
};

pub(super) fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext {
            bolt: msg.bolt,
            breaker: Some(msg.breaker),
            ..default()
        };
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Bump,
                entity,
                bound,
                &mut staged,
                &mut commands,
                context,
            );
            evaluate_staged_effects(&Trigger::Bump, entity, &mut staged, &mut commands, context);
        }
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridge_bump
            .in_set(EffectSystems::Bridge)
            .after(BreakerSystems::GradeBump)
            .run_if(in_state(NodeState::Playing)),
    );
}
