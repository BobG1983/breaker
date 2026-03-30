//! Deferred pending effect application for bolt, breaker, cell, and wall entities.

use bevy::prelude::*;
use breaker::effect::{BoundEffects, StagedEffects};

use super::types::{
    PendingBoltEffects, PendingBreakerEffects, PendingCellEffects, PendingWallEffects,
};
use crate::invariants::{ScenarioTagBolt, ScenarioTagBreaker, ScenarioTagCell, ScenarioTagWall};

/// Applies deferred breaker effects from [`PendingBreakerEffects`] to tagged breaker entities.
///
/// Runs in `FixedUpdate` after [`super::entity_tagging::tag_game_entities`]. Uses a `Local<bool>` guard
/// so it fires at most once. Waits until at least one `ScenarioTagBreaker` entity
/// exists before applying. Inserts `BoundEffects` and `StagedEffects` on entities
/// that lack them.
pub fn apply_pending_breaker_effects(
    mut done: Local<bool>,
    mut pending: Option<ResMut<PendingBreakerEffects>>,
    breaker_query: Query<Entity, With<ScenarioTagBreaker>>,
    mut commands: Commands,
) {
    if *done {
        return;
    }
    let Some(ref mut pending) = pending else {
        return;
    };
    if breaker_query.is_empty() {
        return;
    }
    let entries = pending.0.clone();
    for entity in &breaker_query {
        commands
            .entity(entity)
            .insert_if_new((BoundEffects::default(), StagedEffects::default()));
        let entries_clone = entries.clone();
        commands.queue(move |world: &mut World| {
            if let Some(mut bound) = world.entity_mut(entity).get_mut::<BoundEffects>() {
                bound.0.extend(entries_clone);
            }
        });
    }
    pending.0.clear();
    *done = true;
}

/// Applies deferred bolt effects from [`PendingBoltEffects`] to tagged bolt entities.
///
/// Runs in `FixedUpdate` after [`super::entity_tagging::tag_game_entities`]. Uses a `Local<bool>` guard
/// so it fires at most once. Waits until at least one `ScenarioTagBolt` entity
/// exists before applying. Inserts `BoundEffects` and `StagedEffects` on entities
/// that lack them (bolts may not be spawned with effect components).
pub fn apply_pending_bolt_effects(
    mut done: Local<bool>,
    mut pending: Option<ResMut<PendingBoltEffects>>,
    bolt_query: Query<Entity, With<ScenarioTagBolt>>,
    mut commands: Commands,
) {
    if *done {
        return;
    }
    let Some(ref mut pending) = pending else {
        return;
    };
    if bolt_query.is_empty() {
        return;
    }
    let entries = pending.0.clone();
    for entity in &bolt_query {
        commands
            .entity(entity)
            .insert_if_new((BoundEffects::default(), StagedEffects::default()));
        let entries_clone = entries.clone();
        commands.queue(move |world: &mut World| {
            if let Some(mut bound) = world.entity_mut(entity).get_mut::<BoundEffects>() {
                bound.0.extend(entries_clone);
            }
        });
    }
    pending.0.clear();
    *done = true;
}

/// Applies deferred cell effects from [`PendingCellEffects`] to tagged cell entities.
///
/// Runs in `FixedUpdate` after [`super::entity_tagging::tag_game_entities`]. Uses a `Local<bool>` guard
/// so it fires at most once. Waits until at least one `ScenarioTagCell` entity
/// exists before applying. Inserts `BoundEffects` and `StagedEffects` on entities
/// that lack them (cells are not spawned with effect components).
pub fn apply_pending_cell_effects(
    mut done: Local<bool>,
    mut pending: Option<ResMut<PendingCellEffects>>,
    cell_query: Query<Entity, With<ScenarioTagCell>>,
    mut commands: Commands,
) {
    if *done {
        return;
    }
    let Some(ref mut pending) = pending else {
        return;
    };
    if cell_query.is_empty() {
        return;
    }
    let entries = pending.0.clone();
    for entity in &cell_query {
        commands
            .entity(entity)
            .insert_if_new((BoundEffects::default(), StagedEffects::default()));
        let entries_clone = entries.clone();
        commands.queue(move |world: &mut World| {
            if let Some(mut bound) = world.entity_mut(entity).get_mut::<BoundEffects>() {
                bound.0.extend(entries_clone);
            }
        });
    }
    pending.0.clear();
    *done = true;
}

/// Applies deferred wall effects from [`PendingWallEffects`] to tagged wall entities.
///
/// Runs in `FixedUpdate` after [`super::entity_tagging::tag_game_entities`]. Uses a `Local<bool>` guard
/// so it fires at most once. Waits until at least one `ScenarioTagWall` entity
/// exists before applying. Inserts `BoundEffects` and `StagedEffects` on entities
/// that lack them (walls are not spawned with effect components).
pub fn apply_pending_wall_effects(
    mut done: Local<bool>,
    mut pending: Option<ResMut<PendingWallEffects>>,
    wall_query: Query<Entity, With<ScenarioTagWall>>,
    mut commands: Commands,
) {
    if *done {
        return;
    }
    let Some(ref mut pending) = pending else {
        return;
    };
    if wall_query.is_empty() {
        return;
    }
    let entries = pending.0.clone();
    for entity in &wall_query {
        commands
            .entity(entity)
            .insert_if_new((BoundEffects::default(), StagedEffects::default()));
        let entries_clone = entries.clone();
        commands.queue(move |world: &mut World| {
            if let Some(mut bound) = world.entity_mut(entity).get_mut::<BoundEffects>() {
                bound.0.extend(entries_clone);
            }
        });
    }
    pending.0.clear();
    *done = true;
}
