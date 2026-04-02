use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use crate::chips::definition::{
    ChipDefinition, ChipTemplate, EvolutionIngredient, EvolutionTemplate,
};

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
/// iteration (chip offer display). Populated during loading by `build_chip_catalog`
/// from `ChipTemplateRegistry` and `EvolutionTemplateRegistry`.
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
    ///
    /// If a definition with the same name already exists, the `chips` map is
    /// overwritten but the `order` vec grows a duplicate entry. This means
    /// `ordered_values()` will yield the definition twice. Callers that need
    /// deduplication should check `get()` before inserting.
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
    ///
    /// Ingredient `chip_name` values are template names (e.g., "Splinter"), matched
    /// against the inventory's template-level taken count — not individual expanded
    /// chip names.
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
                    .all(|ing| inventory.template_taken(&ing.chip_name) >= ing.stacks_required)
            })
            .collect()
    }
}

/// Registry of chip templates loaded from `.chip.ron` files.
#[derive(Resource, Debug, Default)]
pub(crate) struct ChipTemplateRegistry {
    templates: HashMap<String, (AssetId<ChipTemplate>, ChipTemplate)>,
}

impl ChipTemplateRegistry {
    /// Look up a template by name.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn get(&self, name: &str) -> Option<&(AssetId<ChipTemplate>, ChipTemplate)> {
        self.templates.get(name)
    }

    /// Returns the number of templates in the registry.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.templates.len()
    }

    /// Iterate all chip templates.
    pub(crate) fn templates(&self) -> impl Iterator<Item = &ChipTemplate> {
        self.templates.values().map(|(_, t)| t)
    }
}

impl SeedableRegistry for ChipTemplateRegistry {
    type Asset = ChipTemplate;

    fn asset_dir() -> &'static str {
        "chips/standard"
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
pub(crate) struct EvolutionTemplateRegistry {
    evolutions: HashMap<String, (AssetId<EvolutionTemplate>, EvolutionTemplate)>,
}

impl EvolutionTemplateRegistry {
    /// Look up an evolution template by name.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn get(
        &self,
        name: &str,
    ) -> Option<&(AssetId<EvolutionTemplate>, EvolutionTemplate)> {
        self.evolutions.get(name)
    }

    /// Returns the number of evolution templates in the registry.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.evolutions.len()
    }

    /// Iterate all evolution templates.
    pub(crate) fn templates(&self) -> impl Iterator<Item = &EvolutionTemplate> {
        self.evolutions.values().map(|(_, t)| t)
    }
}

impl SeedableRegistry for EvolutionTemplateRegistry {
    type Asset = EvolutionTemplate;

    fn asset_dir() -> &'static str {
        "chips/evolutions"
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
