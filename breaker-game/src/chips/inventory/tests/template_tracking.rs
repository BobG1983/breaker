use super::helpers::*;
use crate::chips::inventory::ChipInventory;

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
