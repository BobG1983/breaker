//! Behaviors 5-9: Weight frequency, decay, `compute_weight`, seed determinism.

use super::{super::system::*, helpers::*};
use crate::{
    chips::{definition::Rarity, inventory::ChipInventory, resources::ChipCatalog},
    prelude::*,
};

// --- Behavior 5: Higher weight more frequent (statistical) ---

#[test]
fn higher_weight_chip_appears_more_frequently() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip_rarity("CommonChip", Rarity::Common, 99));
    registry.insert(test_chip_rarity("RareChip", Rarity::Rare, 99));

    let inventory = ChipInventory::default();
    let mut config = test_config();
    config.offers_per_node = 1;

    let mut common_count = 0_u32;
    for seed in 0..1000_u64 {
        let mut rng = GameRng::from_seed(seed);
        let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
        if result.len() == 1 && result[0].name == "CommonChip" {
            common_count += 1;
        }
    }
    assert!(
        common_count > 800,
        "CommonChip (weight=100) should appear >800/1000 times vs Rare (weight=15), got {common_count}"
    );
}

// --- Behavior 6: Weight decay reduces frequency (statistical) ---

#[test]
fn weight_decay_reduces_stale_chip_frequency() {
    let mut registry = ChipCatalog::default();
    registry.insert(test_chip("Fresh", 99));
    registry.insert(test_chip("Stale", 99));

    let mut inventory = ChipInventory::default();
    inventory.record_offered("Stale", 0.1);

    let mut config = test_config();
    config.offers_per_node = 1;

    let mut fresh_count = 0_u32;
    for seed in 0..1000_u64 {
        let mut rng = GameRng::from_seed(seed);
        let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
        if result.len() == 1 && result[0].name == "Fresh" {
            fresh_count += 1;
        }
    }
    assert!(
        fresh_count > 800,
        "Fresh chip should appear >800/1000 times (Stale has 0.1 decay), got {fresh_count}"
    );
}

// --- Behavior 7: compute_weight multiplies ---

#[test]
fn compute_weight_multiplies_base_and_decay() {
    let result = compute_weight(50.0, 0.5);
    assert!(
        (result - 25.0).abs() < f32::EPSILON,
        "expected 25.0, got {result}"
    );
}

#[test]
fn compute_weight_with_identity_decay() {
    let result = compute_weight(100.0, 1.0);
    assert!(
        (result - 100.0).abs() < f32::EPSILON,
        "expected 100.0, got {result}"
    );
}

#[test]
fn compute_weight_with_zero_base() {
    let result = compute_weight(0.0, 0.5);
    assert!(result.abs() < f32::EPSILON, "expected 0.0, got {result}");
}

// --- Behavior 8: Same seed same offerings ---

#[test]
fn same_seed_produces_same_offerings() {
    let mut registry = ChipCatalog::default();
    for i in 0..5 {
        registry.insert(test_chip(&format!("Chip{i}"), 3));
    }

    let inventory = ChipInventory::default();
    let config = test_config();

    let mut rng1 = GameRng::from_seed(42);
    let result1 = generate_offerings(&registry, &inventory, &config, &mut rng1.0);

    let mut rng2 = GameRng::from_seed(42);
    let result2 = generate_offerings(&registry, &inventory, &config, &mut rng2.0);

    let names1: Vec<&str> = result1.iter().map(|d| d.name.as_str()).collect();
    let names2: Vec<&str> = result2.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names1, names2, "same seed should produce same offerings");
}

// --- Behavior 9: Different seed different offerings ---

#[test]
fn different_seed_produces_different_offerings() {
    let mut registry = ChipCatalog::default();
    for i in 0..10 {
        let rarity = match i % 3 {
            0 => Rarity::Common,
            1 => Rarity::Uncommon,
            _ => Rarity::Rare,
        };
        registry.insert(test_chip_rarity(&format!("Chip{i}"), rarity, 3));
    }

    let inventory = ChipInventory::default();
    let config = test_config();

    let mut rng1 = GameRng::from_seed(42);
    let result1 = generate_offerings(&registry, &inventory, &config, &mut rng1.0);

    let mut rng2 = GameRng::from_seed(999);
    let result2 = generate_offerings(&registry, &inventory, &config, &mut rng2.0);

    let names1: Vec<&str> = result1.iter().map(|d| d.name.as_str()).collect();
    let names2: Vec<&str> = result2.iter().map(|d| d.name.as_str()).collect();
    assert_ne!(
        names1, names2,
        "different seeds should produce different offerings"
    );
}
