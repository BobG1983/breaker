//! Chip inventory — tracks the player's chip build during a run.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::{ChipDefinition, Rarity};

/// A single entry in the chip inventory, tracking stacks and metadata.
#[derive(Debug, Clone)]
pub(crate) struct ChipEntry {
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{chips::definition::TriggerChain, effect::definition::Target};

    /// Helper: create a Piercing Shot definition with `max_stacks=3`, Common rarity.
    fn piercing_shot_def() -> ChipDefinition {
        ChipDefinition::test("Piercing Shot", TriggerChain::Piercing(1), 3)
    }

    /// Helper: create a Wide Breaker definition with `max_stacks=3`, Rare rarity.
    fn wide_breaker_def() -> ChipDefinition {
        ChipDefinition {
            rarity: Rarity::Rare,
            ..ChipDefinition::test(
                "Wide Breaker",
                TriggerChain::SizeBoost(Target::Breaker, 20.0),
                3,
            )
        }
    }

    /// Helper: create a Damage Up definition with `max_stacks=2`, Common rarity.
    fn damage_up_def() -> ChipDefinition {
        ChipDefinition::test("Damage Up", TriggerChain::DamageBoost(0.5), 2)
    }

    /// Helper: create a chip definition with `max_stacks=1`, Common rarity.
    fn single_stack_def() -> ChipDefinition {
        ChipDefinition::test("Single Stack", TriggerChain::Piercing(1), 1)
    }

    // --- Behavior 1: Default inventory is empty ---

    #[test]
    fn default_inventory_total_held_is_zero() {
        let inv = ChipInventory::default();
        assert_eq!(inv.total_held(), 0);
    }

    #[test]
    fn default_inventory_held_chips_is_empty() {
        let inv = ChipInventory::default();
        assert_eq!(inv.held_chips().count(), 0);
    }

    #[test]
    fn default_inventory_maxed_chips_is_empty() {
        let inv = ChipInventory::default();
        assert_eq!(inv.maxed_chips().count(), 0);
    }

    #[test]
    fn default_inventory_stacks_returns_zero_for_unknown() {
        let inv = ChipInventory::default();
        assert_eq!(inv.stacks("anything"), 0);
    }

    #[test]
    fn default_inventory_is_maxed_returns_false_for_unknown() {
        let inv = ChipInventory::default();
        assert!(!inv.is_maxed("anything"));
    }

    #[test]
    fn default_inventory_has_seen_returns_false_for_unknown() {
        let inv = ChipInventory::default();
        assert!(!inv.has_seen("anything"));
    }

    // --- Behavior 2: Adding a chip creates a new entry at 1 stack ---

    #[test]
    fn add_chip_returns_true_on_first_add() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        assert!(inv.add_chip("Piercing Shot", &def));
    }

    #[test]
    fn add_chip_sets_stacks_to_one() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        let _ = inv.add_chip("Piercing Shot", &def);
        assert_eq!(inv.stacks("Piercing Shot"), 1);
    }

    #[test]
    fn add_chip_increments_total_held() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        let _ = inv.add_chip("Piercing Shot", &def);
        assert_eq!(inv.total_held(), 1);
    }

    #[test]
    fn add_chip_is_not_maxed_when_below_max() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        assert!(!inv.is_maxed("Piercing Shot")); // 1 < 3
    }

    // --- Behavior 3: Adding the same chip again increments stacks ---

    #[test]
    fn add_chip_twice_returns_true_both_times() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        assert!(inv.add_chip("Piercing Shot", &def));
        assert!(inv.add_chip("Piercing Shot", &def));
    }

    #[test]
    fn add_chip_twice_increments_stacks_to_two() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        assert_eq!(inv.stacks("Piercing Shot"), 2);
    }

    #[test]
    fn add_chip_twice_total_held_still_one() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        assert_eq!(inv.total_held(), 1);
    }

    #[test]
    fn add_chip_twice_not_maxed_when_below_max() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        assert!(!inv.is_maxed("Piercing Shot")); // 2 < 3
    }

    // --- Behavior 4: Adding beyond max_stacks is rejected ---

    #[test]
    fn add_chip_beyond_max_stacks_returns_false() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        assert!(!inv.add_chip("Piercing Shot", &def)); // 4th add rejected
    }

    #[test]
    fn add_chip_beyond_max_stacks_keeps_stacks_at_max() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def); // rejected
        assert_eq!(inv.stacks("Piercing Shot"), 3);
    }

    #[test]
    fn add_chip_beyond_max_stacks_total_held_unchanged() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def); // rejected
        assert_eq!(inv.total_held(), 1);
    }

    #[test]
    fn single_stack_chip_second_add_returns_false() {
        let mut inv = ChipInventory::default();
        let def = single_stack_def(); // max_stacks=1
        assert!(inv.add_chip("Single Stack", &def));
        assert!(!inv.add_chip("Single Stack", &def));
    }

    #[test]
    fn single_stack_chip_stacks_stay_at_one() {
        let mut inv = ChipInventory::default();
        let def = single_stack_def(); // max_stacks=1
        let _ = inv.add_chip("Single Stack", &def);
        let _ = inv.add_chip("Single Stack", &def); // rejected
        assert_eq!(inv.stacks("Single Stack"), 1);
    }

    // --- Behavior 5: is_maxed returns true when stacks equals max_stacks ---

    #[test]
    fn is_maxed_true_when_stacks_equals_max() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        assert!(inv.is_maxed("Piercing Shot")); // 3 == 3
    }

    #[test]
    fn is_maxed_true_for_single_stack_chip_at_one() {
        let mut inv = ChipInventory::default();
        let def = single_stack_def(); // max_stacks=1
        let _ = inv.add_chip("Single Stack", &def);
        assert!(inv.is_maxed("Single Stack")); // 1 == 1
    }

    // --- Behavior 6: mark_seen and has_seen tracking ---

    #[test]
    fn mark_seen_makes_has_seen_return_true() {
        let mut inv = ChipInventory::default();
        inv.mark_seen("Piercing Shot");
        assert!(inv.has_seen("Piercing Shot"));
    }

    #[test]
    fn has_seen_returns_false_for_unseen_chip() {
        let mut inv = ChipInventory::default();
        inv.mark_seen("Piercing Shot");
        assert!(!inv.has_seen("Wide Breaker"));
    }

    #[test]
    fn mark_seen_is_idempotent() {
        let mut inv = ChipInventory::default();
        inv.mark_seen("Piercing Shot");
        inv.mark_seen("Piercing Shot");
        assert!(inv.has_seen("Piercing Shot"));
    }

    // --- Behavior 7: mark_seen is independent of held chips ---

    #[test]
    fn add_chip_does_not_mark_seen() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        let _ = inv.add_chip("Piercing Shot", &def);
        assert!(!inv.has_seen("Piercing Shot"));
    }

    #[test]
    fn mark_seen_does_not_create_held_entry() {
        let mut inv = ChipInventory::default();
        inv.mark_seen("Wide Breaker");
        assert_eq!(inv.stacks("Wide Breaker"), 0);
        assert!(inv.has_seen("Wide Breaker"));
    }

    #[test]
    fn mark_seen_and_add_chip_both_tracked_independently() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        inv.mark_seen("Piercing Shot");
        let _ = inv.add_chip("Piercing Shot", &def);
        assert!(inv.has_seen("Piercing Shot"));
        assert_eq!(inv.stacks("Piercing Shot"), 1);
    }

    // --- Behavior 8: clear resets held and seen ---

    #[test]
    fn clear_resets_total_held_to_zero() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        inv.clear();
        assert_eq!(inv.total_held(), 0);
    }

    #[test]
    fn clear_resets_stacks_to_zero() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        inv.clear();
        assert_eq!(inv.stacks("Piercing Shot"), 0);
    }

    #[test]
    fn clear_resets_has_seen() {
        let mut inv = ChipInventory::default();
        inv.mark_seen("Wide Breaker");
        inv.clear();
        assert!(!inv.has_seen("Wide Breaker"));
    }

    #[test]
    fn clear_resets_held_chips_iterator() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        let _ = inv.add_chip("Piercing Shot", &def);
        inv.clear();
        assert_eq!(inv.held_chips().count(), 0);
    }

    #[test]
    fn clear_resets_maxed_chips_iterator() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def();
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        inv.clear();
        assert_eq!(inv.maxed_chips().count(), 0);
    }

    #[test]
    fn clear_on_empty_inventory_does_not_panic() {
        let mut inv = ChipInventory::default();
        inv.clear(); // should not panic
        assert_eq!(inv.total_held(), 0);
    }

    // --- Behavior 9: maxed_chips yields only maxed chip names ---

    #[test]
    fn maxed_chips_contains_only_maxed_entries() {
        let mut inv = ChipInventory::default();
        let piercing = piercing_shot_def(); // max_stacks=3
        let wide = wide_breaker_def(); // max_stacks=3
        let damage = damage_up_def(); // max_stacks=2

        // Piercing Shot at 3/3 — maxed
        let _ = inv.add_chip("Piercing Shot", &piercing);
        let _ = inv.add_chip("Piercing Shot", &piercing);
        let _ = inv.add_chip("Piercing Shot", &piercing);

        // Wide Breaker at 1/3 — not maxed
        let _ = inv.add_chip("Wide Breaker", &wide);

        // Damage Up at 2/2 — maxed
        let _ = inv.add_chip("Damage Up", &damage);
        let _ = inv.add_chip("Damage Up", &damage);

        let maxed: HashSet<&str> = inv.maxed_chips().collect();
        assert_eq!(maxed.len(), 2);
        assert!(maxed.contains("Piercing Shot"));
        assert!(maxed.contains("Damage Up"));
        assert!(!maxed.contains("Wide Breaker"));
    }

    #[test]
    fn maxed_chips_empty_when_none_maxed() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3
        let _ = inv.add_chip("Piercing Shot", &def); // 1/3 — not maxed
        assert_eq!(inv.maxed_chips().count(), 0);
    }

    // --- Behavior 10: held_chips iterates all entries with correct data ---

    #[test]
    fn held_chips_returns_all_entries_with_correct_data() {
        let mut inv = ChipInventory::default();
        let piercing = piercing_shot_def(); // max_stacks=3, Common
        let wide = wide_breaker_def(); // max_stacks=3, Rare

        let _ = inv.add_chip("Piercing Shot", &piercing);
        let _ = inv.add_chip("Piercing Shot", &piercing);
        let _ = inv.add_chip("Wide Breaker", &wide);

        let entries: HashMap<&str, &ChipEntry> = inv.held_chips().collect();
        assert_eq!(entries.len(), 2);

        let ps = entries
            .get("Piercing Shot")
            .expect("should have Piercing Shot");
        assert_eq!(ps.stacks, 2);
        assert_eq!(ps.max_stacks, 3);
        assert_eq!(ps.rarity, Rarity::Common);

        let wb = entries
            .get("Wide Breaker")
            .expect("should have Wide Breaker");
        assert_eq!(wb.stacks, 1);
        assert_eq!(wb.max_stacks, 3);
        assert_eq!(wb.rarity, Rarity::Rare);
    }

    #[test]
    fn held_chips_empty_on_default_inventory() {
        let inv = ChipInventory::default();
        assert_eq!(inv.held_chips().count(), 0);
    }

    // --- Behavior: Decay weight tracking ---

    #[test]
    fn record_offered_initializes_decay_weight() {
        let mut inv = ChipInventory::default();
        inv.record_offered("Piercing Shot", 0.8);
        let weight = inv.weight_decay("Piercing Shot");
        assert!(
            (weight - 0.8).abs() < f32::EPSILON,
            "expected 0.8, got {weight}"
        );
    }

    #[test]
    fn record_offered_does_not_affect_other_chips() {
        let mut inv = ChipInventory::default();
        inv.record_offered("Piercing Shot", 0.8);
        let weight = inv.weight_decay("Other");
        assert!(
            (weight - 1.0).abs() < f32::EPSILON,
            "expected 1.0 for unknown chip, got {weight}"
        );
    }

    #[test]
    fn record_offered_compounds_existing_decay() {
        let mut inv = ChipInventory::default();
        inv.record_offered("PS", 0.8);
        inv.record_offered("PS", 0.8);
        let weight = inv.weight_decay("PS");
        assert!(
            (weight - 0.64).abs() < f32::EPSILON,
            "expected 0.64 (0.8*0.8), got {weight}"
        );
    }

    #[test]
    fn record_offered_compounds_three_times() {
        let mut inv = ChipInventory::default();
        inv.record_offered("PS", 0.8);
        inv.record_offered("PS", 0.8);
        inv.record_offered("PS", 0.8);
        let weight = inv.weight_decay("PS");
        assert!(
            (weight - 0.512).abs() < 1e-6,
            "expected 0.512 (0.8^3), got {weight}"
        );
    }

    #[test]
    fn record_offered_with_different_factors() {
        let mut inv = ChipInventory::default();
        inv.record_offered("X", 0.5);
        inv.record_offered("X", 0.8);
        let weight = inv.weight_decay("X");
        assert!(
            (weight - 0.4).abs() < f32::EPSILON,
            "expected 0.4 (0.5*0.8), got {weight}"
        );
    }

    #[test]
    fn weight_decay_returns_one_for_unknown_chip() {
        let inv = ChipInventory::default();
        let weight = inv.weight_decay("anything");
        assert!(
            (weight - 1.0).abs() < f32::EPSILON,
            "expected 1.0 for unknown chip, got {weight}"
        );
    }

    #[test]
    fn has_seen_returns_true_after_record_offered() {
        let mut inv = ChipInventory::default();
        inv.record_offered("PS", 0.8);
        assert!(
            inv.has_seen("PS"),
            "has_seen should return true after record_offered"
        );
    }

    #[test]
    fn mark_seen_uses_0_8_decay_factor() {
        let mut inv = ChipInventory::default();
        inv.mark_seen("PS");
        let weight = inv.weight_decay("PS");
        assert!(
            (weight - 0.8).abs() < f32::EPSILON,
            "expected 0.8 after mark_seen, got {weight}"
        );
    }

    #[test]
    fn mark_seen_twice_compounds_decay() {
        let mut inv = ChipInventory::default();
        inv.mark_seen("PS");
        inv.mark_seen("PS");
        let weight = inv.weight_decay("PS");
        assert!(
            (weight - 0.64).abs() < f32::EPSILON,
            "expected 0.64 after two mark_seen calls, got {weight}"
        );
    }

    #[test]
    fn clear_resets_decay_weights() {
        let mut inv = ChipInventory::default();
        inv.record_offered("PS", 0.8);
        inv.clear();
        let weight = inv.weight_decay("PS");
        assert!(
            (weight - 1.0).abs() < f32::EPSILON,
            "expected 1.0 after clear, got {weight}"
        );
    }

    #[test]
    fn clear_resets_has_seen_after_record_offered() {
        let mut inv = ChipInventory::default();
        inv.record_offered("PS", 0.8);
        inv.clear();
        assert!(
            !inv.has_seen("PS"),
            "has_seen should return false after clear"
        );
    }

    #[test]
    fn record_offered_with_1_0_factor_keeps_weight_at_one() {
        let mut inv = ChipInventory::default();
        inv.record_offered("X", 1.0);
        let weight = inv.weight_decay("X");
        assert!(
            (weight - 1.0).abs() < f32::EPSILON,
            "expected 1.0 after 1.0 factor, got {weight}"
        );
    }

    #[test]
    fn record_offered_with_1_0_factor_still_marks_seen() {
        let mut inv = ChipInventory::default();
        inv.record_offered("X", 1.0);
        assert!(
            inv.has_seen("X"),
            "has_seen should return true even with 1.0 factor"
        );
    }

    #[test]
    fn record_offered_with_0_0_factor_zeroes_weight() {
        let mut inv = ChipInventory::default();
        inv.record_offered("X", 0.0);
        let weight = inv.weight_decay("X");
        assert!(
            weight.abs() < f32::EPSILON,
            "expected 0.0 after 0.0 factor, got {weight}"
        );
    }

    #[test]
    fn record_offered_with_0_0_factor_stays_zero_after_further_offering() {
        let mut inv = ChipInventory::default();
        inv.record_offered("X", 0.0);
        inv.record_offered("X", 0.8);
        let weight = inv.weight_decay("X");
        assert!(
            weight.abs() < f32::EPSILON,
            "expected 0.0 (0.0*0.8), got {weight}"
        );
    }

    // --- Behavior: remove_chip decrements stack count ---

    #[test]
    fn remove_chip_decrements_stacks_from_two_to_one() {
        let mut inv = ChipInventory::default();
        let def = piercing_shot_def(); // max_stacks=3, Common
        let _ = inv.add_chip("Piercing Shot", &def);
        let _ = inv.add_chip("Piercing Shot", &def);
        assert_eq!(inv.stacks("Piercing Shot"), 2);

        let removed = inv.remove_chip("Piercing Shot");
        assert!(removed, "remove_chip should return true when chip is held");
        assert_eq!(
            inv.stacks("Piercing Shot"),
            1,
            "stacks should decrease from 2 to 1"
        );
    }

    #[test]
    fn remove_chip_at_one_stack_removes_entry_entirely() {
        let mut inv = ChipInventory::default();
        let def = single_stack_def(); // max_stacks=1
        let _ = inv.add_chip("Single Stack", &def);
        assert_eq!(inv.stacks("Single Stack"), 1);

        let removed = inv.remove_chip("Single Stack");
        assert!(removed, "remove_chip should return true");
        assert_eq!(
            inv.stacks("Single Stack"),
            0,
            "stacks should be 0 after removing last stack"
        );
        assert_eq!(
            inv.total_held(),
            0,
            "total_held should decrease when last stack removed"
        );
        assert!(
            !inv.held_chips().any(|(name, _)| name == "Single Stack"),
            "held_chips should no longer contain removed chip"
        );
        assert!(
            !inv.is_maxed("Single Stack"),
            "is_maxed should return false after removal"
        );
    }

    #[test]
    fn remove_chip_on_chip_not_held_returns_false() {
        let mut inv = ChipInventory::default();
        assert!(
            !inv.remove_chip("Nonexistent"),
            "remove_chip should return false for unheld chip"
        );
        assert_eq!(inv.total_held(), 0, "total_held should remain 0");
    }

    #[test]
    fn remove_chip_does_not_affect_other_held_chips() {
        let mut inv = ChipInventory::default();
        let def_a = piercing_shot_def(); // max_stacks=3
        let def_b = single_stack_def(); // max_stacks=1
        let _ = inv.add_chip("Piercing Shot", &def_a);
        let _ = inv.add_chip("Piercing Shot", &def_a);
        let _ = inv.add_chip("Single Stack", &def_b);

        let _ = inv.remove_chip("Piercing Shot");
        assert_eq!(
            inv.stacks("Piercing Shot"),
            1,
            "Piercing Shot should go from 2 to 1"
        );
        assert_eq!(
            inv.stacks("Single Stack"),
            1,
            "Single Stack should be unchanged"
        );
    }

    #[test]
    fn remove_chip_then_add_chip_re_adds_from_zero() {
        let mut inv = ChipInventory::default();
        let def = single_stack_def(); // max_stacks=1
        let _ = inv.add_chip("Single Stack", &def);
        let _ = inv.remove_chip("Single Stack");
        assert_eq!(inv.stacks("Single Stack"), 0);

        let added = inv.add_chip("Single Stack", &def);
        assert!(added, "add_chip should succeed after removal");
        assert_eq!(
            inv.stacks("Single Stack"),
            1,
            "stacks should be 1 after re-adding"
        );
    }

    // ======================================================================
    // B5: Template-based max_taken tracking (spec behaviors 13-22, 27-31)
    // ======================================================================

    /// Helper: create a chip definition with a `template_name`.
    fn template_chip_def(name: &str, template_name: &str, max_stacks: u32) -> ChipDefinition {
        ChipDefinition {
            template_name: Some(template_name.to_owned()),
            ..ChipDefinition::test(name, TriggerChain::Piercing(1), max_stacks)
        }
    }

    /// Helper: create a chip definition with no template.
    fn standalone_chip_def(name: &str, max_stacks: u32) -> ChipDefinition {
        ChipDefinition::test(name, TriggerChain::Piercing(1), max_stacks)
    }

    // --- Behavior 13: add_chip increments template_taken ---

    #[test]
    fn add_chip_increments_template_taken_for_templated_chip() {
        let mut inv = ChipInventory::default();
        let def = template_chip_def("Basic Piercing", "Piercing", 3);
        let result = inv.add_chip("Basic Piercing", &def);
        assert!(result);
        assert_eq!(inv.template_taken("Piercing"), 1);
    }

    // --- Behavior 14: Different rarity of same template shares counter ---

    #[test]
    fn add_chip_different_rarity_same_template_shares_counter() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);
        let keen = template_chip_def("Keen Piercing", "Piercing", 3);

        let _ = inv.add_chip("Basic Piercing", &basic);
        let _ = inv.add_chip("Keen Piercing", &keen);

        assert_eq!(inv.template_taken("Piercing"), 2);
        assert_eq!(inv.stacks("Basic Piercing"), 1);
        assert_eq!(inv.stacks("Keen Piercing"), 1);
    }

    // --- Behavior 15: add_chip rejects when template is maxed ---

    #[test]
    fn add_chip_rejects_when_template_taken_reaches_max() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);
        let keen = template_chip_def("Keen Piercing", "Piercing", 3);
        let brutal = template_chip_def("Brutal Piercing", "Piercing", 3);

        let _ = inv.add_chip("Basic Piercing", &basic);
        let _ = inv.add_chip("Basic Piercing", &basic);
        let _ = inv.add_chip("Keen Piercing", &keen);
        // template_taken("Piercing") == 3, which equals max_stacks 3

        let result = inv.add_chip("Brutal Piercing", &brutal);
        assert!(!result, "should reject when template is maxed");
        assert_eq!(inv.stacks("Brutal Piercing"), 0);
        assert_eq!(inv.template_taken("Piercing"), 3);
    }

    // --- Behavior 16: Stacking same chip until both individual + template full ---

    #[test]
    fn add_chip_stacks_same_chip_until_template_full() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);

        assert!(inv.add_chip("Basic Piercing", &basic)); // stacks=1, template_taken=1
        assert!(inv.add_chip("Basic Piercing", &basic)); // stacks=2, template_taken=2
        assert!(inv.add_chip("Basic Piercing", &basic)); // stacks=3, template_taken=3

        assert_eq!(inv.stacks("Basic Piercing"), 3);
        assert_eq!(inv.template_taken("Piercing"), 3);

        // Further add should fail
        assert!(!inv.add_chip("Basic Piercing", &basic));
    }

    // --- Behavior 17: add_chip with None template doesn't touch template_taken ---

    #[test]
    fn add_chip_without_template_does_not_touch_template_taken() {
        let mut inv = ChipInventory::default();
        let def = standalone_chip_def("Supernova", 1);

        let result = inv.add_chip("Supernova", &def);
        assert!(result);
        assert_eq!(inv.template_taken("Supernova"), 0);
        assert_eq!(inv.stacks("Supernova"), 1);
    }

    // --- Behavior 18: is_template_maxed ---

    #[test]
    fn is_template_maxed_true_when_taken_equals_max() {
        let mut inv = ChipInventory::default();
        let def = template_chip_def("Basic Piercing", "Piercing", 3);
        let _ = inv.add_chip("Basic Piercing", &def);
        let _ = inv.add_chip("Basic Piercing", &def);
        let _ = inv.add_chip("Basic Piercing", &def);
        assert!(inv.is_template_maxed("Piercing"));
    }

    #[test]
    fn is_template_maxed_false_for_unknown_template() {
        let inv = ChipInventory::default();
        assert!(!inv.is_template_maxed("Unknown"));
    }

    // --- Behavior 19: is_chip_available false when template maxed ---

    #[test]
    fn is_chip_available_false_when_template_maxed_even_if_individual_not_held() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);
        let keen = template_chip_def("Keen Piercing", "Piercing", 3);
        let brutal = template_chip_def("Brutal Piercing", "Piercing", 3);

        let _ = inv.add_chip("Basic Piercing", &basic);
        let _ = inv.add_chip("Keen Piercing", &keen);
        let _ = inv.add_chip("Keen Piercing", &keen);
        // template_taken=3, but Brutal is not held at all

        assert!(
            !inv.is_chip_available(&brutal),
            "Brutal Piercing should be unavailable when template is maxed"
        );
    }

    // --- Behavior 20: remove_chip decrements template_taken ---

    #[test]
    fn remove_chip_decrements_template_taken() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);
        let _ = inv.add_chip("Basic Piercing", &basic);
        let _ = inv.add_chip("Basic Piercing", &basic);
        assert_eq!(inv.template_taken("Piercing"), 2);

        let _ = inv.remove_chip("Basic Piercing");
        assert_eq!(inv.stacks("Basic Piercing"), 1);
        assert_eq!(inv.template_taken("Piercing"), 1);
    }

    // --- Behavior 21: remove last stack zeroes template counter ---

    #[test]
    fn remove_last_stack_zeroes_template_counter() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);
        let _ = inv.add_chip("Basic Piercing", &basic);
        assert_eq!(inv.template_taken("Piercing"), 1);

        let _ = inv.remove_chip("Basic Piercing");
        assert_eq!(inv.stacks("Basic Piercing"), 0);
        assert_eq!(inv.template_taken("Piercing"), 0);
    }

    // --- Behavior 22: clear resets template_taken and template_maxes ---

    #[test]
    fn clear_resets_template_tracking() {
        let mut inv = ChipInventory::default();
        let def = template_chip_def("Basic Piercing", "Piercing", 3);
        let _ = inv.add_chip("Basic Piercing", &def);
        let _ = inv.add_chip("Basic Piercing", &def);
        let _ = inv.add_chip("Basic Piercing", &def);
        assert!(inv.is_template_maxed("Piercing"));

        inv.clear();

        assert_eq!(inv.template_taken("Piercing"), 0);
        assert!(!inv.is_template_maxed("Piercing"));
    }

    // --- Behavior 27: add_chip rejects when template maxed via OTHER chip names ---

    #[test]
    fn add_chip_rejects_when_template_maxed_via_other_chip_names() {
        let mut inv = ChipInventory::default();
        let keen = template_chip_def("Keen Piercing", "Piercing", 3);
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);

        // Fill template via Keen only
        let _ = inv.add_chip("Keen Piercing", &keen);
        let _ = inv.add_chip("Keen Piercing", &keen);
        let _ = inv.add_chip("Keen Piercing", &keen);
        assert_eq!(inv.template_taken("Piercing"), 3);

        // Basic should be rejected even though it has 0 individual stacks
        let result = inv.add_chip("Basic Piercing", &basic);
        assert!(
            !result,
            "Basic should be rejected when template is maxed via Keen"
        );
        assert_eq!(inv.stacks("Basic Piercing"), 0);
        assert_eq!(inv.template_taken("Piercing"), 3);
    }

    // --- Behavior 28: remove_chip decrements template when other variants still held ---

    #[test]
    fn remove_chip_decrements_template_when_other_variants_still_held() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);
        let keen = template_chip_def("Keen Piercing", "Piercing", 3);

        let _ = inv.add_chip("Basic Piercing", &basic);
        let _ = inv.add_chip("Keen Piercing", &keen);
        assert_eq!(inv.template_taken("Piercing"), 2);

        let _ = inv.remove_chip("Basic Piercing");
        assert_eq!(inv.stacks("Basic Piercing"), 0);
        assert_eq!(
            inv.template_taken("Piercing"),
            1,
            "template_taken should be 1 (Keen still held)"
        );
    }

    // --- Behavior 29: is_chip_available false for None-template chip individually maxed ---

    #[test]
    fn is_chip_available_false_for_individually_maxed_standalone() {
        let mut inv = ChipInventory::default();
        let def = standalone_chip_def("Standalone", 1);
        let _ = inv.add_chip("Standalone", &def);

        assert!(
            !inv.is_chip_available(&def),
            "standalone chip at max stacks should be unavailable"
        );
    }

    // --- Behavior 30: is_chip_available true when neither maxed ---

    #[test]
    fn is_chip_available_true_when_neither_individually_nor_template_maxed() {
        let mut inv = ChipInventory::default();
        let basic = template_chip_def("Basic Piercing", "Piercing", 3);
        let _ = inv.add_chip("Basic Piercing", &basic);

        assert!(
            inv.is_chip_available(&basic),
            "chip with 1/3 stacks and template 1/3 should be available"
        );
    }

    // --- Behavior 31: is_chip_available true for unheld chip with non-maxed template ---

    #[test]
    fn is_chip_available_true_for_unheld_chip_with_non_maxed_template() {
        let inv = ChipInventory::default();
        let def = template_chip_def("Basic Piercing", "Piercing", 3);
        assert!(
            inv.is_chip_available(&def),
            "unheld chip with non-maxed template should be available"
        );
    }
}
