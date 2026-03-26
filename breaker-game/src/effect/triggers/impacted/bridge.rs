//! Bridge systems for impacted triggers — evaluates BOTH entities (bolt + hit target)
//! for `Trigger::Impacted(Cell/Wall/Breaker)`.

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
    effect::{
        definition::{EffectChains, EffectTarget, ImpactTarget, Trigger},
        helpers::evaluate_entity_chains,
    },
};

/// Bridge for `BoltHitCell` — evaluates BOTH bolt and cell entities for
/// `Trigger::Impacted(Cell)`.
pub(crate) fn bridge_cell_impacted(
    mut reader: MessageReader<BoltHitCell>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impacted(ImpactTarget::Cell);
    for hit in reader.read() {
        let targets = vec![EffectTarget::Entity(hit.bolt)];

        if let Ok(mut chains) = chains_query.get_mut(hit.bolt) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        if let Ok(mut chains) = chains_query.get_mut(hit.cell) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }
    }
}

/// Bridge for `BoltHitWall` — evaluates BOTH bolt and wall entities for
/// `Trigger::Impacted(Wall)`.
pub(crate) fn bridge_wall_impacted(
    mut reader: MessageReader<BoltHitWall>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impacted(ImpactTarget::Wall);
    for hit in reader.read() {
        let targets = vec![EffectTarget::Entity(hit.bolt)];

        if let Ok(mut chains) = chains_query.get_mut(hit.bolt) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }

        if let Ok(mut chains) = chains_query.get_mut(hit.wall) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }
    }
}

/// Bridge for `BoltHitBreaker` — evaluates BOTH bolt and breaker entities for
/// `Trigger::Impacted(Breaker)`.
///
/// Note: `BoltHitBreaker` only carries `bolt: Entity`. The breaker entity is not
/// included in the message, so only the bolt's chains are evaluated.
pub(crate) fn bridge_breaker_impacted(
    mut reader: MessageReader<BoltHitBreaker>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let trigger_kind = Trigger::Impacted(ImpactTarget::Breaker);
    for hit in reader.read() {
        let targets = vec![EffectTarget::Entity(hit.bolt)];

        if let Ok(mut chains) = chains_query.get_mut(hit.bolt) {
            evaluate_entity_chains(&mut chains, trigger_kind, targets.clone(), &mut commands);
        }
    }
}

/// Registers bridge systems for impacted triggers.
pub(crate) fn register(app: &mut App) {
    use crate::{bolt::BoltSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        (
            bridge_cell_impacted
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
            bridge_wall_impacted
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
            bridge_breaker_impacted
                .after(BoltSystems::BreakerCollision)
                .in_set(EffectSystems::Bridge),
        )
            .run_if(in_state(PlayingState::Active)),
    );
}
