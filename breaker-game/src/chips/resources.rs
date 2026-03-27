//! Chip registry — `HashMap` pool of all loaded chip definitions.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::definition::{ChipDefinition, ChipTemplate, EvolutionIngredient};

/// A recipe combining ingredient chips into an evolved chip.
/// Stores only the result name — the full `ChipDefinition` is in `ChipCatalog.chips`.
#[derive(Clone, Debug)]
pub(crate) struct Recipe {
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
pub(crate) struct ChipCatalog {
    chips: HashMap<String, ChipDefinition>,
    order: Vec<String>,
    recipes: Vec<Recipe>,
}

impl ChipCatalog {
    /// Look up a chip by name.
    #[must_use]
    pub(crate) fn get(&self, name: &str) -> Option<&ChipDefinition> {
        self.chips.get(name)
    }

    /// Iterate all chip definitions in insertion order.
    pub(crate) fn ordered_values(&self) -> impl Iterator<Item = &ChipDefinition> {
        self.order.iter().filter_map(|name| self.chips.get(name))
    }

    /// Insert a chip definition, keyed by its name.
    pub(crate) fn insert(&mut self, def: ChipDefinition) {
        let name = def.name.clone();
        self.chips.insert(name.clone(), def);
        self.order.push(name);
    }

    /// Add a recipe to the registry.
    pub(crate) fn insert_recipe(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Returns a slice of all recipes.
    #[must_use]
    pub(crate) fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Returns recipes whose ingredients are all satisfied by the given inventory.
    #[must_use]
    pub(crate) fn eligible_recipes(
        &self,
        inventory: &super::inventory::ChipInventory,
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
pub(crate) struct ChipTemplateRegistry {
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

/// Registry of evolution chip definitions loaded from `.evolution.ron` files.
#[derive(Resource, Debug, Default)]
pub(crate) struct EvolutionRegistry {
    evolutions: HashMap<String, (AssetId<ChipDefinition>, ChipDefinition)>,
}

impl EvolutionRegistry {
    /// Look up an evolution definition by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&(AssetId<ChipDefinition>, ChipDefinition)> {
        self.evolutions.get(name)
    }

    /// Returns the number of evolution definitions in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.evolutions.len()
    }

    /// Returns `true` if the registry contains no evolution definitions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.evolutions.is_empty()
    }

    /// Iterate all evolution chip definitions.
    pub fn definitions(&self) -> impl Iterator<Item = &ChipDefinition> {
        self.evolutions.values().map(|(_, d)| d)
    }
}

impl SeedableRegistry for EvolutionRegistry {
    type Asset = ChipDefinition;

    fn asset_dir() -> &'static str {
        "chips/evolution"
    }

    fn extensions() -> &'static [&'static str] {
        &["evolution.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<ChipDefinition>, ChipDefinition)]) {
        self.evolutions.clear();
        for (id, def) in assets {
            self.evolutions.insert(def.name.clone(), (*id, def.clone()));
        }
    }

    fn update_single(&mut self, id: AssetId<ChipDefinition>, asset: &ChipDefinition) {
        self.evolutions
            .insert(asset.name.clone(), (id, asset.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_no_entries() {
        let registry = ChipCatalog::default();
        assert!(registry.get("anything").is_none());
        assert_eq!(registry.ordered_values().count(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut registry = ChipCatalog::default();
        registry.insert(ChipDefinition::test_simple("Piercing Shot"));
        assert!(registry.get("Piercing Shot").is_some());
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn ordered_values_preserves_insertion_order() {
        let mut registry = ChipCatalog::default();
        registry.insert(ChipDefinition::test_simple("C"));
        registry.insert(ChipDefinition::test_simple("A"));
        registry.insert(ChipDefinition::test_simple("B"));
        let names: Vec<&str> = registry.ordered_values().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["C", "A", "B"]);
    }

    // --- ChipCatalog recipe tests (B12d behaviors 1-7) ---

    use crate::{
        chips::{definition::EvolutionIngredient, inventory::ChipInventory},
        effect::definition::{Effect, EffectNode},
    };

    fn ingredient(name: &str, stacks: u32) -> EvolutionIngredient {
        EvolutionIngredient {
            chip_name: name.to_owned(),
            stacks_required: stacks,
        }
    }

    // --- Behavior 1: ChipCatalog has a `recipes` field that starts empty ---

    #[test]
    fn default_registry_recipes_empty() {
        let registry = ChipCatalog::default();
        assert_eq!(
            registry.recipes().len(),
            0,
            "default registry should have no recipes"
        );
    }

    #[test]
    fn default_registry_both_chips_and_recipes_empty() {
        let registry = ChipCatalog::default();
        assert!(registry.get("anything").is_none());
        assert_eq!(registry.recipes().len(), 0);
    }

    // --- Behavior 2: insert_recipe adds a Recipe ---

    #[test]
    fn insert_recipe_adds_recipe() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![EvolutionIngredient {
                chip_name: "Piercing Shot".to_owned(),
                stacks_required: 2,
            }],
            result_name: "Piercing Barrage".to_owned(),
        });
        assert_eq!(registry.recipes().len(), 1);
        assert_eq!(registry.recipes()[0].result_name, "Piercing Barrage");
        assert_eq!(
            registry.recipes()[0].ingredients[0].chip_name,
            "Piercing Shot"
        );
        assert_eq!(registry.recipes()[0].ingredients[0].stacks_required, 2);
    }

    #[test]
    fn insert_recipe_duplicate_result_name_stores_both() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("A", 1)],
            result_name: "Dup".to_owned(),
        });
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("B", 1)],
            result_name: "Dup".to_owned(),
        });
        assert_eq!(
            registry.recipes().len(),
            2,
            "inserting two recipes with the same result_name should store both"
        );
    }

    // --- Behavior 3: eligible_recipes returns matching recipes when all ingredients met ---

    #[test]
    fn eligible_recipes_returns_recipe_when_all_ingredients_met() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Piercing Shot", 2), ingredient("Damage Up", 1)],
            result_name: "Piercing Barrage".to_owned(),
        });

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let du_def = ChipDefinition::test("Damage Up", EffectNode::Do(Effect::DamageBoost(0.5)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Damage Up", &du_def);

        let eligible = registry.eligible_recipes(&inventory);
        assert_eq!(eligible.len(), 1, "should find one eligible recipe");
    }

    #[test]
    fn eligible_recipes_exact_threshold_still_eligible() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Piercing Shot", 2)],
            result_name: "Barrage".to_owned(),
        });

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def); // exactly 2

        let eligible = registry.eligible_recipes(&inventory);
        assert_eq!(
            eligible.len(),
            1,
            "exactly meeting threshold should still be eligible"
        );
    }

    // --- Behavior 4: eligible_recipes returns empty when one ingredient insufficient ---

    #[test]
    fn eligible_recipes_empty_when_one_ingredient_missing() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Piercing Shot", 2), ingredient("Damage Up", 1)],
            result_name: "Piercing Barrage".to_owned(),
        });

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        // No Damage Up added

        let eligible = registry.eligible_recipes(&inventory);
        assert!(
            eligible.is_empty(),
            "should not be eligible without all ingredients"
        );
    }

    #[test]
    fn eligible_recipes_empty_when_all_ingredients_zero() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Piercing Shot", 2), ingredient("Damage Up", 1)],
            result_name: "Barrage".to_owned(),
        });

        let inventory = ChipInventory::default(); // nothing held
        let eligible = registry.eligible_recipes(&inventory);
        assert!(
            eligible.is_empty(),
            "should not be eligible with zero stacks of all ingredients"
        );
    }

    // --- Behavior 5: eligible_recipes returns empty when ingredient stacks insufficient ---

    #[test]
    fn eligible_recipes_empty_when_ingredient_stacks_insufficient() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Piercing Shot", 2)],
            result_name: "Barrage".to_owned(),
        });

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def); // only 1 stack, need 2

        let eligible = registry.eligible_recipes(&inventory);
        assert!(
            eligible.is_empty(),
            "should not be eligible with insufficient stacks"
        );
    }

    #[test]
    fn eligible_recipes_empty_when_ingredient_not_present_at_all() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Missing Chip", 1)],
            result_name: "Barrage".to_owned(),
        });

        let inventory = ChipInventory::default(); // nothing held
        let eligible = registry.eligible_recipes(&inventory);
        assert!(
            eligible.is_empty(),
            "ingredient not present at all should be treated as 0 stacks"
        );
    }

    // --- Behavior 6: eligible_recipes returns only eligible among multiple recipes ---

    #[test]
    fn eligible_recipes_returns_only_eligible_among_multiple() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Piercing Shot", 2)],
            result_name: "Recipe A".to_owned(),
        });
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Wide Breaker", 1)],
            result_name: "Recipe B".to_owned(),
        });

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        // No Wide Breaker

        let eligible = registry.eligible_recipes(&inventory);
        assert_eq!(eligible.len(), 1, "only Recipe A should be eligible");
        assert_eq!(eligible[0].result_name, "Recipe A");
    }

    // --- Behavior 7: eligible_recipes returns multiple when both satisfied ---

    #[test]
    fn eligible_recipes_returns_multiple_when_both_satisfied() {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Piercing Shot", 1)],
            result_name: "Recipe A".to_owned(),
        });
        registry.insert_recipe(Recipe {
            ingredients: vec![ingredient("Damage Up", 1)],
            result_name: "Recipe B".to_owned(),
        });

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let du_def = ChipDefinition::test("Damage Up", EffectNode::Do(Effect::DamageBoost(0.5)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Damage Up", &du_def);

        let eligible = registry.eligible_recipes(&inventory);
        assert_eq!(eligible.len(), 2, "both recipes should be eligible");
    }

    #[test]
    fn eligible_recipes_empty_for_default_registry_with_any_inventory() {
        let registry = ChipCatalog::default(); // no recipes
        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);

        let eligible = registry.eligible_recipes(&inventory);
        assert!(
            eligible.is_empty(),
            "default ChipCatalog with no recipes should return empty Vec"
        );
    }

    // =========================================================================
    // ChipTemplateRegistry — SeedableRegistry tests
    // =========================================================================

    use crate::{
        chips::definition::{ChipTemplate, RaritySlot},
        effect::definition::{RootEffect, Target},
    };

    fn make_chip_template(name: &str, max_taken: u32, prefix: &str) -> ChipTemplate {
        ChipTemplate {
            name: name.to_owned(),
            max_taken,
            common: Some(RaritySlot {
                prefix: prefix.to_owned(),
                effects: vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::Do(Effect::Piercing(1))],
                }],
            }),
            uncommon: None,
            rare: None,
            legendary: None,
        }
    }

    /// Creates `AssetId` values by adding assets to an `Assets<ChipTemplate>` store.
    fn template_asset_pairs(
        templates: Vec<ChipTemplate>,
    ) -> Vec<(AssetId<ChipTemplate>, ChipTemplate)> {
        let mut assets = Assets::<ChipTemplate>::default();
        templates
            .into_iter()
            .map(|t| {
                let handle = assets.add(t.clone());
                (handle.id(), t)
            })
            .collect()
    }

    // ── Behavior 1: seed() populates from 2 templates ───────────────

    #[test]
    fn chip_template_registry_seed_populates_from_templates() {
        let pairs = template_asset_pairs(vec![
            make_chip_template("Piercing", 3, "Basic"),
            make_chip_template("Surge", 2, "Quick"),
        ]);

        let mut registry = ChipTemplateRegistry::default();
        registry.seed(&pairs);

        assert_eq!(
            registry.len(),
            2,
            "registry should contain 2 templates after seed"
        );
        assert!(
            registry.get("Piercing").is_some(),
            "registry should contain 'Piercing'"
        );
        assert!(
            registry.get("Surge").is_some(),
            "registry should contain 'Surge'"
        );
    }

    // ── Behavior 2: seed() clears existing entries ──────────────────

    #[test]
    fn chip_template_registry_seed_clears_existing() {
        let old_pairs = template_asset_pairs(vec![make_chip_template("Old", 1, "Stale")]);
        let new_pairs = template_asset_pairs(vec![make_chip_template("New", 2, "Fresh")]);

        let mut registry = ChipTemplateRegistry::default();
        registry.seed(&old_pairs);
        assert_eq!(registry.len(), 1);
        assert!(registry.get("Old").is_some());

        registry.seed(&new_pairs);

        assert_eq!(registry.len(), 1, "after re-seed, only 'New' should remain");
        assert!(
            registry.get("New").is_some(),
            "'New' should be present after re-seed"
        );
        assert!(
            registry.get("Old").is_none(),
            "'Old' should be gone after re-seed"
        );
    }

    // ── Behavior 3: update_single() upserts by name ────────────────

    #[test]
    fn chip_template_registry_update_single_upserts_by_name() {
        let pairs = template_asset_pairs(vec![make_chip_template("Piercing", 3, "Basic")]);

        let mut registry = ChipTemplateRegistry::default();
        registry.seed(&pairs);

        let (original_id, _) = &pairs[0];
        let updated = make_chip_template("Piercing", 5, "Basic");
        registry.update_single(*original_id, &updated);

        let (_, template) = registry.get("Piercing").expect("'Piercing' should exist");
        assert_eq!(
            template.max_taken, 5,
            "max_taken should be updated to 5 after update_single"
        );
    }

    // ── Behavior 4: update_single() inserts new ────────────────────

    #[test]
    fn chip_template_registry_update_single_inserts_new() {
        let pairs = template_asset_pairs(vec![make_chip_template("Piercing", 3, "Basic")]);
        let new_pairs = template_asset_pairs(vec![make_chip_template("Surge", 2, "Quick")]);

        let mut registry = ChipTemplateRegistry::default();
        registry.seed(&pairs);
        assert_eq!(registry.len(), 1);

        let (new_id, _) = &new_pairs[0];
        let new_template = make_chip_template("Surge", 2, "Quick");
        registry.update_single(*new_id, &new_template);

        assert_eq!(
            registry.len(),
            2,
            "registry should contain 2 templates after inserting new"
        );
        assert!(
            registry.get("Surge").is_some(),
            "'Surge' should be present after update_single"
        );
    }

    // ── Behavior 5: asset_dir() returns correct path ────────────────

    #[test]
    fn chip_template_registry_asset_dir() {
        assert_eq!(
            ChipTemplateRegistry::asset_dir(),
            "chips/templates",
            "asset_dir() should return \"chips/templates\""
        );
    }

    // ── Behavior 6: extensions() returns correct extension ──────────

    #[test]
    fn chip_template_registry_extensions() {
        assert_eq!(
            ChipTemplateRegistry::extensions(),
            &["chip.ron"],
            "extensions() should return [\"chip.ron\"]"
        );
    }

    // =========================================================================
    // EvolutionRegistry — SeedableRegistry tests
    // =========================================================================

    use crate::chips::definition::Rarity;

    fn make_evolution_def(name: &str) -> ChipDefinition {
        ChipDefinition {
            name: name.to_owned(),
            description: String::new(),
            rarity: Rarity::Evolution,
            max_stacks: 1,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::Piercing(5))],
            }],
            ingredients: Some(vec![EvolutionIngredient {
                chip_name: "Piercing Shot".to_owned(),
                stacks_required: 2,
            }]),
            template_name: None,
        }
    }

    /// Creates `AssetId` values by adding assets to an `Assets<ChipDefinition>` store.
    fn evolution_asset_pairs(
        defs: Vec<ChipDefinition>,
    ) -> Vec<(AssetId<ChipDefinition>, ChipDefinition)> {
        let mut assets = Assets::<ChipDefinition>::default();
        defs.into_iter()
            .map(|d| {
                let handle = assets.add(d.clone());
                (handle.id(), d)
            })
            .collect()
    }

    // ── Behavior 1: seed() populates from 2 evolution defs ──────────

    #[test]
    fn evolution_registry_seed_populates_from_definitions() {
        let pairs = evolution_asset_pairs(vec![
            make_evolution_def("Barrage"),
            make_evolution_def("Supernova"),
        ]);

        let mut registry = EvolutionRegistry::default();
        registry.seed(&pairs);

        assert_eq!(
            registry.len(),
            2,
            "registry should contain 2 evolution definitions"
        );
        assert!(
            registry.get("Barrage").is_some(),
            "'Barrage' should be present"
        );
        assert!(
            registry.get("Supernova").is_some(),
            "'Supernova' should be present"
        );
    }

    // ── Behavior 2: seed() clears existing ──────────────────────────

    #[test]
    fn evolution_registry_seed_clears_existing() {
        let old_pairs = evolution_asset_pairs(vec![make_evolution_def("Old")]);
        let new_pairs = evolution_asset_pairs(vec![make_evolution_def("New")]);

        let mut registry = EvolutionRegistry::default();
        registry.seed(&old_pairs);
        assert_eq!(registry.len(), 1);
        assert!(registry.get("Old").is_some());

        registry.seed(&new_pairs);

        assert_eq!(registry.len(), 1, "after re-seed, only 'New' should remain");
        assert!(
            registry.get("New").is_some(),
            "'New' should be present after re-seed"
        );
        assert!(
            registry.get("Old").is_none(),
            "'Old' should be gone after re-seed"
        );
    }

    // ── Behavior 3: update_single() upserts ─────────────────────────

    #[test]
    fn evolution_registry_update_single_upserts() {
        let pairs = evolution_asset_pairs(vec![make_evolution_def("Barrage")]);

        let mut registry = EvolutionRegistry::default();
        registry.seed(&pairs);

        let (original_id, _) = &pairs[0];
        let mut updated = make_evolution_def("Barrage");
        updated.max_stacks = 3;
        registry.update_single(*original_id, &updated);

        let (_, def) = registry.get("Barrage").expect("'Barrage' should exist");
        assert_eq!(
            def.max_stacks, 3,
            "max_stacks should be updated to 3 after update_single"
        );
    }

    // ── Behavior 4: asset_dir() returns correct path ────────────────

    #[test]
    fn evolution_registry_asset_dir() {
        assert_eq!(
            EvolutionRegistry::asset_dir(),
            "chips/evolution",
            "asset_dir() should return \"chips/evolution\""
        );
    }

    // ── Behavior 5: extensions() returns correct extension ──────────

    #[test]
    fn evolution_registry_extensions() {
        assert_eq!(
            EvolutionRegistry::extensions(),
            &["evolution.ron"],
            "extensions() should return [\"evolution.ron\"]"
        );
    }
}
