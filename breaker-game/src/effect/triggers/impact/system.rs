//! Bridge systems for global `Impact` triggers -- one per collision type.
//!
//! Each system reads a collision message and fires `Impact(X)` + `Impact(Y)` globally,
//! sweeping all entities with `BoundEffects`.
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

/// `BoltImpactCell` -> `Impact(Cell)` global + `Impact(Bolt)` global.
pub(super) fn bridge_impact_bolt_cell(
    mut reader: MessageReader<BoltImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
        }
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
        }
    }
}

/// `BoltImpactWall` -> `Impact(Wall)` global + `Impact(Bolt)` global.
pub(super) fn bridge_impact_bolt_wall(
    mut reader: MessageReader<BoltImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
        }
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
        }
    }
}

/// `BoltImpactBreaker` -> `Impact(Breaker)` global + `Impact(Bolt)` global.
pub(super) fn bridge_impact_bolt_breaker(
    mut reader: MessageReader<BoltImpactBreaker>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
        }
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Bolt),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Bolt),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.bolt),
            );
        }
    }
}

/// `BreakerImpactCell` -> `Impact(Cell)` global + `Impact(Breaker)` global.
pub(super) fn bridge_impact_breaker_cell(
    mut reader: MessageReader<BreakerImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
        }
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
        }
    }
}

/// `BreakerImpactWall` -> `Impact(Wall)` global + `Impact(Breaker)` global.
pub(super) fn bridge_impact_breaker_wall(
    mut reader: MessageReader<BreakerImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
        }
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Breaker),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Breaker),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.breaker),
            );
        }
    }
}

/// `CellImpactWall` -> `Impact(Wall)` global + `Impact(Cell)` global.
pub(super) fn bridge_impact_cell_wall(
    mut reader: MessageReader<CellImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Wall),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Wall),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.wall),
            );
        }
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(
                &Trigger::Impact(ImpactTarget::Cell),
                entity,
                bound,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
            evaluate_staged_effects(
                &Trigger::Impact(ImpactTarget::Cell),
                entity,
                &mut staged,
                &mut commands,
                Some(msg.cell),
            );
        }
    }
}

/// Register all global impact bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridge_impact_bolt_cell.after(BoltSystems::CellCollision),
            bridge_impact_bolt_wall.after(BoltSystems::CellCollision),
            bridge_impact_bolt_breaker.after(BoltSystems::BreakerCollision),
            bridge_impact_breaker_cell,
            bridge_impact_breaker_wall,
            bridge_impact_cell_wall,
        )
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}
