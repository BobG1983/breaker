//! Chip inventory — tracks the player's chip build during a run.

use std::collections::{HashMap, HashSet};

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
/// `seen` tracks chip names the player has encountered in offer screens.
#[derive(Resource, Debug, Default)]
pub(crate) struct ChipInventory {
    held: HashMap<String, ChipEntry>,
    seen: HashSet<String>,
}

impl ChipInventory {
    /// Attempt to add one stack of the named chip.
    ///
    /// Returns `true` if the chip was added, `false` if already at max stacks.
    #[must_use]
    pub(crate) fn add_chip(&mut self, name: &str, def: &ChipDefinition) -> bool {
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
    pub(crate) fn stacks(&self, name: &str) -> u32 {
        self.held.get(name).map_or(0, |entry| entry.stacks)
    }

    /// Returns `true` if the named chip is at its maximum stack count.
    #[must_use]
    pub(crate) fn is_maxed(&self, name: &str) -> bool {
        self.held
            .get(name)
            .is_some_and(|entry| entry.stacks >= entry.max_stacks)
    }

    /// Record that the player has seen this chip in an offer screen.
    pub(crate) fn mark_seen(&mut self, name: &str) {
        self.seen.insert(name.to_owned());
    }

    /// Returns `true` if the player has seen the named chip.
    #[must_use]
    pub(crate) fn has_seen(&self, name: &str) -> bool {
        self.seen.contains(name)
    }

    /// Iterate all held chips as `(name, entry)` pairs.
    pub(crate) fn held_chips(&self) -> impl Iterator<Item = (&str, &ChipEntry)> {
        self.held.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterate names of all chips currently at max stacks.
    pub(crate) fn maxed_chips(&self) -> impl Iterator<Item = &str> {
        self.held
            .iter()
            .filter(|(_, entry)| entry.stacks >= entry.max_stacks)
            .map(|(name, _)| name.as_str())
    }

    /// Returns the number of distinct chips held (not total stacks).
    #[must_use]
    pub(crate) fn total_held(&self) -> usize {
        self.held.len()
    }

    /// Remove all held chips and seen history.
    pub(crate) fn clear(&mut self) {
        self.held.clear();
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
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
}
