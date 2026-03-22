//! Chip offering algorithm — selects chips to present at each node.

use std::collections::HashMap;

use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
};

use crate::chips::{
    definition::{ChipDefinition, Rarity},
    inventory::ChipInventory,
    resources::ChipRegistry,
};

/// Configuration for the offering algorithm.
#[derive(Debug, Clone)]
pub(crate) struct OfferingConfig {
    /// Base weight per rarity tier.
    pub rarity_weights: HashMap<Rarity, f32>,
    /// Number of chips to offer per node.
    pub offers_per_node: usize,
}

/// Computes effective weight: base * decay.
#[must_use]
pub(crate) fn compute_weight(base_weight: f32, decay: f32) -> f32 {
    base_weight * decay
}

/// Builds the active pool: all chips minus maxed, with effective weights.
#[must_use]
pub(crate) fn build_active_pool(
    registry: &ChipRegistry,
    inventory: &ChipInventory,
    config: &OfferingConfig,
) -> Vec<(String, f32)> {
    let mut pool = Vec::new();
    for chip in registry.ordered_values() {
        if inventory.is_maxed(&chip.name) {
            continue;
        }
        let base_weight = config
            .rarity_weights
            .get(&chip.rarity)
            .copied()
            .unwrap_or(0.0);
        let decay = inventory.weight_decay(&chip.name);
        let effective_weight = compute_weight(base_weight, decay);
        pool.push((chip.name.clone(), effective_weight));
    }
    pool
}

/// Draws `count` chip names from the weighted pool without replacement.
#[must_use]
pub(crate) fn draw_offerings(
    pool: &[(String, f32)],
    count: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    if pool.is_empty() {
        return Vec::new();
    }

    let draws = count.min(pool.len());
    let mut remaining: Vec<(String, f32)> = pool.to_vec();
    let mut results = Vec::with_capacity(draws);

    for _ in 0..draws {
        let weights: Vec<f32> = remaining.iter().map(|(_, w)| *w).collect();
        let Ok(dist) = WeightedIndex::new(&weights) else {
            break;
        };
        let idx = dist.sample(rng);
        results.push(remaining[idx].0.clone());
        remaining.swap_remove(idx);
    }

    results
}

/// Top-level: generates chip offerings for the current node.
#[must_use]
pub(crate) fn generate_offerings(
    registry: &ChipRegistry,
    inventory: &ChipInventory,
    config: &OfferingConfig,
    rng: &mut impl Rng,
) -> Vec<ChipDefinition> {
    let pool = build_active_pool(registry, inventory, config);
    let names = draw_offerings(&pool, config.offers_per_node, rng);
    names
        .iter()
        .filter_map(|name| registry.get(name).cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{
        chips::definition::{AmpEffect, ChipEffect},
        shared::GameRng,
    };

    fn test_config() -> OfferingConfig {
        let mut rarity_weights = HashMap::new();
        rarity_weights.insert(Rarity::Common, 100.0);
        rarity_weights.insert(Rarity::Uncommon, 50.0);
        rarity_weights.insert(Rarity::Rare, 15.0);
        rarity_weights.insert(Rarity::Legendary, 3.0);
        OfferingConfig {
            rarity_weights,
            offers_per_node: 3,
        }
    }

    fn test_chip(name: &str, max_stacks: u32) -> ChipDefinition {
        ChipDefinition::test(name, ChipEffect::Amp(AmpEffect::Piercing(1)), max_stacks)
    }

    fn test_chip_rarity(name: &str, rarity: Rarity, max_stacks: u32) -> ChipDefinition {
        ChipDefinition {
            rarity,
            ..ChipDefinition::test(name, ChipEffect::Amp(AmpEffect::Piercing(1)), max_stacks)
        }
    }

    // --- Behavior 1: Empty registry produces empty offerings ---

    #[test]
    fn empty_registry_returns_empty_offerings() {
        let registry = ChipRegistry::default();
        let inventory = ChipInventory::default();
        let config = test_config();
        let mut rng = GameRng::from_seed(42);
        let result = generate_offerings(&registry, &inventory, &config, &mut rng.0);
        assert!(result.is_empty(), "expected empty, got {result:?}");
    }

    // --- Behavior 2: Pool smaller than count returns all ---

    #[test]
    fn pool_smaller_than_count_returns_all() {
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
        let a = test_chip("A", 1);
        let b = test_chip("B", 1);
        let c = test_chip("C", 1);
        registry.insert(a.clone());
        registry.insert(b.clone());
        registry.insert(c.clone());

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

    // --- Behavior 5: Higher weight more frequent (statistical) ---

    #[test]
    fn higher_weight_chip_appears_more_frequently() {
        let mut registry = ChipRegistry::default();
        registry.insert(test_chip_rarity("CommonChip", Rarity::Common, 99));
        registry.insert(test_chip_rarity("LegendaryChip", Rarity::Legendary, 99));

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
            common_count > 900,
            "CommonChip (weight=100) should appear >900/1000 times, got {common_count}"
        );
    }

    // --- Behavior 6: Weight decay reduces frequency (statistical) ---

    #[test]
    fn weight_decay_reduces_stale_chip_frequency() {
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
        for i in 0..10 {
            let rarity = match i % 4 {
                0 => Rarity::Common,
                1 => Rarity::Uncommon,
                2 => Rarity::Rare,
                _ => Rarity::Legendary,
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

    // --- Behavior 10: Single chip offered alone ---

    #[test]
    fn single_chip_offered_alone() {
        let mut registry = ChipRegistry::default();
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
        let mut registry = ChipRegistry::default();
        let a = test_chip("A", 1);
        let b = test_chip("B", 1);
        let c = test_chip("C", 1);
        registry.insert(a.clone());
        registry.insert(b.clone());
        registry.insert(c.clone());

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
        let mut registry = ChipRegistry::default();
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
            !pool.iter().any(|(name, _)| name == "A"),
            "maxed chip A should be excluded from pool"
        );

        // B is Rare (base=15.0), decay=0.8 => effective=12.0
        let b_entry = pool.iter().find(|(name, _)| name == "B");
        assert!(b_entry.is_some(), "B should be in pool");
        let b_weight = b_entry.unwrap().1;
        assert!(
            (b_weight - 12.0).abs() < f32::EPSILON,
            "B weight: expected 12.0 (15.0*0.8), got {b_weight}"
        );

        // C is Legendary (base=3.0), no decay => effective=3.0
        let c_entry = pool.iter().find(|(name, _)| name == "C");
        assert!(c_entry.is_some(), "C should be in pool");
        let c_weight = c_entry.unwrap().1;
        assert!(
            (c_weight - 3.0).abs() < f32::EPSILON,
            "C weight: expected 3.0 (3.0*1.0), got {c_weight}"
        );
    }

    // --- Behavior 13: draw_offerings without replacement ---

    #[test]
    fn draw_offerings_returns_all_distinct_names() {
        let pool = vec![
            ("A".to_owned(), 100.0),
            ("B".to_owned(), 100.0),
            ("C".to_owned(), 100.0),
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
}
