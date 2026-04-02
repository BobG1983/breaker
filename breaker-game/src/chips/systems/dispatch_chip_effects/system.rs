//! Dispatches chip effects to target entities when a chip is selected.
//!
//! Reads [`ChipSelected`] messages, resolves `RootEffect::On { target, then }`
//! to entities, pushes children to `BoundEffects`. Bare `Do` children fire
//! immediately via `commands.fire_effect()`.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    chips::{inventory::ChipInventory, resources::ChipCatalog},
    effect::{
        BoundEffects, EffectCommandsExt, EffectNode, RootEffect, StagedEffects, Target, Trigger,
        TriggerContext,
    },
    ui::messages::ChipSelected,
    wall::components::Wall,
};

/// Bundled entity queries for target resolution — reduces system parameter count.
#[derive(SystemParam)]
pub(crate) struct DispatchTargets<'w, 's> {
    breakers: Query<'w, 's, Entity, With<Breaker>>,
    bolts: Query<'w, 's, Entity, With<Bolt>>,
    cells: Query<'w, 's, Entity, With<Cell>>,
    walls: Query<'w, 's, Entity, With<Wall>>,
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

        for root_effect in &effects {
            let RootEffect::On { target, then } = root_effect;

            if *target == Target::Breaker {
                // Direct dispatch: breaker exists during ChipSelect
                let entities = resolve_target_entities(Target::Breaker, &targets);
                for entity in entities {
                    dispatch_children(entity, then, &chip_name, &targets, &mut commands);
                }
            } else {
                // Deferred dispatch: non-Breaker entities don't exist during ChipSelect.
                // Wrap in When(NodeStart, On(original_target, original_children))
                // and push to the Breaker's BoundEffects.
                let wrapped = EffectNode::When {
                    trigger: Trigger::NodeStart,
                    then: vec![EffectNode::On {
                        target: *target,
                        permanent: true,
                        then: then.clone(),
                    }],
                };
                for breaker_entity in targets.breakers.iter() {
                    commands.push_bound_effects(
                        breaker_entity,
                        vec![(chip_name.clone(), wrapped.clone())],
                    );
                }
            }
        }
    }
}

/// Dispatch a list of child [`EffectNode`]s to a single entity.
///
/// - `Do(effect)` children are fired immediately via `commands.fire_effect`.
/// - `On { target, then, .. }` children resolve the inner target and recurse.
/// - All other children are transferred to the entity's [`BoundEffects`].
fn dispatch_children(
    entity: Entity,
    children: &[EffectNode],
    chip_name: &str,
    targets: &DispatchTargets,
    commands: &mut Commands,
) {
    // Ensure BoundEffects and StagedEffects exist on the entity before dispatching
    commands
        .entity(entity)
        .insert_if_new(BoundEffects::default());
    commands
        .entity(entity)
        .insert_if_new(StagedEffects::default());

    for child in children {
        match child {
            EffectNode::Do(effect) => {
                commands.fire_effect(entity, effect.clone(), chip_name.to_owned());
            }
            EffectNode::On {
                target: inner_target,
                then: inner_children,
                ..
            } => {
                let inner_entities = resolve_target_entities(*inner_target, targets);
                for inner_entity in inner_entities {
                    dispatch_children(inner_entity, inner_children, chip_name, targets, commands);
                }
            }
            other => {
                commands.transfer_effect(
                    entity,
                    chip_name.to_owned(),
                    vec![other.clone()],
                    true,
                    TriggerContext::default(),
                );
            }
        }
    }
}

/// Map a [`Target`] to the set of matching entities.
fn resolve_target_entities(target: Target, targets: &DispatchTargets) -> Vec<Entity> {
    match target {
        Target::Breaker => targets.breakers.iter().collect(),
        Target::Bolt | Target::AllBolts => targets.bolts.iter().collect(),
        Target::Cell | Target::AllCells => targets.cells.iter().collect(),
        Target::Wall | Target::AllWalls => targets.walls.iter().collect(),
    }
}
