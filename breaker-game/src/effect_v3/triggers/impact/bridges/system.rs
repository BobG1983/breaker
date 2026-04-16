//! Impact trigger bridge systems.
//!
//! Each bridge reads collision messages, builds a [`TriggerContext`], and dispatches
//! the corresponding trigger to entities with bound effects.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    cells::messages::{CellImpactWall, SalvoImpactBreaker},
    effect_v3::{
        storage::{BoundEffects, StagedEffects},
        types::{EntityKind, Trigger, TriggerContext},
        walking::{walk_bound_effects, walk_staged_effects},
    },
};

/// Bundled message readers for all collision types — avoids `too_many_arguments`.
#[derive(SystemParam)]
pub(crate) struct ImpactReaders<'w, 's> {
    bolt_cell:     MessageReader<'w, 's, BoltImpactCell>,
    bolt_wall:     MessageReader<'w, 's, BoltImpactWall>,
    bolt_breaker:  MessageReader<'w, 's, BoltImpactBreaker>,
    breaker_cell:  MessageReader<'w, 's, BreakerImpactCell>,
    breaker_wall:  MessageReader<'w, 's, BreakerImpactWall>,
    cell_wall:     MessageReader<'w, 's, CellImpactWall>,
    salvo_breaker: MessageReader<'w, 's, SalvoImpactBreaker>,
}

/// Local bridge: fires `Impacted(entity_kind)` on entities involved in a collision.
///
/// For each collision message, both participants receive the trigger with the
/// `EntityKind` of the *other* entity. E.g., when a bolt hits a cell, the bolt
/// gets `Impacted(Cell)` and the cell gets `Impacted(Bolt)`.
pub(crate) fn on_impacted(
    mut readers: ImpactReaders,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    for msg in readers.bolt_cell.read() {
        dispatch_local(
            msg.bolt,
            EntityKind::Bolt,
            msg.cell,
            EntityKind::Cell,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.bolt_wall.read() {
        dispatch_local(
            msg.bolt,
            EntityKind::Bolt,
            msg.wall,
            EntityKind::Wall,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.bolt_breaker.read() {
        dispatch_local(
            msg.bolt,
            EntityKind::Bolt,
            msg.breaker,
            EntityKind::Breaker,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.breaker_cell.read() {
        dispatch_local(
            msg.breaker,
            EntityKind::Breaker,
            msg.cell,
            EntityKind::Cell,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.breaker_wall.read() {
        dispatch_local(
            msg.breaker,
            EntityKind::Breaker,
            msg.wall,
            EntityKind::Wall,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.cell_wall.read() {
        dispatch_local(
            msg.cell,
            EntityKind::Cell,
            msg.wall,
            EntityKind::Wall,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.salvo_breaker.read() {
        dispatch_local(
            msg.salvo,
            EntityKind::Salvo,
            msg.breaker,
            EntityKind::Breaker,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `ImpactOccurred(entity_kind)` on all entities with bound
/// effects when a collision involving the given entity kind happens.
pub(crate) fn on_impact_occurred(
    mut readers: ImpactReaders,
    bound_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    // Collect all entity kinds involved in collisions this frame
    let mut kinds = Vec::new();
    for msg in readers.bolt_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.cell,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Cell, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.bolt_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.bolt_breaker.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.breaker,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.breaker_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.cell,
        };
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Cell, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.breaker_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.cell_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.cell,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Cell, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.salvo_breaker.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.salvo,
            impactee: msg.breaker,
        };
        kinds.push((EntityKind::Salvo, ctx.clone()));
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }

    for (kind, ctx) in &kinds {
        let trigger = Trigger::ImpactOccurred(*kind);
        for (entity, bound, staged) in bound_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger, ctx, &staged_trees, &mut commands);
            walk_bound_effects(entity, &trigger, ctx, &bound_trees, &mut commands);
        }
    }
}

fn dispatch_local(
    entity_a: Entity,
    kind_a: EntityKind,
    entity_b: Entity,
    kind_b: EntityKind,
    bound_query: &Query<(&BoundEffects, Option<&StagedEffects>)>,
    commands: &mut Commands,
) {
    let ctx = TriggerContext::Impact {
        impactor: entity_a,
        impactee: entity_b,
    };
    walk_local_impact(
        entity_a,
        kind_b,
        entity_b,
        kind_a,
        &ctx,
        bound_query,
        commands,
    );
}

/// Walk effects on both collision participants (local dispatch).
///
/// Walks staged entries before bound entries for each entity so a
/// freshly-armed inner gate does not match the same-tick trigger.
fn walk_local_impact(
    entity_a: Entity,
    kind_b: EntityKind,
    entity_b: Entity,
    kind_a: EntityKind,
    context: &TriggerContext,
    bound_query: &Query<(&BoundEffects, Option<&StagedEffects>)>,
    commands: &mut Commands,
) {
    // Entity A gets Impacted(kind_of_B) — and also Impacted(Any).
    if let Ok((bound, staged)) = bound_query.get(entity_a) {
        let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
        let bound_trees = bound.0.clone();

        // Walk staged first for both trigger variants against the same
        // snapshot — matches the existing clone-once pattern for bound.
        walk_staged_effects(
            entity_a,
            &Trigger::Impacted(kind_b),
            context,
            &staged_trees,
            commands,
        );
        walk_staged_effects(
            entity_a,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &staged_trees,
            commands,
        );

        walk_bound_effects(
            entity_a,
            &Trigger::Impacted(kind_b),
            context,
            &bound_trees,
            commands,
        );
        walk_bound_effects(
            entity_a,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &bound_trees,
            commands,
        );
    }
    // Entity B gets Impacted(kind_of_A) — and also Impacted(Any).
    if let Ok((bound, staged)) = bound_query.get(entity_b) {
        let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
        let bound_trees = bound.0.clone();

        walk_staged_effects(
            entity_b,
            &Trigger::Impacted(kind_a),
            context,
            &staged_trees,
            commands,
        );
        walk_staged_effects(
            entity_b,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &staged_trees,
            commands,
        );

        walk_bound_effects(
            entity_b,
            &Trigger::Impacted(kind_a),
            context,
            &bound_trees,
            commands,
        );
        walk_bound_effects(
            entity_b,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &bound_trees,
            commands,
        );
    }
}
