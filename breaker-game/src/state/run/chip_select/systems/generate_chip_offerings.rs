//! System to generate weighted random chip offerings before the selection screen.

use std::collections::HashMap;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    chips::{
        ChipCatalog,
        definition::Rarity,
        inventory::ChipInventory,
        offering::{OfferingConfig, generate_offerings},
    },
    mutators::protocols::greed::{GreedConfig, GreedStacks, apply_greed_boost},
    prelude::*,
    shared::rng::ChipRng,
    state::run::{
        chip_select::{
            ChipSelectConfig,
            resources::{ChipOffering, ChipOffers},
        },
        node::{ActiveNodeLayout, definition::NodePool},
    },
};

/// Bundled parameters for chip offering generation.
#[derive(SystemParam)]
pub(crate) struct ChipOfferingParams<'w, 's> {
    commands:      Commands<'w, 's>,
    registry:      Res<'w, ChipCatalog>,
    inventory:     Res<'w, ChipInventory>,
    config:        Res<'w, ChipSelectConfig>,
    rng:           Res<'w, ChipRng>,
    active_layout: Option<Res<'w, ActiveNodeLayout>>,
    greed_config:  Option<Res<'w, GreedConfig>>,
    greed_stacks:  Option<Res<'w, GreedStacks>>,
}

/// Generates chip offerings using weighted random selection and inserts `ChipOffers`.
///
/// Runs `OnEnter(GameState::ChipSelect)`, before `spawn_chip_select`.
/// On boss nodes with eligible evolutions, evolution offerings take priority
/// and remaining slots are filled with normal offerings.
pub(crate) fn generate_chip_offerings(mut params: ChipOfferingParams) {
    // Build rarity weight map from config
    let mut rarity_weights = HashMap::from([
        (Rarity::Common, params.config.rarity_weight_common),
        (Rarity::Uncommon, params.config.rarity_weight_uncommon),
        (Rarity::Rare, params.config.rarity_weight_rare),
    ]);

    // Apply Greed rarity boost when the protocol is active. Harness-safe:
    // both resources must be present for the boost to apply.
    if let (Some(greed_config), Some(greed_stacks)) = (&params.greed_config, &params.greed_stacks) {
        apply_greed_boost(&mut rarity_weights, **greed_stacks, **greed_config);
    }

    // Check for boss node with eligible evolutions
    let mut evolution_offers: Vec<ChipOffering> = Vec::new();
    if let Some(layout) = &params.active_layout
        && layout.0.pool == NodePool::Boss
    {
        let eligible = params.registry.eligible_recipes(&params.inventory);
        for recipe in eligible.iter().take(params.config.offers_per_node) {
            if let Some(result_def) = params.registry.get(&recipe.result_name) {
                evolution_offers.push(ChipOffering::Evolution {
                    ingredients: recipe.ingredients.clone(),
                    result:      result_def.clone(),
                });
            }
        }
    }

    // Fill remaining slots with normal offerings
    let remaining_slots = params
        .config
        .offers_per_node
        .saturating_sub(evolution_offers.len());
    let offering_config = OfferingConfig {
        rarity_weights,
        offers_per_node: remaining_slots,
    };
    // ChipRng is intentionally NOT advanced by this system. `reseed_chip_rng` runs
    // in `ChipSelectSystems::ReseedRng` immediately before this system and installs
    // a fresh seeded state derived from (run_seed, "chip", ChipSelectCount); cloning
    // for the draw means any later observer reads the same canonical state, which
    // keeps `ChipRng` an idempotent function of (run_seed, ChipSelectCount) instead
    // of a stateful stream that drifts across systems within one selection cycle.
    let mut rng_clone = params.rng.0.clone();
    let normal_offers = generate_offerings(
        &params.registry,
        &params.inventory,
        &offering_config,
        &mut rng_clone,
    );

    // Combine: evolutions first, then normal
    let mut chip_offers: Vec<ChipOffering> = evolution_offers;
    chip_offers.extend(normal_offers.into_iter().map(ChipOffering::Normal));

    // Insert offers resource
    params.commands.insert_resource(ChipOffers(chip_offers));
}

#[cfg(test)]
mod tests;
