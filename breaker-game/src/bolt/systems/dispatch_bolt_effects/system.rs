//! Bolt effect dispatch system -- resolves targets and pushes effects.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef},
        registry::BoltRegistry,
    },
    effect_v3::{commands::EffectCommandsExt, types::*},
    prelude::*,
};

/// Dispatches bolt-defined effects to target entities.
///
/// Resolves `RootNode::Stamp(target, tree)` from the bolt definition
/// and stamps the tree onto matching target entities via
/// `commands.stamp_effect()`.
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

        for root in &def.effects {
            match root {
                RootNode::Stamp(target, tree) => {
                    let target_entities: Vec<Entity> = match target {
                        StampTarget::Breaker
                        | StampTarget::ActiveBreakers
                        | StampTarget::EveryBreaker => breaker_query.iter().collect(),
                        StampTarget::Bolt
                        | StampTarget::ActiveBolts
                        | StampTarget::EveryBolt
                        | StampTarget::PrimaryBolts
                        | StampTarget::ExtraBolts => bolt_query.iter().collect(),
                        StampTarget::ActiveCells | StampTarget::EveryCell => {
                            cell_query.iter().collect()
                        }
                        StampTarget::ActiveWalls | StampTarget::EveryWall => {
                            wall_query.iter().collect()
                        }
                    };

                    for entity in target_entities {
                        commands.stamp_effect(entity, String::new(), tree.clone());
                    }
                }
                RootNode::Spawn(_kind, _tree) => {
                    // Spawn-based roots are not dispatched at bolt spawn time.
                    // They are handled by the spawn stamp registry.
                }
            }
        }
    }
}
