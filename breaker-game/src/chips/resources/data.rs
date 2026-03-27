use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use crate::chips::definition::{ChipDefinition, ChipTemplate, EvolutionIngredient, EvolutionTemplate};

/// A recipe combining ingredient chips into an evolved chip.
/// Stores only the result name — the full `ChipDefinition` is in `ChipCatalog.chips`.
#[derive(Clone, Debug)]
pub struct Recipe {
    /// Chips consumed by this evolution.
    pub ingredients: Vec<EvolutionIngredient>,
    /// Name of the chip produced when this recipe is fulfilled.
    pub result_name: String,
}

/// `HashMap` pool of all loaded chip definitions, keyed by name.
///
/// Preserves insertion order via a separate `Vec<String>` for deterministic
/// iteration (chip offer display). Populated during loading by `seed_chip_registry`.
#[derive(Resource, Debug, Default)]
pub struct ChipCatalog {
    chips: HashMap<String, ChipDefinition>,
    order: Vec<String>,
    recipes: Vec<Recipe>,
}

impl ChipCatalog {
    /// Look up a chip by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ChipDefinition> {
        self.chips.get(name)
    }

    /// Iterate all chip definitions in insertion order.
    pub fn ordered_values(&self) -> impl Iterator<Item = &ChipDefinition> {
        self.order.iter().filter_map(|name| self.chips.get(name))
    }

    /// Insert a chip definition, keyed by its name.
    pub fn insert(&mut self, def: ChipDefinition) {
        let name = def.name.clone();
        self.chips.insert(name.clone(), def);
        self.order.push(name);
    }

    /// Add a recipe to the registry.
    pub fn insert_recipe(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Returns a slice of all recipes.
    #[must_use]
    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Returns recipes whose ingredients are all satisfied by the given inventory.
    #[must_use]
    pub fn eligible_recipes(
        &self,
        inventory: &crate::chips::inventory::ChipInventory,
    ) -> Vec<&Recipe> {
        self.recipes
            .iter()
            .filter(|recipe| {
                recipe
                    .ingredients
                    .iter()
                    .all(|ing| inventory.stacks(&ing.chip_name) >= ing.stacks_required)
            })
            .collect()
    }
}

/// Registry of chip templates loaded from `.chip.ron` files.
#[derive(Resource, Debug, Default)]
pub struct ChipTemplateRegistry {
    templates: HashMap<String, (AssetId<ChipTemplate>, ChipTemplate)>,
}

impl ChipTemplateRegistry {
    /// Look up a template by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&(AssetId<ChipTemplate>, ChipTemplate)> {
        self.templates.get(name)
    }

    /// Returns the number of templates in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.templates.len()
    }

    /// Returns `true` if the registry contains no templates.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    /// Iterate all chip templates.
    pub fn templates(&self) -> impl Iterator<Item = &ChipTemplate> {
        self.templates.values().map(|(_, t)| t)
    }
}

impl SeedableRegistry for ChipTemplateRegistry {
    type Asset = ChipTemplate;

    fn asset_dir() -> &'static str {
        "chips/templates"
    }

    fn extensions() -> &'static [&'static str] {
        &["chip.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<ChipTemplate>, ChipTemplate)]) {
        self.templates.clear();
        for (id, template) in assets {
            self.templates
                .insert(template.name.clone(), (*id, template.clone()));
        }
    }

    fn update_single(&mut self, id: AssetId<ChipTemplate>, asset: &ChipTemplate) {
        self.templates
            .insert(asset.name.clone(), (id, asset.clone()));
    }
}

/// Registry of evolution templates loaded from `.evolution.ron` files.
#[derive(Resource, Debug, Default)]
pub struct EvolutionTemplateRegistry {
    evolutions: HashMap<String, (AssetId<EvolutionTemplate>, EvolutionTemplate)>,
}

impl EvolutionTemplateRegistry {
    /// Look up an evolution template by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&(AssetId<EvolutionTemplate>, EvolutionTemplate)> {
        self.evolutions.get(name)
    }

    /// Returns the number of evolution templates in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.evolutions.len()
    }

    /// Returns `true` if the registry contains no evolution templates.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.evolutions.is_empty()
    }

    /// Iterate all evolution templates.
    pub fn templates(&self) -> impl Iterator<Item = &EvolutionTemplate> {
        self.evolutions.values().map(|(_, t)| t)
    }
}

impl SeedableRegistry for EvolutionTemplateRegistry {
    type Asset = EvolutionTemplate;

    fn asset_dir() -> &'static str {
        "chips/evolution"
    }

    fn extensions() -> &'static [&'static str] {
        &["evolution.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<EvolutionTemplate>, EvolutionTemplate)]) {
        self.evolutions.clear();
        for (id, template) in assets {
            self.evolutions
                .insert(template.name.clone(), (*id, template.clone()));
        }
    }

    fn update_single(&mut self, id: AssetId<EvolutionTemplate>, asset: &EvolutionTemplate) {
        self.evolutions
            .insert(asset.name.clone(), (id, asset.clone()));
    }
}
