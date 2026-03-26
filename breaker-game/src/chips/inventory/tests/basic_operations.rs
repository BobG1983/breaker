use std::collections::{HashMap, HashSet};

use super::helpers::*;
use crate::chips::{
    definition::Rarity,
    inventory::{ChipEntry, ChipInventory},
};

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
