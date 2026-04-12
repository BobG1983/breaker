//! Behaviors 10-12 + B12d: Single chip, all maxed, `build_active_pool`, evolution exclusion.

use std::collections::HashSet;

use super::{super::system::*, helpers::*};
use crate::{
    chips::{definition::Rarity, inventory::ChipInventory, resources::ChipCatalog},
    shared::GameRng,
};

// --- Behavior 10: Single chip offered alone ---

#[test]
fn single_chip_offered_alone() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip("Only", 3));

    let inventory = ChipInventory::default();
    let config = test_config();
    let mut rng = GameRng::from_seed(42);

    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
    assert_eq!(result.len(), 1, "single chip pool should return 1");
    assert_eq!(result[0].name, "Only");
}

// --- Behavior 11: All maxed except one ---

#[test]
fn all_maxed_except_one_returns_only_remaining() {
    let mut registry = ChipCatalog::default();
    let a = test_chip("A", 1);
    let b = test_chip("B", 1);
    let c = test_chip("C", 1);
    registry.insert(a.clone());
    registry.insert(b.clone());
    registry.insert(c);

    let mut inventory = ChipInventory::default();
    let _ = inventory.add_chip("A", &a);
    let _ = inventory.add_chip("B", &b);

    let config = test_config();
    let mut rng = GameRng::from_seed(42);

    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
    assert_eq!(result.len(), 1, "only C should remain");
    assert_eq!(result[0].name, "C");
}

// --- Behavior 12: build_active_pool applies weights and decay ---

#[test]
fn build_active_pool_applies_weights_and_decay() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_rarity("A", Rarity::Common, 1)); // maxed
    registry.insert(test_chip_rarity("B", Rarity::Rare, 3)); // not maxed, decay 0.8
    registry.insert(test_chip_rarity("C", Rarity::Legendary, 3)); // not maxed, no decay

    let mut inventory = ChipInventory::default();
    let a_def = test_chip_rarity("A", Rarity::Common, 1);
    let _ = inventory.add_chip("A", &a_def); // maxed at 1/1
    inventory.record_offered("B", 0.8);

    let config = test_config();

    let pool = build_active_pool(&registry, &inventory, &config);

    // A is maxed, should not be in pool
    assert!(
        !pool.iter().any(|e| e.name == "A"),
        "maxed chip A should be excluded from pool"
    );

    // B is Rare (base=15.0), decay=0.8 => effective=12.0
    let b_entry = pool.iter().find(|e| e.name == "B");
    assert!(b_entry.is_some(), "B should be in pool");
    let b_weight = b_entry.unwrap().weight;
    assert!(
        (b_weight - 12.0).abs() < f32::EPSILON,
        "B weight: expected 12.0 (15.0*0.8), got {b_weight}"
    );

    // C is Legendary (base=3.0), no decay => effective=3.0
    let c_entry = pool.iter().find(|e| e.name == "C");
    assert!(c_entry.is_some(), "C should be in pool");
    let c_weight = c_entry.unwrap().weight;
    assert!(
        (c_weight - 3.0).abs() < f32::EPSILON,
        "C weight: expected 3.0 (3.0*1.0), got {c_weight}"
    );
}

// --- Behavior 13: draw_offerings without replacement ---

#[test]
fn draw_offerings_returns_all_distinct_names() {
    let pool = vec![
        PoolEntry {
            name:          "A".to_owned(),
            weight:        100.0,
            template_name: None,
        },
        PoolEntry {
            name:          "B".to_owned(),
            weight:        100.0,
            template_name: None,
        },
        PoolEntry {
            name:          "C".to_owned(),
            weight:        100.0,
            template_name: None,
        },
    ];
    let mut rng = GameRng::from_seed(42);
    let result = draw_offerings(&pool, 3, &mut rng.0);
    assert_eq!(result.len(), 3, "should draw exactly 3");
    let names: HashSet<&str> = result.iter().map(String::as_str).collect();
    assert_eq!(names.len(), 3, "all 3 should be distinct");
    assert!(names.contains("A"));
    assert!(names.contains("B"));
    assert!(names.contains("C"));
}

// --- B12d Behavior 12: build_active_pool excludes Evolution-rarity chips ---

#[test]
fn build_active_pool_excludes_evolution_rarity_chips() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_rarity("Normal Chip", Rarity::Common, 3));
    registry.insert(test_chip_rarity("Barrage", Rarity::Evolution, 1));

    let inventory = ChipInventory::default();
    let config = test_config();

    let pool = build_active_pool(&registry, &inventory, &config);

    let pool_names: Vec<&str> = pool.iter().map(|e| e.name.as_str()).collect();
    assert!(
        pool_names.contains(&"Normal Chip"),
        "Common chip should be in pool"
    );
    assert!(
        !pool_names.contains(&"Barrage"),
        "Evolution-rarity chip 'Barrage' should NOT be in pool"
    );
}

#[test]
fn build_active_pool_only_evolution_chips_produces_empty_pool() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_rarity("Evo A", Rarity::Evolution, 1));
    registry.insert(test_chip_rarity("Evo B", Rarity::Evolution, 1));

    let inventory = ChipInventory::default();
    let config = test_config();

    let pool = build_active_pool(&registry, &inventory, &config);
    assert!(
        pool.is_empty(),
        "pool should be empty when only Evolution chips in registry"
    );
}
