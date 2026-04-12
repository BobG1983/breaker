//! Impact trigger bridge systems.
//!
//! Each bridge reads collision messages, builds a [`TriggerContext`], and dispatches
//! the corresponding trigger to entities with bound effects.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    cells::messages::CellImpactWall,
    effect_v3::{
        storage::BoundEffects,
        types::{EntityKind, Trigger, TriggerContext},
        walking::walk_effects,
    },
};

/// Bundled message readers for all collision types — avoids `too_many_arguments`.
#[derive(SystemParam)]
pub(crate) struct ImpactReaders<'w, 's> {
    bolt_cell:    MessageReader<'w, 's, BoltImpactCell>,
    bolt_wall:    MessageReader<'w, 's, BoltImpactWall>,
    bolt_breaker: MessageReader<'w, 's, BoltImpactBreaker>,
    breaker_cell: MessageReader<'w, 's, BreakerImpactCell>,
    breaker_wall: MessageReader<'w, 's, BreakerImpactWall>,
    cell_wall:    MessageReader<'w, 's, CellImpactWall>,
}

/// Local bridge: fires `Impacted(entity_kind)` on entities involved in a collision.
///
/// For each collision message, both participants receive the trigger with the
/// `EntityKind` of the *other* entity. E.g., when a bolt hits a cell, the bolt
/// gets `Impacted(Cell)` and the cell gets `Impacted(Bolt)`.
pub(crate) fn on_impacted(
    mut readers: ImpactReaders,
    bound_query: Query<&BoundEffects>,
    mut commands: Commands,
) {
    for msg in readers.bolt_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.cell,
        };
        walk_local_impact(
            msg.bolt,
            EntityKind::Cell,
            msg.cell,
            EntityKind::Bolt,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.bolt_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.wall,
        };
        walk_local_impact(
            msg.bolt,
            EntityKind::Wall,
            msg.wall,
            EntityKind::Bolt,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.bolt_breaker.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.breaker,
        };
        walk_local_impact(
            msg.bolt,
            EntityKind::Breaker,
            msg.breaker,
            EntityKind::Bolt,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.breaker_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.cell,
        };
        walk_local_impact(
            msg.breaker,
            EntityKind::Cell,
            msg.cell,
            EntityKind::Breaker,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.breaker_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.wall,
        };
        walk_local_impact(
            msg.breaker,
            EntityKind::Wall,
            msg.wall,
            EntityKind::Breaker,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.cell_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.cell,
            impactee: msg.wall,
        };
        walk_local_impact(
            msg.cell,
            EntityKind::Wall,
            msg.wall,
            EntityKind::Cell,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `ImpactOccurred(entity_kind)` on all entities with bound
/// effects when a collision involving the given entity kind happens.
pub(crate) fn on_impact_occurred(
    mut readers: ImpactReaders,
    bound_query: Query<(Entity, &BoundEffects)>,
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
        kinds.push((EntityKind::Cell, ctx));
    }
    for msg in readers.bolt_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx));
    }
    for msg in readers.bolt_breaker.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.breaker,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Breaker, ctx));
    }
    for msg in readers.breaker_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.cell,
        };
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Cell, ctx));
    }
    for msg in readers.breaker_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx));
    }
    for msg in readers.cell_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.cell,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Cell, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx));
    }

    for (kind, ctx) in &kinds {
        let trigger = Trigger::ImpactOccurred(*kind);
        for (entity, bound) in bound_query.iter() {
            let trees = bound.0.clone();
            walk_effects(entity, &trigger, ctx, &trees, &mut commands);
        }
    }
}

/// Walk effects on both collision participants (local dispatch).
fn walk_local_impact(
    entity_a: Entity,
    kind_b: EntityKind,
    entity_b: Entity,
    kind_a: EntityKind,
    context: &TriggerContext,
    bound_query: &Query<&BoundEffects>,
    commands: &mut Commands,
) {
    // Entity A gets Impacted(kind_of_B) — and also Impacted(Any)
    if let Ok(bound) = bound_query.get(entity_a) {
        let trees = bound.0.clone();
        walk_effects(
            entity_a,
            &Trigger::Impacted(kind_b),
            context,
            &trees,
            commands,
        );
        walk_effects(
            entity_a,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &trees,
            commands,
        );
    }
    // Entity B gets Impacted(kind_of_A) — and also Impacted(Any)
    if let Ok(bound) = bound_query.get(entity_b) {
        let trees = bound.0.clone();
        walk_effects(
            entity_b,
            &Trigger::Impacted(kind_a),
            context,
            &trees,
            commands,
        );
        walk_effects(
            entity_b,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &trees,
            commands,
        );
    }
}
