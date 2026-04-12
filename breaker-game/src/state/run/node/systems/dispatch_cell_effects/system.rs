//! Dispatches cell-defined effects to target entities when spawned.
//!
//! Reads each cell's `CellTypeDefinition.effects` (optional) and stamps
//! trees onto the appropriate target entities.

use bevy::prelude::*;

use crate::{
    cells::{
        components::{CellEffectsDispatched, CellTypeAlias},
        resources::CellTypeRegistry,
    },
    effect_v3::{
        commands::EffectCommandsExt,
        types::{RootNode, StampTarget},
    },
    prelude::*,
};

/// Query for cells that have not yet had their effects dispatched.
type CellDispatchQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static CellTypeAlias), (With<Cell>, Without<CellEffectsDispatched>)>;

/// Dispatches effects from cell type definitions to target entities.
///
/// For each cell entity without `CellEffectsDispatched`, looks up the cell's
/// definition in `CellTypeRegistry` and processes each `RootNode`:
/// - `Stamp(target, tree)` — stamps the tree onto the resolved target entities
/// - `Spawn(kind, tree)` — deferred (spawn-watching not yet implemented)
///
/// Inserts `CellEffectsDispatched` marker after processing to prevent double-dispatch.
pub(crate) fn dispatch_cell_effects(
    mut commands: Commands,
    cell_query: CellDispatchQuery,
    registry: Option<Res<CellTypeRegistry>>,
    bolt_query: Query<Entity, With<Bolt>>,
    breaker_query: Query<Entity, With<Breaker>>,
    wall_query: Query<Entity, With<Wall>>,
    all_cells_query: Query<Entity, With<Cell>>,
) {
    let Some(registry) = registry else {
        return;
    };
    for (entity, alias) in &cell_query {
        let Some(def) = registry.get(&alias.0) else {
            continue;
        };

        let effects = match &def.effects {
            None => continue,
            Some(effects) if effects.is_empty() => continue,
            Some(effects) => effects,
        };

        for root in effects {
            match root {
                RootNode::Stamp(target, tree) => {
                    let target_entities: Vec<Entity> = match target {
                        StampTarget::Bolt
                        | StampTarget::ActiveBolts
                        | StampTarget::EveryBolt
                        | StampTarget::PrimaryBolts
                        | StampTarget::ExtraBolts => bolt_query.iter().collect(),
                        StampTarget::Breaker
                        | StampTarget::ActiveBreakers
                        | StampTarget::EveryBreaker => breaker_query
                            .single()
                            .map_or_else(|_| Vec::new(), |b| vec![b]),
                        StampTarget::ActiveCells | StampTarget::EveryCell => {
                            all_cells_query.iter().collect()
                        }
                        StampTarget::ActiveWalls | StampTarget::EveryWall => {
                            wall_query.iter().collect()
                        }
                    };

                    for target_entity in target_entities {
                        commands.stamp_effect(target_entity, String::new(), tree.clone());
                    }
                }
                RootNode::Spawn(_kind, _tree) => {
                    // Spawn-type effects handled by SpawnStampRegistry — deferred.
                }
            }
        }

        commands.entity(entity).insert(CellEffectsDispatched);
    }
}
