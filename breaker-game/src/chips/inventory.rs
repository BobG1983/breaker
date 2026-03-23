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
}

/// Tracks chips the player has acquired during a run.
///
/// `held` maps chip names to their [`ChipEntry`] (stacks, max, rarity).
/// `decay_weights` tracks accumulated offering decay per chip name.
#[derive(Resource, Debug, Default)]
pub struct ChipInventory {
    held: HashMap<String, ChipEntry>,
    decay_weights: HashMap<String, f32>,
}

impl ChipInventory {
    /// Attempt to add one stack of the named chip.
    ///
    /// Returns `true` if the chip was added, `false` if already at max stacks.
    #[must_use]
    pub fn add_chip(&mut self, name: &str, def: &ChipDefinition) -> bool {
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
                },
            );
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
        entry.stacks -= 1;
        if entry.stacks == 0 {
            self.held.remove(name);
        }
        true
    }

    /// Remove all held chips and seen history.
    pub fn clear(&mut self) {
        self.held.clear();
        self.decay_weights.clear();
    }

    /// Directly insert a chip entry with arbitrary `stacks` and `max_stacks`,
    /// bypassing normal cap enforcement.
    ///
    /// **For scenario-runner self-tests only.** This is the only sanctioned way
    /// to construct an over-stacked entry (where `stacks > max_stacks`) for
    /// testing [`InvariantKind::ChipStacksConsistent`] violation detection.
    ///
    /// Never call this from game logic — `add_chip` enforces the stack cap.
    pub fn force_insert_entry(&mut self, name: &str, stacks: u32, max_stacks: u32) {
        self.held.insert(
            name.to_owned(),
            ChipEntry {
                stacks,
                max_stacks,
                rarity: Rarity::Common,
            },
        );
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
    use crate::chips::definition::{AmpEffect, ChipEffect};

    /// Helper: create a Piercing Shot definition with `max_stacks=3`, Common rarity.
    fn piercing_shot_def() -> ChipDefinition {
        ChipDefinition::test("Piercing Shot", ChipEffect::Amp(AmpEffect::Piercing(1)), 3)
    }

    /// Helper: create a Wide Breaker definition with `max_stacks=3`, Rare rarity.
    fn wide_breaker_def() -> ChipDefinition {
        ChipDefinition {
            rarity: Rarity::Rare,
            ..ChipDefinition::test(
                "Wide Breaker",
                ChipEffect::Augment(crate::chips::definition::AugmentEffect::WidthBoost(20.0)),
                3,
            )
        }
    }

    /// Helper: create a Damage Up definition with `max_stacks=2`, Common rarity.
    fn damage_up_def() -> ChipDefinition {
        ChipDefinition::test("Damage Up", ChipEffect::Amp(AmpEffect::DamageBoost(0.5)), 2)
    }

    /// Helper: create a chip definition with `max_stacks=1`, Common rarity.
    fn single_stack_def() -> ChipDefinition {
        ChipDefinition::test("Single Stack", ChipEffect::Amp(AmpEffect::Piercing(1)), 1)
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
}
