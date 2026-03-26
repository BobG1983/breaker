//! Thin dispatcher: reads [`ChipSelected`] messages, looks up the chip in the
//! [`ChipRegistry`], and dispatches effects via `RootEffect::On` target routing.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    chips::{inventory::ChipInventory, resources::ChipRegistry},
    effect::{
        definition::{EffectChains, EffectNode, RootEffect, Target},
        typed_events::fire_passive_event,
    },
    ui::messages::ChipSelected,
};

/// Reads [`ChipSelected`] messages, looks up the chip definition in the
/// [`ChipRegistry`], and dispatches effects via `RootEffect::On` target routing.
///
/// For each `On { target, then }` node in the chip's effects:
/// - `Do(eff)` children fire as passive events immediately
/// - `When`/`Once`/`Until`/`On` children push to the target entity's `EffectChains`
pub(crate) fn dispatch_chip_effects(
    mut reader: MessageReader<ChipSelected>,
    registry: Option<Res<ChipRegistry>>,
    mut inventory: Option<ResMut<ChipInventory>>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut bolt_query: Query<&mut EffectChains, (With<Bolt>, Without<Breaker>)>,
    mut commands: Commands,
) {
    let Some(registry) = registry else {
        return;
    };
    for msg in reader.read() {
        let Some(chip) = registry.get(&msg.name) else {
            debug!("chip not found in registry: {}", msg.name);
            continue;
        };
        if let Some(inv) = inventory.as_mut() {
            let _ = inv.add_chip(&msg.name, chip);
        }
        for root in &chip.effects {
            let RootEffect::On { target, then } = root;
            for child in then {
                match child {
                    EffectNode::Do(eff) => {
                        fire_passive_event(
                            eff.clone(),
                            chip.max_stacks,
                            msg.name.clone(),
                            &mut commands,
                        );
                    }
                    node => {
                        let entry = (Some(msg.name.clone()), node.clone());
                        match target {
                            Target::Bolt => {
                                for mut chains in &mut bolt_query {
                                    chains.0.push(entry.clone());
                                }
                            }
                            Target::Breaker => {
                                for mut chains in &mut breaker_query {
                                    chains.0.push(entry.clone());
                                }
                            }
                            _ => {
                                warn!("dispatch_chip_effects: unhandled target {:?}", target);
                            }
                        }
                    }
                }
            }
        }
    }
}
