//! Chip registry — `HashMap` pool of all loaded chip definitions.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::{ChipDefinition, EvolutionIngredient};

/// A recipe combining ingredient chips into an evolved chip.
/// Stores only the result name — the full `ChipDefinition` is in `ChipRegistry.chips`.
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
pub(crate) struct ChipRegistry {
    chips: HashMap<String, ChipDefinition>,
    order: Vec<String>,
    recipes: Vec<Recipe>,
}

impl ChipRegistry {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_no_entries() {
        let registry = ChipRegistry::default();
        assert!(registry.get("anything").is_none());
        assert_eq!(registry.ordered_values().count(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut registry = ChipRegistry::default();
        registry.insert(ChipDefinition::test_simple("Piercing Shot"));
        assert!(registry.get("Piercing Shot").is_some());
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn ordered_values_preserves_insertion_order() {
        let mut registry = ChipRegistry::default();
        registry.insert(ChipDefinition::test_simple("C"));
        registry.insert(ChipDefinition::test_simple("A"));
        registry.insert(ChipDefinition::test_simple("B"));
        let names: Vec<&str> = registry.ordered_values().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["C", "A", "B"]);
    }

    // --- ChipRegistry recipe tests (B12d behaviors 1-7) ---

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

    // --- Behavior 1: ChipRegistry has a `recipes` field that starts empty ---

    #[test]
    fn default_registry_recipes_empty() {
        let registry = ChipRegistry::default();
        assert_eq!(
            registry.recipes().len(),
            0,
            "default registry should have no recipes"
        );
    }

    #[test]
    fn default_registry_both_chips_and_recipes_empty() {
        let registry = ChipRegistry::default();
        assert!(registry.get("anything").is_none());
        assert_eq!(registry.recipes().len(), 0);
    }

    // --- Behavior 2: insert_recipe adds a Recipe ---

    #[test]
    fn insert_recipe_adds_recipe() {
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let registry = ChipRegistry::default(); // no recipes
        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", EffectNode::Do(Effect::Piercing(1)), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);

        let eligible = registry.eligible_recipes(&inventory);
        assert!(
            eligible.is_empty(),
            "default ChipRegistry with no recipes should return empty Vec"
        );
    }
}
