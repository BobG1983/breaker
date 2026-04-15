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
    prelude::*,
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
    rng:           ResMut<'w, GameRng>,
    active_layout: Option<Res<'w, ActiveNodeLayout>>,
}

/// Generates chip offerings using weighted random selection and inserts `ChipOffers`.
///
/// Runs `OnEnter(GameState::ChipSelect)`, before `spawn_chip_select`.
/// On boss nodes with eligible evolutions, evolution offerings take priority
/// and remaining slots are filled with normal offerings.
pub(crate) fn generate_chip_offerings(mut params: ChipOfferingParams) {
    // Build rarity weight map from config
    let rarity_weights = HashMap::from([
        (Rarity::Common, params.config.rarity_weight_common),
        (Rarity::Uncommon, params.config.rarity_weight_uncommon),
        (Rarity::Rare, params.config.rarity_weight_rare),
        (Rarity::Legendary, params.config.rarity_weight_legendary),
    ]);

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
    let normal_offers = generate_offerings(
        &params.registry,
        &params.inventory,
        &offering_config,
        &mut params.rng.0,
    );

    // Combine: evolutions first, then normal
    let mut chip_offers: Vec<ChipOffering> = evolution_offers;
    chip_offers.extend(normal_offers.into_iter().map(ChipOffering::Normal));

    // Insert offers resource
    params.commands.insert_resource(ChipOffers(chip_offers));
}

#[cfg(test)]
mod tests;
