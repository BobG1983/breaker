//! Chip inventory — tracks the player's chip build during a run.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::chips::definition::{ChipDefinition, Rarity};

/// A single entry in the chip inventory, tracking stacks and metadata.
#[derive(Debug, Clone)]
pub struct ChipEntry {
    /// Current number of stacks held.
    pub stacks: u32,
    /// Maximum stacks allowed for this chip.
    pub max_stacks: u32,
    /// Rarity of this chip.
    pub rarity: Rarity,
    /// Template this chip belongs to, if any.
    pub template_name: Option<String>,
}

/// Tracks chips the player has acquired during a run.
///
/// `held` maps chip names to their [`ChipEntry`] (stacks, max, rarity).
/// `decay_weights` tracks accumulated offering decay per chip name.
/// `template_taken` tracks how many chips have been taken per template name.
/// `template_maxes` stores the maximum allowed per template name.
#[derive(Resource, Debug, Default)]
pub struct ChipInventory {
    held: HashMap<String, ChipEntry>,
    decay_weights: HashMap<String, f32>,
    /// Current count of chips taken per template name.
    template_taken: HashMap<String, u32>,
    /// Maximum allowed per template name.
    template_maxes: HashMap<String, u32>,
}

impl ChipInventory {
    /// Attempt to add one stack of the named chip.
    ///
    /// Returns `true` if the chip was added, `false` if already at max stacks
    /// or the template cap has been reached.
    #[must_use]
    pub fn add_chip(&mut self, name: &str, def: &ChipDefinition) -> bool {
        // Check template-level cap first
        if let Some(tname) = &def.template_name {
            let taken = self
                .template_taken
                .get(tname.as_str())
                .copied()
                .unwrap_or(0);
            if taken >= def.max_stacks {
                return false;
            }
        }

        // Check individual cap
        if let Some(entry) = self.held.get_mut(name) {
            if entry.stacks >= entry.max_stacks {
                return false;
            }
            entry.stacks += 1;
        } else {
            self.held.insert(
                name.to_owned(),
                ChipEntry {
                    stacks: 1,
                    max_stacks: def.max_stacks,
                    rarity: def.rarity,
                    template_name: def.template_name.clone(),
                },
            );
        }

        // Update template tracking on successful add
        if let Some(tname) = &def.template_name {
            *self.template_taken.entry(tname.clone()).or_insert(0) += 1;
            self.template_maxes
                .entry(tname.clone())
                .or_insert(def.max_stacks);
        }

        true
    }

    /// Returns the current stack count for the named chip, or 0 if not held.
    #[must_use]
    pub fn stacks(&self, name: &str) -> u32 {
        self.held.get(name).map_or(0, |entry| entry.stacks)
    }

    /// Returns `true` if the named chip is at its maximum stack count.
    #[must_use]
    pub fn is_maxed(&self, name: &str) -> bool {
        self.held
            .get(name)
            .is_some_and(|entry| entry.stacks >= entry.max_stacks)
    }

    /// Record that the player has seen this chip in an offer screen.
    pub fn mark_seen(&mut self, name: &str) {
        self.record_offered(name, 0.8);
    }

    /// Returns `true` if the player has seen the named chip.
    #[must_use]
    pub fn has_seen(&self, name: &str) -> bool {
        self.decay_weights.contains_key(name)
    }

    /// Iterate all held chips as `(name, entry)` pairs.
    pub fn held_chips(&self) -> impl Iterator<Item = (&str, &ChipEntry)> {
        self.held.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate held chips as `(name, stacks, max_stacks)` tuples.
    ///
    /// Exposes stack counts without revealing [`ChipEntry`] internals. Used
    /// by the scenario runner's [`ChipStacksConsistent`] invariant checker.
    pub fn iter_held_stacks(&self) -> impl Iterator<Item = (&str, u32, u32)> {
        self.held
            .iter()
            .map(|(k, v)| (k.as_str(), v.stacks, v.max_stacks))
    }

    /// Iterate names of all chips currently at max stacks.
    pub fn maxed_chips(&self) -> impl Iterator<Item = &str> {
        self.held
            .iter()
            .filter(|(_, entry)| entry.stacks >= entry.max_stacks)
            .map(|(name, _)| name.as_str())
    }

    /// Returns the number of distinct chips held (not total stacks).
    #[must_use]
    pub fn total_held(&self) -> usize {
        self.held.len()
    }

    /// Remove one stack of the named chip.
    ///
    /// Returns `true` if a stack was removed, `false` if the chip is not held.
    /// If this reduces the stack count to 0, the entry is removed entirely.
    #[must_use]
    pub fn remove_chip(&mut self, name: &str) -> bool {
        let Some(entry) = self.held.get_mut(name) else {
            return false;
        };
        let template_name = entry.template_name.clone();
        entry.stacks -= 1;
        if entry.stacks == 0 {
            self.held.remove(name);
        }

        // Decrement template tracking
        if let Some(tname) = template_name
            && let Some(taken) = self.template_taken.get_mut(&tname)
        {
            *taken = taken.saturating_sub(1);
            if *taken == 0 {
                self.template_taken.remove(&tname);
            }
        }

        true
    }

    /// Remove all held chips and seen history.
    pub fn clear(&mut self) {
        self.held.clear();
        self.decay_weights.clear();
        self.template_taken.clear();
        self.template_maxes.clear();
    }

    /// Directly insert a chip entry with arbitrary `stacks` and `max_stacks`,
    /// bypassing normal cap enforcement.
    ///
    /// **For scenario-runner self-tests only.** This is the only sanctioned way
    /// to construct an over-stacked entry (where `stacks > max_stacks`) for
    /// testing [`InvariantKind::ChipStacksConsistent`] violation detection.
    ///
    /// Never call this from game logic — `add_chip` enforces the stack cap.
    // NOTE: does not update template_taken or template_maxes — test-only
    pub fn force_insert_entry(
        &mut self,
        name: &str,
        stacks: u32,
        max_stacks: u32,
        template_name: Option<&str>,
    ) {
        self.held.insert(
            name.to_owned(),
            ChipEntry {
                stacks,
                max_stacks,
                rarity: Rarity::Common,
                template_name: template_name.map(str::to_owned),
            },
        );
    }

    /// Returns `true` if the template's taken count has reached its max.
    #[must_use]
    pub fn is_template_maxed(&self, template_name: &str) -> bool {
        let taken = self.template_taken.get(template_name).copied().unwrap_or(0);
        let max = self
            .template_maxes
            .get(template_name)
            .copied()
            .unwrap_or(u32::MAX);
        taken >= max
    }

    /// Returns the current taken count for a template, or 0 if unknown.
    #[must_use]
    pub fn template_taken(&self, template_name: &str) -> u32 {
        self.template_taken.get(template_name).copied().unwrap_or(0)
    }

    /// Returns `true` if this chip can still be added — checks both individual
    /// and template-level caps.
    #[must_use]
    pub fn is_chip_available(&self, def: &ChipDefinition) -> bool {
        // Check template-level cap
        if let Some(tname) = &def.template_name {
            let taken = self
                .template_taken
                .get(tname.as_str())
                .copied()
                .unwrap_or(0);
            if taken >= def.max_stacks {
                return false;
            }
        }
        // Check individual cap
        !self.is_maxed(&def.name)
    }

    /// Record that a chip was offered, multiplying existing decay by the factor.
    pub fn record_offered(&mut self, name: &str, decay_factor: f32) {
        if let Some(existing) = self.decay_weights.get_mut(name) {
            *existing *= decay_factor;
        } else {
            self.decay_weights.insert(name.to_owned(), decay_factor);
        }
    }

    /// Returns the accumulated decay weight for a chip (1.0 if never offered).
    #[must_use]
    pub fn weight_decay(&self, name: &str) -> f32 {
        self.decay_weights.get(name).copied().unwrap_or(1.0)
    }
}
