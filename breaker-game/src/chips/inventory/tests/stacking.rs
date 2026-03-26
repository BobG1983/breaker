use super::helpers::*;
use crate::chips::inventory::ChipInventory;

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
