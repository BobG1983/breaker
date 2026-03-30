//! `ChipCatalog` basic tests and recipe tests (behaviors 1-8).

use crate::{
    chips::{
        definition::{ChipDefinition, EvolutionIngredient},
        inventory::ChipInventory,
        resources::*,
    },
    effect::{EffectKind, EffectNode},
};

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
    let ps_def = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let du_def = ChipDefinition::test(
        "Minor Damage Up",
        EffectNode::Do(EffectKind::DamageBoost(0.5)),
        5,
    )
    .with_template("Damage Up");
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Minor Damage Up", &du_def);

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
    let ps_def = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def); // exactly 2

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
    let ps_def = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
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
    let ps_def = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def); // only 1 stack, need 2

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
    let ps_def = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
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
    let ps_def = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let du_def = ChipDefinition::test(
        "Minor Damage Up",
        EffectNode::Do(EffectKind::DamageBoost(0.5)),
        5,
    )
    .with_template("Damage Up");
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);
    let _ = inventory.add_chip("Minor Damage Up", &du_def);

    let eligible = registry.eligible_recipes(&inventory);
    assert_eq!(eligible.len(), 2, "both recipes should be eligible");
}

#[test]
fn eligible_recipes_empty_for_default_registry_with_any_inventory() {
    let registry = ChipCatalog::default(); // no recipes
    let mut inventory = ChipInventory::default();
    let ps_def = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let _ = inventory.add_chip("Basic Piercing Shot", &ps_def);

    let eligible = registry.eligible_recipes(&inventory);
    assert!(
        eligible.is_empty(),
        "default ChipCatalog with no recipes should return empty Vec"
    );
}

// --- Behavior 8: eligible_recipes matches by template name across rarity variants ---

#[test]
fn eligible_recipes_matches_mixed_rarity_variants_of_same_template() {
    let mut registry = ChipCatalog::default();
    registry.insert_recipe(Recipe {
        ingredients: vec![ingredient("Piercing Shot", 3)],
        result_name: "Railgun".to_owned(),
    });

    let mut inventory = ChipInventory::default();
    // Add different rarity variants of the same template
    let basic = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let keen = ChipDefinition::test(
        "Keen Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(2)),
        5,
    )
    .with_template("Piercing Shot");
    let _ = inventory.add_chip("Basic Piercing Shot", &basic);
    let _ = inventory.add_chip("Basic Piercing Shot", &basic);
    let _ = inventory.add_chip("Keen Piercing Shot", &keen);
    // 3 total from "Piercing Shot" template (2 Basic + 1 Keen)

    let eligible = registry.eligible_recipes(&inventory);
    assert_eq!(
        eligible.len(),
        1,
        "mixed rarity variants should count toward the same template ingredient"
    );
    assert_eq!(eligible[0].result_name, "Railgun");
}

#[test]
fn eligible_recipes_not_eligible_when_template_taken_below_threshold() {
    let mut registry = ChipCatalog::default();
    registry.insert_recipe(Recipe {
        ingredients: vec![ingredient("Piercing Shot", 3)],
        result_name: "Railgun".to_owned(),
    });

    let mut inventory = ChipInventory::default();
    let basic = ChipDefinition::test(
        "Basic Piercing Shot",
        EffectNode::Do(EffectKind::Piercing(1)),
        5,
    )
    .with_template("Piercing Shot");
    let _ = inventory.add_chip("Basic Piercing Shot", &basic);
    let _ = inventory.add_chip("Basic Piercing Shot", &basic);
    // Only 2 from "Piercing Shot" template, need 3

    let eligible = registry.eligible_recipes(&inventory);
    assert!(
        eligible.is_empty(),
        "2 of 3 required template chips should not satisfy the recipe"
    );
}
