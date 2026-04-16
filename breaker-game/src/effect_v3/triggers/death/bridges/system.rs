//! Death trigger bridge systems.
//!
//! Each bridge reads `Destroyed<T>` messages and dispatches `Died`, `Killed`, and
//! `DeathOccurred` triggers to entities with bound effects.

use bevy::prelude::*;

use crate::{
    cells::behaviors::survival::salvo::components::Salvo,
    effect_v3::{
        types::{EntityKind, Trigger, TriggerContext},
        walking::{walk_bound_effects, walk_staged_effects},
    },
    prelude::*,
    shared::death_pipeline::GameEntity,
};

/// Generic death bridge — reads `Destroyed<T>` and dispatches death triggers.
///
/// For each destroyed entity:
/// - `Died` on the victim entity (local)
/// - `Killed(kind)` on the killer entity (local, if killer exists)
/// - `DeathOccurred(kind)` on all entities (global)
fn on_destroyed_inner<T: GameEntity>(
    kind: EntityKind,
    reader: &mut MessageReader<Destroyed<T>>,
    bound_query: &Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: &Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    commands: &mut Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Death {
            victim: msg.victim,
            killer: msg.killer,
        };

        // Local: Died on victim — staged first, then bound.
        if let Ok((bound, staged)) = bound_query.get(msg.victim) {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(
                msg.victim,
                &Trigger::Died,
                &context,
                &staged_trees,
                commands,
            );
            walk_bound_effects(msg.victim, &Trigger::Died, &context, &bound_trees, commands);
        }

        // Local: Killed(kind) and Killed(Any) on killer — staged first
        // for both trigger variants against a single snapshot.
        if let Some(killer) = msg.killer
            && let Ok((bound, staged)) = bound_query.get(killer)
        {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();

            walk_staged_effects(
                killer,
                &Trigger::Killed(kind),
                &context,
                &staged_trees,
                commands,
            );
            walk_staged_effects(
                killer,
                &Trigger::Killed(EntityKind::Any),
                &context,
                &staged_trees,
                commands,
            );

            walk_bound_effects(
                killer,
                &Trigger::Killed(kind),
                &context,
                &bound_trees,
                commands,
            );
            walk_bound_effects(
                killer,
                &Trigger::Killed(EntityKind::Any),
                &context,
                &bound_trees,
                commands,
            );
        }

        // Global: DeathOccurred(kind) on all entities.
        let trigger = Trigger::DeathOccurred(kind);
        for (entity, bound, staged) in global_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger, &context, &staged_trees, commands);
            walk_bound_effects(entity, &trigger, &context, &bound_trees, commands);
        }

        // Global: DeathOccurred(Any) on all entities.
        let trigger_any = Trigger::DeathOccurred(EntityKind::Any);
        for (entity, bound, staged) in global_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger_any, &context, &staged_trees, commands);
            walk_bound_effects(entity, &trigger_any, &context, &bound_trees, commands);
        }
    }
}

/// Bridge for cell deaths.
pub(crate) fn on_cell_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Cell,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

/// Bridge for bolt deaths.
pub(crate) fn on_bolt_destroyed(
    mut reader: MessageReader<Destroyed<Bolt>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Bolt,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

/// Bridge for wall deaths.
pub(crate) fn on_wall_destroyed(
    mut reader: MessageReader<Destroyed<Wall>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Wall,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

/// Bridge for breaker deaths.
pub(crate) fn on_breaker_destroyed(
    mut reader: MessageReader<Destroyed<Breaker>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Breaker,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

/// Bridge for salvo deaths.
pub(crate) fn on_salvo_destroyed(
    mut reader: MessageReader<Destroyed<Salvo>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Salvo,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}
