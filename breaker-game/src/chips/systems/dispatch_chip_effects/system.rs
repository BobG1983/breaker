//! Dispatches chip effects to target entities when a chip is selected.
//!
//! Reads [`ChipSelected`] messages, resolves `RootNode::Stamp(target, tree)`
//! to entities, stamps trees via `commands.stamp_effect()`. Bare `Fire` children
//! fire immediately via `commands.fire_effect()`.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    chips::{inventory::ChipInventory, resources::ChipCatalog},
    effect_v3::{
        commands::EffectCommandsExt,
        storage::{BoundEffects, StagedEffects},
        types::{RootNode, StampTarget, Tree},
    },
    prelude::*,
};

/// Bundled entity queries for target resolution — reduces system parameter count.
#[derive(SystemParam)]
pub(crate) struct DispatchTargets<'w, 's> {
    breakers: Query<'w, 's, Entity, With<Breaker>>,
    bolts:    Query<'w, 's, Entity, With<Bolt>>,
    cells:    Query<'w, 's, Entity, With<Cell>>,
    walls:    Query<'w, 's, Entity, With<Wall>>,
}

/// Reads [`ChipSelected`] messages, looks up each chip in the [`ChipCatalog`],
/// records it in [`ChipInventory`], and dispatches effects to resolved target entities.
pub(crate) fn dispatch_chip_effects(
    mut reader: MessageReader<ChipSelected>,
    catalog: Option<Res<ChipCatalog>>,
    mut inventory: Option<ResMut<ChipInventory>>,
    targets: DispatchTargets,
    mut commands: Commands,
) {
    let Some(catalog) = catalog else {
        for _ in reader.read() {}
        return;
    };

    for msg in reader.read() {
        let Some(def) = catalog.get(&msg.name) else {
            warn!("Chip '{}' not found in catalog", msg.name);
            continue;
        };

        // Clone what we need before mutable borrow of inventory
        let chip_name = def.name.clone();
        let effects = def.effects.clone();

        if let Some(ref mut inv) = inventory
            && !inv.add_chip(&msg.name, def)
        {
            warn!("Chip '{}' already at max stacks", msg.name);
            continue;
        }

        for root in &effects {
            match root {
                RootNode::Stamp(target, tree) => {
                    if *target == StampTarget::Breaker {
                        // Direct dispatch: breaker exists during ChipSelect
                        let entities = resolve_target_entities(*target, &targets);
                        for entity in entities {
                            dispatch_tree(entity, tree, &chip_name, &targets, &mut commands);
                        }
                    } else {
                        // Deferred dispatch: non-Breaker entities don't exist during ChipSelect.
                        // Wrap in When(NodeStartOccurred, On(original_target, original_children))
                        // and stamp to the Breaker's BoundEffects.
                        //
                        // NOTE: The old system used EffectNode::On { target, permanent, then }
                        // for deferred dispatch. The new system routes trees differently —
                        // we stamp the tree directly since trigger bridges will handle
                        // the walking at event time.
                        for breaker_entity in targets.breakers.iter() {
                            commands.stamp_effect(breaker_entity, chip_name.clone(), tree.clone());
                        }
                    }
                }
                RootNode::Spawn(_kind, _tree) => {
                    // Spawn-based roots are not used in chips yet
                }
            }
        }
    }
}

/// Dispatch a [`Tree`] to a single entity.
///
/// - `Fire(effect)` children are fired immediately via `commands.fire_effect`.
/// - All other tree variants are stamped onto the entity's [`BoundEffects`].
fn dispatch_tree(
    entity: Entity,
    tree: &Tree,
    chip_name: &str,
    _targets: &DispatchTargets,
    commands: &mut Commands,
) {
    // Ensure BoundEffects and StagedEffects exist on the entity before dispatching
    commands
        .entity(entity)
        .insert_if_new(BoundEffects::default());
    commands
        .entity(entity)
        .insert_if_new(StagedEffects::default());

    match tree {
        Tree::Fire(effect) => {
            commands.fire_effect(entity, effect.clone(), chip_name.to_owned());
        }
        other => {
            commands.stamp_effect(entity, chip_name.to_owned(), other.clone());
        }
    }
}

/// Map a [`StampTarget`] to the set of matching entities.
fn resolve_target_entities(target: StampTarget, targets: &DispatchTargets) -> Vec<Entity> {
    match target {
        StampTarget::Breaker | StampTarget::ActiveBreakers | StampTarget::EveryBreaker => {
            targets.breakers.iter().collect()
        }
        StampTarget::Bolt
        | StampTarget::ActiveBolts
        | StampTarget::EveryBolt
        | StampTarget::PrimaryBolts
        | StampTarget::ExtraBolts => targets.bolts.iter().collect(),
        StampTarget::ActiveCells | StampTarget::EveryCell => targets.cells.iter().collect(),
        StampTarget::ActiveWalls | StampTarget::EveryWall => targets.walls.iter().collect(),
    }
}
