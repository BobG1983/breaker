//! Chip registry — `HashMap` pool of all loaded chip definitions.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::ChipDefinition;

/// `HashMap` pool of all loaded chip definitions, keyed by name.
///
/// Preserves insertion order via a separate `Vec<String>` for deterministic
/// iteration (chip offer display). Populated during loading by `seed_chip_registry`.
#[derive(Resource, Debug, Default)]
pub(crate) struct ChipRegistry {
    chips: HashMap<String, ChipDefinition>,
    order: Vec<String>,
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
}

/// Registry of all chip evolution recipes.
///
/// Populated at load time. Provides lookups for which evolutions are eligible
/// given the player's current [`ChipInventory`].
#[derive(Resource, Debug, Default)]
pub(crate) struct EvolutionRegistry {
    recipes: Vec<super::definition::EvolutionRecipe>,
}

impl EvolutionRegistry {
    /// Add a recipe to the registry.
    pub(crate) fn insert(&mut self, recipe: super::definition::EvolutionRecipe) {
        self.recipes.push(recipe);
    }

    /// Returns a slice of all recipes.
    #[must_use]
    pub(crate) fn recipes(&self) -> &[super::definition::EvolutionRecipe] {
        &self.recipes
    }

    /// Returns recipes whose ingredients are all satisfied by the given inventory.
    #[must_use]
    pub(crate) fn eligible_evolutions(
        &self,
        inventory: &super::inventory::ChipInventory,
    ) -> Vec<&super::definition::EvolutionRecipe> {
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

    // --- EvolutionRegistry tests ---

    use crate::chips::{
        definition::{EvolutionIngredient, EvolutionRecipe, Rarity, TriggerChain},
        inventory::ChipInventory,
    };

    /// Helper: create a recipe requiring the given ingredients and producing a
    /// chip with the given name.
    fn test_recipe(name: &str, ingredients: Vec<EvolutionIngredient>) -> EvolutionRecipe {
        EvolutionRecipe {
            ingredients,
            result_definition: ChipDefinition {
                name: name.to_owned(),
                description: format!("{name} description"),
                rarity: Rarity::Legendary,
                max_stacks: 1,
                effects: vec![TriggerChain::Piercing(5)],
                ingredients: None,
                template_name: None,
            },
        }
    }

    fn ingredient(name: &str, stacks: u32) -> EvolutionIngredient {
        EvolutionIngredient {
            chip_name: name.to_owned(),
            stacks_required: stacks,
        }
    }

    #[test]
    fn default_evolution_registry_is_empty() {
        let registry = EvolutionRegistry::default();
        let inventory = ChipInventory::default();
        let eligible = registry.eligible_evolutions(&inventory);
        assert!(
            eligible.is_empty(),
            "empty registry should have no eligible evolutions"
        );
    }

    #[test]
    fn evolution_registry_insert_adds_recipe() {
        let mut registry = EvolutionRegistry::default();
        let recipe = test_recipe("Piercing Barrage", vec![ingredient("Piercing Shot", 2)]);
        registry.insert(recipe);
        assert_eq!(registry.recipes().len(), 1);
        assert_eq!(
            registry.recipes()[0].result_definition.name,
            "Piercing Barrage"
        );
    }

    #[test]
    fn eligible_evolutions_returns_recipe_when_all_ingredients_met() {
        let mut registry = EvolutionRegistry::default();
        registry.insert(test_recipe(
            "Piercing Barrage",
            vec![ingredient("Piercing Shot", 2), ingredient("Damage Up", 1)],
        ));

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", TriggerChain::Piercing(1), 5);
        let du_def = ChipDefinition::test("Damage Up", TriggerChain::DamageBoost(0.5), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Damage Up", &du_def);

        let eligible = registry.eligible_evolutions(&inventory);
        assert_eq!(eligible.len(), 1, "should find one eligible recipe");
    }

    #[test]
    fn eligible_evolutions_empty_when_one_ingredient_missing() {
        let mut registry = EvolutionRegistry::default();
        registry.insert(test_recipe(
            "Piercing Barrage",
            vec![ingredient("Piercing Shot", 2), ingredient("Damage Up", 1)],
        ));

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", TriggerChain::Piercing(1), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        // No Damage Up added

        let eligible = registry.eligible_evolutions(&inventory);
        assert!(
            eligible.is_empty(),
            "should not be eligible without all ingredients"
        );
    }

    #[test]
    fn eligible_evolutions_empty_when_ingredient_stacks_insufficient() {
        let mut registry = EvolutionRegistry::default();
        registry.insert(test_recipe(
            "Piercing Barrage",
            vec![ingredient("Piercing Shot", 2)],
        ));

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", TriggerChain::Piercing(1), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def); // only 1 stack, need 2

        let eligible = registry.eligible_evolutions(&inventory);
        assert!(
            eligible.is_empty(),
            "should not be eligible with insufficient stacks"
        );
    }

    #[test]
    fn eligible_evolutions_with_multiple_recipes_returns_only_eligible() {
        let mut registry = EvolutionRegistry::default();
        registry.insert(test_recipe(
            "Recipe A",
            vec![ingredient("Piercing Shot", 2)],
        ));
        registry.insert(test_recipe("Recipe B", vec![ingredient("Wide Breaker", 1)]));

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", TriggerChain::Piercing(1), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        // No Wide Breaker

        let eligible = registry.eligible_evolutions(&inventory);
        assert_eq!(eligible.len(), 1, "only Recipe A should be eligible");
        assert_eq!(eligible[0].result_definition.name, "Recipe A");
    }

    #[test]
    fn eligible_evolutions_returns_multiple_eligible_recipes() {
        let mut registry = EvolutionRegistry::default();
        registry.insert(test_recipe(
            "Recipe A",
            vec![ingredient("Piercing Shot", 1)],
        ));
        registry.insert(test_recipe("Recipe B", vec![ingredient("Damage Up", 1)]));

        let mut inventory = ChipInventory::default();
        let ps_def = ChipDefinition::test("Piercing Shot", TriggerChain::Piercing(1), 5);
        let du_def = ChipDefinition::test("Damage Up", TriggerChain::DamageBoost(0.5), 5);
        let _ = inventory.add_chip("Piercing Shot", &ps_def);
        let _ = inventory.add_chip("Damage Up", &du_def);

        let eligible = registry.eligible_evolutions(&inventory);
        assert_eq!(eligible.len(), 2, "both recipes should be eligible");
    }
}
