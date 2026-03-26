//! Bridge systems for impact triggers — sweeps ALL entities with `EffectChains`
//! for `Trigger::Impact(Cell/Wall/Breaker)`, evaluates `ArmedEffects` on bolt,
//! and evaluates `Until` children.

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, EffectTarget, ImpactTarget, Trigger},
        effect_nodes::until::{UntilTimers, UntilTriggers},
        helpers::*,
    },
};

/// Bridge for `BoltHitCell` — sweeps ALL entities with `EffectChains` for
/// `Trigger::Impact(Cell)`, evaluates `ArmedEffects` on bolt, and evaluates
/// `Until` children.
pub(crate) fn bridge_cell_impact(
    mut reader: MessageReader<BoltHitCell>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impact(ImpactTarget::Cell);
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        for mut chains in &mut chains_query {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        evaluate_armed(&mut armed_query, &mut commands, bolt_entity, trigger_kind);

        evaluate_until_children(
            &until_query,
            bolt_entity,
            trigger_kind,
            &targets,
            &mut commands,
        );
    }
}

/// Bridge for `BoltHitWall` — sweeps ALL entities with `EffectChains` for
/// `Trigger::Impact(Wall)`, evaluates `ArmedEffects` on bolt, and evaluates
/// `Until` children.
pub(crate) fn bridge_wall_impact(
    mut reader: MessageReader<BoltHitWall>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impact(ImpactTarget::Wall);
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        for mut chains in &mut chains_query {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        evaluate_armed(&mut armed_query, &mut commands, bolt_entity, trigger_kind);

        evaluate_until_children(
            &until_query,
            bolt_entity,
            trigger_kind,
            &targets,
            &mut commands,
        );
    }
}

/// Bridge for `BoltHitBreaker` — sweeps ALL entities with `EffectChains` for
/// `Trigger::Impact(Breaker)`, evaluates `ArmedEffects` on bolt, and evaluates
/// `Until` children.
pub(crate) fn bridge_breaker_impact(
    mut reader: MessageReader<BoltHitBreaker>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impact(ImpactTarget::Breaker);
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        for mut chains in &mut chains_query {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        evaluate_armed(&mut armed_query, &mut commands, bolt_entity, trigger_kind);
    }
}

/// Registers bridge systems for impact triggers.
pub(crate) fn register(app: &mut App) {
    use crate::{bolt::BoltSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        (
            bridge_cell_impact
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
            bridge_wall_impact
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
            bridge_breaker_impact
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
        )
            .run_if(in_state(PlayingState::Active)),
    );
}
