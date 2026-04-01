//! Bridge systems for targeted `Impacted` triggers -- one per collision type.
//!
//! Each system reads a collision message and fires `Impacted(X)` on entity A
//! and `Impacted(Y)` on entity B, evaluating only those specific entities.
use bevy::prelude::*;

use crate::{
    bolt::{
        messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
        sets::BoltSystems,
    },
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    cells::messages::CellImpactWall,
    effect::{
        core::*,
        sets::EffectSystems,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::playing_state::PlayingState,
};

/// `BoltImpactCell` -> `Impacted(Cell)` on bolt + `Impacted(Bolt)` on cell.
pub(super) fn bridge_impacted_bolt_cell(
    mut reader: MessageReader<BoltImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
        }
    }
}

/// `BoltImpactWall` -> `Impacted(Wall)` on bolt + `Impacted(Bolt)` on wall.
pub(super) fn bridge_impacted_bolt_wall(
    mut reader: MessageReader<BoltImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.wall) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
        }
    }
}

/// `BoltImpactBreaker` -> `Impacted(Breaker)` on bolt + `Impacted(Bolt)` on breaker.
pub(super) fn bridge_impacted_bolt_breaker(
    mut reader: MessageReader<BoltImpactBreaker>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.bolt) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.breaker) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
        }
    }
}

/// `BreakerImpactCell` -> `Impacted(Cell)` on breaker + `Impacted(Breaker)` on cell.
pub(super) fn bridge_impacted_breaker_cell(
    mut reader: MessageReader<BreakerImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.breaker) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
        }
    }
}

/// `BreakerImpactWall` -> `Impacted(Wall)` on breaker + `Impacted(Breaker)` on wall.
pub(super) fn bridge_impacted_breaker_wall(
    mut reader: MessageReader<BreakerImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.breaker) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.wall) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
        }
    }
}

/// `CellImpactWall` -> `Impacted(Wall)` on cell + `Impacted(Cell)` on wall.
pub(super) fn bridge_impacted_cell_wall(
    mut reader: MessageReader<CellImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.cell) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
        }
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.wall) {
            evaluate_bound_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
            evaluate_staged_effects(
                &Trigger::Impacted(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
        }
    }
}

/// Register all targeted impacted bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridge_impacted_bolt_cell.after(BoltSystems::CellCollision),
            bridge_impacted_bolt_wall.after(BoltSystems::CellCollision),
            bridge_impacted_bolt_breaker.after(BoltSystems::BreakerCollision),
            bridge_impacted_breaker_cell,
            bridge_impacted_breaker_wall,
            bridge_impacted_cell_wall,
        )
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}
