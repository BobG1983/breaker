use crate::chips::inventory::ChipInventory;

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
