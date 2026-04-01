//! Chip offering algorithm -- selects chips to present at each node.

use std::collections::HashMap;

use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
};

use crate::chips::{
    definition::{ChipDefinition, Rarity},
    inventory::ChipInventory,
    resources::ChipCatalog,
};

/// A single entry in the weighted offering pool, carrying template metadata
/// for deduplication.
#[derive(Clone, Debug)]
pub(crate) struct PoolEntry {
    /// Chip name.
    pub name: String,
    /// Effective weight for weighted draw.
    pub weight: f32,
    /// Template this chip belongs to, if any.
    pub template_name: Option<String>,
}

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

/// Builds the active pool: all chips minus maxed, with effective weights and
/// template metadata for deduplication.
#[must_use]
pub(crate) fn build_active_pool(
    registry: &ChipCatalog,
    inventory: &ChipInventory,
    config: &OfferingConfig,
) -> Vec<PoolEntry> {
    let mut pool = Vec::new();
    for chip in registry.ordered_values() {
        if chip.rarity == Rarity::Evolution {
            continue;
        }
        if !inventory.is_chip_available(chip) {
            continue;
        }
        let base_weight = config
            .rarity_weights
            .get(&chip.rarity)
            .copied()
            .unwrap_or(0.0);
        let decay = inventory.weight_decay(&chip.name);
        let effective_weight = compute_weight(base_weight, decay);
        pool.push(PoolEntry {
            name: chip.name.clone(),
            weight: effective_weight,
            template_name: chip.template_name.clone(),
        });
    }
    pool
}

/// Top-level: generates chip offerings for the current node.
///
/// Performs template-aware deduplication: after each draw, any remaining pool
/// entries sharing the same `template_name` are removed so no two offerings
/// come from the same template. Chips with `template_name: None` are never
/// deduplicated against each other.
#[must_use]
pub(crate) fn generate_offerings(
    registry: &ChipCatalog,
    inventory: &ChipInventory,
    config: &OfferingConfig,
    rng: &mut impl Rng,
) -> Vec<ChipDefinition> {
    let mut pool = build_active_pool(registry, inventory, config);

    let draws = config.offers_per_node.min(pool.len());
    let mut results = Vec::with_capacity(draws);

    for _ in 0..draws {
        if pool.is_empty() {
            break;
        }
        let weights: Vec<f32> = pool.iter().map(|e| e.weight).collect();
        let Ok(dist) = WeightedIndex::new(&weights) else {
            break;
        };
        let idx = dist.sample(rng);
        let chosen = pool.swap_remove(idx);

        // Remove all other entries sharing the same template_name (if Some)
        if let Some(ref tname) = chosen.template_name {
            pool.retain(|e| e.template_name.as_deref() != Some(tname));
        }

        results.push(chosen.name);
    }

    results
        .iter()
        .filter_map(|name| {
            let def = registry.get(name);
            debug_assert!(def.is_some(), "drawn name must be in registry: {name}");
            def.cloned()
        })
        .collect()
}
