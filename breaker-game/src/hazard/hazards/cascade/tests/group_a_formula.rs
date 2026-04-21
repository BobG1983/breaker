use super::super::system::*;

// ════════════════════════════════════════════════════════════════════════════
// Group A — heal_per_neighbour formula (unchanged pure unit tests)
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn heal_zero_stacks_is_zero() {
    let cfg = CascadeConfig {
        base_heal:      1.0,
        per_level_heal: 0.5,
    };
    assert!((cfg.heal_per_neighbour(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn heal_zero_stacks_is_zero_even_with_large_config() {
    let cfg = CascadeConfig {
        base_heal:      999.0,
        per_level_heal: 999.0,
    };
    assert!((cfg.heal_per_neighbour(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_one_equals_base() {
    let cfg = CascadeConfig {
        base_heal:      10.0,
        per_level_heal: 5.0,
    };
    assert!((cfg.heal_per_neighbour(1) - 10.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_one_with_zero_base_is_zero() {
    let cfg = CascadeConfig {
        base_heal:      0.0,
        per_level_heal: 5.0,
    };
    assert!((cfg.heal_per_neighbour(1) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_three_adds_two_levels() {
    let cfg = CascadeConfig {
        base_heal:      10.0,
        per_level_heal: 5.0,
    };
    // base 10.0 + 5.0 * 2 = 20.0.
    assert!((cfg.heal_per_neighbour(3) - 20.0).abs() < f32::EPSILON);
}

#[test]
fn heal_stack_five_matches_design_doc_value() {
    let cfg = CascadeConfig {
        base_heal:      10.0,
        per_level_heal: 5.0,
    };
    // base 10.0 + 5.0 * 4 = 30.0 (pinned by the design doc table).
    assert!((cfg.heal_per_neighbour(5) - 30.0).abs() < f32::EPSILON);
}
