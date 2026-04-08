//! Bolt effect dispatch system -- resolves targets and pushes effects.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef},
        registry::BoltRegistry,
    },
    effect::*,
    prelude::*,
};

/// Dispatches bolt-defined effects to target entities.
///
/// Resolves `RootEffect::On { target, then }` from the bolt definition
/// and pushes non-`Do` children to target entity's `BoundEffects`.
/// Bare `Do` children are fired immediately via `commands.fire_effect()`.
///
/// Triggered by `Added<BoltDefinitionRef>` -- only runs when a new bolt
/// entity is spawned with a definition reference.
pub(crate) fn dispatch_bolt_effects(
    mut commands: Commands,
    registry: Res<BoltRegistry>,
    new_bolts: Query<(Entity, &BoltDefinitionRef), Added<BoltDefinitionRef>>,
    bolt_query: Query<Entity, With<Bolt>>,
    breaker_query: Query<Entity, With<Breaker>>,
    cell_query: Query<Entity, With<Cell>>,
    wall_query: Query<Entity, With<Wall>>,
) {
    for (_entity, def_ref) in &new_bolts {
        let Some(def) = registry.get(&def_ref.0) else {
            warn!("Bolt '{}' not found in registry", def_ref.0);
            continue;
        };

        for root_effect in &def.effects {
            let RootEffect::On { target, then } = root_effect;

            let mut do_effects = Vec::new();
            let mut bound_children = Vec::new();
            for child in then {
                match child {
                    EffectNode::Do(effect) => do_effects.push(effect.clone()),
                    // Bolt-sourced effects use empty source_chip -- they come from
                    // the bolt definition, not from an evolution chip.
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
                // Fire bare Do children immediately. Empty source_chip because
                // bolt-sourced effects are not attributed to any evolution chip.
                for effect in &do_effects {
                    commands.fire_effect(entity, effect.clone(), String::new());
                }

                if !bound_children.is_empty() {
                    commands.push_bound_effects(entity, bound_children.clone());
                }
            }
        }
    }
}
