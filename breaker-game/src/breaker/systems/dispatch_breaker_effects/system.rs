//! Breaker effect dispatch system — resolves targets and pushes effects.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    bolt::components::Bolt,
    breaker::{SelectedBreaker, components::Breaker, registry::BreakerRegistry},
    cells::components::Cell,
    effect::*,
    wall::components::Wall,
};

/// Dispatches breaker-defined effects to target entities.
///
/// Resolves `RootEffect::On { target, then }` from the breaker definition
/// and pushes non-`Do` children to target entity's `BoundEffects`.
/// Bare `Do` children are fired immediately via `commands.fire_effect()`.
pub(crate) fn dispatch_breaker_effects(
    mut commands: Commands,
    selected: Res<SelectedBreaker>,
    registry: Res<BreakerRegistry>,
    breaker_query: Query<Entity, With<Breaker>>,
    bolt_query: Query<Entity, With<Bolt>>,
    cell_query: Query<Entity, With<Cell>>,
    wall_query: Query<Entity, With<Wall>>,
) {
    // No breaker entity — nothing to dispatch
    if breaker_query.is_empty() {
        return;
    }

    let Some(def) = registry.get(&selected.0) else {
        warn!("Breaker '{}' not found in registry", selected.0);
        return;
    };

    for root_effect in &def.effects {
        let RootEffect::On { target, then } = root_effect;

        let mut do_effects = Vec::new();
        let mut bound_children = Vec::new();
        for child in then {
            match child {
                EffectNode::Do(effect) => do_effects.push(effect.clone()),
                other => bound_children.push((String::new(), other.clone())),
            }
        }

        // Determine target entities based on target type
        let target_entities: Vec<Entity> = match target {
            Target::Breaker => breaker_query.iter().collect(),
            Target::Bolt | Target::AllBolts => bolt_query.iter().collect(),
            Target::Cell | Target::AllCells => cell_query.iter().collect(),
            Target::Wall | Target::AllWalls => wall_query.iter().collect(),
        };

        for entity in target_entities {
            // Fire bare Do children immediately
            for effect in &do_effects {
                commands.fire_effect(entity, effect.clone(), String::new());
            }

            if !bound_children.is_empty() {
                commands.push_bound_effects(entity, bound_children.clone());
            }
        }
    }
}
