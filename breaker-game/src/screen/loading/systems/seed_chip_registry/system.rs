//! Seeds `ChipRegistry` from loaded `ChipDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    chips::{
        ChipDefinition, ChipRegistry, Recipe,
        definition::{ChipTemplate, Rarity, expand_template},
    },
    screen::loading::resources::DefaultsCollection,
};

/// Iterates loaded `ChipTemplate` assets from the chip templates collection,
/// expands each into `ChipDefinition`s, and builds the `ChipRegistry` resource.
///
/// Evolution chips are inserted alongside normal chips and also have their
/// recipes extracted into `ChipRegistry::insert_recipe`.
pub(crate) fn seed_chip_registry(
    collection: Option<Res<DefaultsCollection>>,
    chip_assets: Res<Assets<ChipDefinition>>,
    template_assets: Res<Assets<ChipTemplate>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let mut registry = ChipRegistry::default();

    // Expand chip templates into definitions
    for handle in &collection.chip_templates {
        let Some(template) = template_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        for def in expand_template(template) {
            registry.insert(def);
        }
    }

    // Also load ChipDefinition assets (for backward compatibility —
    // remove this block once all non-evolution chips are defined as ChipTemplates.)
    for handle in &collection.chips {
        let Some(def) = chip_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        if def.rarity == Rarity::Evolution {
            let recipe = Recipe {
                ingredients: def.ingredients.clone().unwrap_or_default(),
                result_name: def.name.clone(),
            };
            registry.insert_recipe(recipe);
        }
        registry.insert(def.clone());
    }

    commands.insert_resource(registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}
