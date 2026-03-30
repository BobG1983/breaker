//! Behaviors 1-4: Empty registry, pool size, no duplicates, maxed exclusion.

use std::collections::HashSet;

use super::{super::system::*, helpers::*};
use crate::{
    chips::{inventory::ChipInventory, resources::ChipCatalog},
    shared::GameRng,
};

// --- Behavior 1: Empty registry produces empty offerings ---

#[test]
fn empty_registry_returns_empty_offerings() {
    let registry = ChipCatalog::default();
    let inventory = ChipInventory::default();
    let config = test_config();
    let mut rng = GameRng::from_seed(42);
    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
    assert!(result.is_empty(), "expected empty, got {result:?}");
}

// --- Behavior 2: Pool smaller than count returns all ---

#[test]
fn pool_smaller_than_count_returns_all() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip("A", 3));
    registry.insert(test_chip("B", 3));

    let inventory = ChipInventory::default();
    let mut config = test_config();
    config.offers_per_node = 3;
    let mut rng = GameRng::from_seed(42);

    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
    assert_eq!(
        result.len(),
        2,
        "only 2 chips in pool, should return 2 not 3"
    );
}

// --- Behavior 3: No duplicates in offering ---

#[test]
fn no_duplicates_in_offering() {
    let mut registry = ChipCatalog::default();
    for i in 0..5 {
        registry.insert(test_chip(&format!("Chip{i}"), 3));
    }

    let inventory = ChipInventory::default();
    let config = test_config();
    let mut rng = GameRng::from_seed(42);

    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
    let names: HashSet<&str> = result.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(
        names.len(),
        result.len(),
        "all offered chips should be distinct"
    );
    assert_eq!(result.len(), 3, "should offer exactly 3 chips");
}

// --- Behavior 4: Maxed chips excluded ---

#[test]
fn maxed_chips_excluded_from_offerings() {
    let mut registry = ChipCatalog::default();
    let a = test_chip("A", 1);
    let b = test_chip("B", 1);
    let c = test_chip("C", 1);
    registry.insert(a.clone());
    registry.insert(b);
    registry.insert(c);

    let mut inventory = ChipInventory::default();
    let _ = inventory.add_chip("A", &a);

    let config = test_config();
    let mut rng = GameRng::from_seed(42);

    let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
    let names: HashSet<&str> = result.iter().map(|d| d.name.as_str()).collect();
    assert!(
        !names.contains("A"),
        "maxed chip A should not appear in offerings"
    );
    assert_eq!(result.len(), 2, "should return B and C only");
}
