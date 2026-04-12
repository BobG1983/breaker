//! Death trigger bridge systems.
//!
//! Each bridge reads `Destroyed<T>` messages and dispatches `Died`, `Killed`, and
//! `DeathOccurred` triggers to entities with bound effects.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect_v3::{
        storage::BoundEffects,
        types::{EntityKind, Trigger, TriggerContext},
        walking::walk_effects,
    },
    shared::death_pipeline::{Destroyed, GameEntity},
    walls::components::Wall,
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
    bound_query: &Query<&BoundEffects>,
    global_query: &Query<(Entity, &BoundEffects)>,
    commands: &mut Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Death {
            victim: msg.victim,
            killer: msg.killer,
        };

        // Local: Died on victim
        if let Ok(bound) = bound_query.get(msg.victim) {
            let trees = bound.0.clone();
            walk_effects(msg.victim, &Trigger::Died, &context, &trees, commands);
        }

        // Local: Killed(kind) on killer
        if let Some(killer) = msg.killer
            && let Ok(bound) = bound_query.get(killer)
        {
            let trees = bound.0.clone();
            walk_effects(killer, &Trigger::Killed(kind), &context, &trees, commands);
        }

        // Global: DeathOccurred(kind) on all entities
        let trigger = Trigger::DeathOccurred(kind);
        for (entity, bound) in global_query.iter() {
            let trees = bound.0.clone();
            walk_effects(entity, &trigger, &context, &trees, commands);
        }
    }
}

/// Bridge for cell deaths.
pub fn on_cell_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    bound_query: Query<&BoundEffects>,
    global_query: Query<(Entity, &BoundEffects)>,
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
pub fn on_bolt_destroyed(
    mut reader: MessageReader<Destroyed<Bolt>>,
    bound_query: Query<&BoundEffects>,
    global_query: Query<(Entity, &BoundEffects)>,
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
pub fn on_wall_destroyed(
    mut reader: MessageReader<Destroyed<Wall>>,
    bound_query: Query<&BoundEffects>,
    global_query: Query<(Entity, &BoundEffects)>,
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
pub fn on_breaker_destroyed(
    mut reader: MessageReader<Destroyed<Breaker>>,
    bound_query: Query<&BoundEffects>,
    global_query: Query<(Entity, &BoundEffects)>,
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
