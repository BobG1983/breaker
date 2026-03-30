//! Behaviors 23-26, 32: Template-aware offering pool and deduplication.

use std::collections::HashSet;

use super::{super::system::*, helpers::*};
use crate::{
    chips::{inventory::ChipInventory, resources::ChipCatalog},
    shared::GameRng,
};

// --- Behavior 23: build_active_pool excludes chips whose template is maxed ---

#[test]
fn build_active_pool_excludes_template_maxed_chips() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_template("Basic Piercing", Some("Piercing"), 3));
    registry.insert(test_chip_template("Keen Piercing", Some("Piercing"), 3));
    registry.insert(test_chip_template("Damage Up", None, 5));

    let mut inventory = ChipInventory::default();
    // Fill template "Piercing" to max
    let basic_def = test_chip_template("Basic Piercing", Some("Piercing"), 3);
    let keen_def = test_chip_template("Keen Piercing", Some("Piercing"), 3);
    let _ = inventory.add_chip("Basic Piercing", &basic_def);
    let _ = inventory.add_chip("Keen Piercing", &keen_def);
    let _ = inventory.add_chip("Keen Piercing", &keen_def);
    // template_taken "Piercing" = 3, template maxed

    // Damage Up at 1 stack (not maxed)
    let damage_def = test_chip_template("Damage Up", None, 5);
    let _ = inventory.add_chip("Damage Up", &damage_def);

    let config = test_config();
    let pool = build_active_pool(&registry, &inventory, &config);

    // Pool should only contain Damage Up
    let pool_names: Vec<&str> = pool.iter().map(|(name, _)| name.as_str()).collect();
    assert!(
        !pool_names.contains(&"Basic Piercing"),
        "template-maxed Basic Piercing should be excluded"
    );
    assert!(
        !pool_names.contains(&"Keen Piercing"),
        "template-maxed Keen Piercing should be excluded"
    );
    assert!(
        pool_names.contains(&"Damage Up"),
        "non-maxed Damage Up should be in pool"
    );
}

#[test]
fn build_active_pool_excludes_none_template_only_when_individually_maxed() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_template("Solo", None, 1));

    let mut inventory = ChipInventory::default();
    let solo_def = test_chip_template("Solo", None, 1);
    let _ = inventory.add_chip("Solo", &solo_def); // 1/1 maxed

    let config = test_config();
    let pool = build_active_pool(&registry, &inventory, &config);

    assert!(
        !pool.iter().any(|(name, _)| name == "Solo"),
        "individually maxed None-template chip should be excluded"
    );
}

// --- Behavior 24: No duplicate template_name in a single offering ---

#[test]
fn generate_offerings_no_duplicate_template_in_offering() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_template("Basic Piercing", Some("Piercing"), 3));
    registry.insert(test_chip_template("Keen Piercing", Some("Piercing"), 3));
    registry.insert(test_chip_template("Basic Damage", Some("Damage"), 3));
    registry.insert(test_chip_template("Keen Damage", Some("Damage"), 3));
    registry.insert(test_chip_template("Standalone", None, 3));

    let inventory = ChipInventory::default();
    let mut config = test_config();
    config.offers_per_node = 3;

    let mut rng = GameRng::from_seed(42);
    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);

    // Check no two results share the same template_name
    let template_names: Vec<Option<&str>> =
        result.iter().map(|d| d.template_name.as_deref()).collect();
    for (i, tname) in template_names.iter().enumerate() {
        if let Some(name) = tname {
            for (j, other) in template_names.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        Some(*name),
                        *other,
                        "two offerings share template '{name}': {} and {}",
                        result[i].name,
                        result[j].name
                    );
                }
            }
        }
    }
}

#[test]
fn generate_offerings_none_template_chips_never_deduplicated() {
    // Chips with template_name: None should never be considered duplicates
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_template("Solo A", None, 3));
    registry.insert(test_chip_template("Solo B", None, 3));
    registry.insert(test_chip_template("Solo C", None, 3));

    let inventory = ChipInventory::default();
    let mut config = test_config();
    config.offers_per_node = 3;

    let mut rng = GameRng::from_seed(42);
    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);

    assert_eq!(
        result.len(),
        3,
        "all 3 None-template chips should be offered"
    );
}

// --- Behavior 25: Template deduplication returns correct count ---

#[test]
fn generate_offerings_correct_count_with_unique_templates() {
    let mut registry = ChipCatalog::default();
    for i in 0..5 {
        registry.insert(test_chip_template(
            &format!("Chip{i}"),
            Some(&format!("Tmpl{i}")),
            3,
        ));
    }

    let inventory = ChipInventory::default();
    let mut config = test_config();
    config.offers_per_node = 3;

    let mut rng = GameRng::from_seed(42);
    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);

    assert_eq!(
        result.len(),
        3,
        "should return exactly 3 offerings from 5 unique templates"
    );
    // All from different templates
    let template_names: HashSet<&str> = result
        .iter()
        .filter_map(|d| d.template_name.as_deref())
        .collect();
    assert_eq!(
        template_names.len(),
        3,
        "all 3 offerings should be from different templates"
    );
}

// --- Behavior 26: Fewer unique templates than slots ---

#[test]
fn generate_offerings_fewer_templates_than_slots() {
    let mut registry = ChipCatalog::default();
    // 4 chips across 2 templates
    registry.insert(test_chip_template("Basic Piercing", Some("Piercing"), 3));
    registry.insert(test_chip_template("Keen Piercing", Some("Piercing"), 3));
    registry.insert(test_chip_template("Basic Damage", Some("Damage"), 3));
    registry.insert(test_chip_template("Keen Damage", Some("Damage"), 3));

    let inventory = ChipInventory::default();
    let mut config = test_config();
    config.offers_per_node = 3;

    let mut rng = GameRng::from_seed(42);
    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);

    assert_eq!(
        result.len(),
        2,
        "only 2 unique templates exist, so at most 2 offerings"
    );
}

// --- Behavior 32: Template deduplication allows all None-template chips ---

#[test]
fn generate_offerings_all_none_template_chips_offered() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_template("A", None, 3));
    registry.insert(test_chip_template("B", None, 3));
    registry.insert(test_chip_template("C", None, 3));

    let inventory = ChipInventory::default();
    let mut config = test_config();
    config.offers_per_node = 3;

    let mut rng = GameRng::from_seed(42);
    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);

    assert_eq!(
        result.len(),
        3,
        "3 None-template chips should all be offered, no deduplication"
    );
    let names: HashSet<&str> = result.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains("A"));
    assert!(names.contains("B"));
    assert!(names.contains("C"));
}
