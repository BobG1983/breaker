//! Shared helpers for offering test sub-modules.

use std::collections::HashMap;

use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
};

use super::super::system::*;
use crate::{
    chips::definition::{ChipDefinition, Rarity},
    effect::{EffectKind, EffectNode},
};

/// Lower-level draw primitive -- test-only.
///
/// Draws `count` chip names from the weighted pool without replacement.
#[must_use]
pub(super) fn draw_offerings(pool: &[PoolEntry], count: usize, rng: &mut impl Rng) -> Vec<String> {
    if pool.is_empty() {
        return Vec::new();
    }

    let draws = count.min(pool.len());
    let mut remaining: Vec<PoolEntry> = pool.to_vec();
    let mut results = Vec::with_capacity(draws);

    for _ in 0..draws {
        let weights: Vec<f32> = remaining.iter().map(|e| e.weight).collect();
        let Ok(dist) = WeightedIndex::new(&weights) else {
            break;
        };
        let idx = dist.sample(rng);
        results.push(remaining[idx].name.clone());
        remaining.swap_remove(idx);
    }

    results
}

pub(super) fn test_config() -> OfferingConfig {
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

pub(super) fn test_chip(name: &str, max_stacks: u32) -> ChipDefinition {
    ChipDefinition::test(name, EffectNode::Do(EffectKind::Piercing(1)), max_stacks)
}

pub(super) fn test_chip_rarity(name: &str, rarity: Rarity, max_stacks: u32) -> ChipDefinition {
    ChipDefinition {
        rarity,
        ..ChipDefinition::test(name, EffectNode::Do(EffectKind::Piercing(1)), max_stacks)
    }
}

/// Helper: create a chip def with a `template_name` for offering tests.
pub(super) fn test_chip_template(
    name: &str,
    template_name: Option<&str>,
    max_stacks: u32,
) -> ChipDefinition {
    ChipDefinition {
        template_name: template_name.map(str::to_owned),
        ..ChipDefinition::test(name, EffectNode::Do(EffectKind::Piercing(1)), max_stacks)
    }
}
