//! Dispatches cell-defined effects to target entities when spawned.
//!
//! Reads each cell's `CellTypeDefinition.effects` (optional) and pushes
//! children to the appropriate target entity's `BoundEffects`.
//! Bare `Do` children are fired immediately via `commands.fire_effect()`.

use bevy::prelude::*;

use crate::{
    cells::{
        components::{CellEffectsDispatched, CellTypeAlias},
        resources::CellTypeRegistry,
    },
    effect::{EffectCommandsExt, Target},
    prelude::*,
};

/// Query for cells that have not yet had their effects dispatched.
type CellDispatchQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static CellTypeAlias), (With<Cell>, Without<CellEffectsDispatched>)>;

/// Dispatches effects from cell type definitions to target entities.
///
/// For each cell entity without `CellEffectsDispatched`, looks up the cell's
/// definition in `CellTypeRegistry` and processes `RootEffect::On { target, then }`:
/// - `Do` children are fired immediately
/// - Non-`Do` children are pushed to the target entity's `BoundEffects`
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
        let Some(def) = registry.get(alias.0) else {
            continue;
        };

        let effects = match &def.effects {
            None => continue,
            Some(effects) if effects.is_empty() => continue,
            Some(effects) => effects,
        };

        for root_effect in effects {
            let RootEffect::On { target, then } = root_effect;

            let mut non_do_children: Vec<(String, EffectNode)> = Vec::new();
            let mut do_children = Vec::new();
            for child in then {
                match child {
                    EffectNode::Do(effect) => do_children.push(effect.clone()),
                    // Cell-sourced effects use empty source_chip — they come from
                    // the cell type definition, not from an evolution chip.
                    other => non_do_children.push((String::new(), other.clone())),
                }
            }

            // Resolve target entities
            let target_entities: Vec<Entity> = match target {
                Target::Cell => vec![entity],
                Target::Bolt | Target::AllBolts => bolt_query.iter().collect(),
                Target::Breaker => breaker_query
                    .single()
                    .map_or_else(|_| Vec::new(), |breaker| vec![breaker]),
                Target::AllCells => all_cells_query.iter().collect(),
                Target::Wall | Target::AllWalls => wall_query.iter().collect(),
            };

            for target_entity in &target_entities {
                // Fire Do children immediately. Empty source_chip because
                // cell-sourced effects are not attributed to any evolution chip.
                for effect in &do_children {
                    commands.fire_effect(*target_entity, effect.clone(), String::new());
                }

                // Push non-Do children to BoundEffects
                if !non_do_children.is_empty() {
                    commands.push_bound_effects(*target_entity, non_do_children.clone());
                }
            }
        }

        commands.entity(entity).insert(CellEffectsDispatched);
    }
}
