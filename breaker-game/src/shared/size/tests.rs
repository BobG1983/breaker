use super::types::*;

// ── Part A: effective_size pure function tests ───────────────────

// Behavior 1: No boosts, no node scale, no constraints returns base dimensions

#[test]
fn effective_size_no_boosts_no_scale_returns_base_dimensions() {
    let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
        "expected (120.0, 20.0), got ({}, {})",
        result.x,
        result.y,
    );
}

// Behavior 2: Size boost multiplier scales both width and height

#[test]
fn effective_size_boost_scales_both_width_and_height() {
    let result = effective_size(
        120.0,
        20.0,
        4.0_f32 / 3.0,
        1.0,
        ClampRange::NONE,
        ClampRange::NONE,
    );
    assert!(
        (result.x - 160.0).abs() < 1e-3,
        "expected width 160.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 20.0 * 4.0 / 3.0).abs() < 1e-3,
        "expected height ~26.666, got {}",
        result.y,
    );
}

#[test]
fn effective_size_identity_boost_returns_base() {
    let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
        "identity boost should return base, got ({}, {})",
        result.x,
        result.y,
    );
}

// Behavior 3: Node scaling factor scales both width and height

#[test]
fn effective_size_node_scale_scales_both_dimensions() {
    let result = effective_size(120.0, 20.0, 1.0, 0.7, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 84.0).abs() < 1e-3,
        "expected width 84.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 14.0).abs() < 1e-3,
        "expected height 14.0, got {}",
        result.y,
    );
}

#[test]
fn effective_size_identity_node_scale_returns_base() {
    let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
        "identity node scale should return base, got ({}, {})",
        result.x,
        result.y,
    );
}

// Behavior 4: Both boost and node scale multiply together

#[test]
fn effective_size_boost_and_node_scale_multiply() {
    let result = effective_size(
        120.0,
        20.0,
        4.0_f32 / 3.0,
        0.7,
        ClampRange::NONE,
        ClampRange::NONE,
    );
    // 120.0 * 4/3 * 0.7 = 112.0, 20.0 * 4/3 * 0.7 = 18.666...
    assert!(
        (result.x - 112.0).abs() < 1e-3,
        "expected width 112.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 18.666_666).abs() < 1e-2,
        "expected height ~18.666, got {}",
        result.y,
    );
}

#[test]
fn effective_size_large_boost_with_half_scale() {
    let result = effective_size(120.0, 20.0, 3.0, 0.5, ClampRange::NONE, ClampRange::NONE);
    // 120.0 * 3.0 * 0.5 = 180.0, 20.0 * 3.0 * 0.5 = 30.0
    assert!(
        (result.x - 180.0).abs() < 1e-3,
        "expected width 180.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 30.0).abs() < 1e-3,
        "expected height 30.0, got {}",
        result.y,
    );
}

// Behavior 5: Clamps width to max when boost would exceed it

#[test]
fn effective_size_clamps_width_to_max() {
    // 120.0 * 3.0 = 360.0, clamped to max 200.0
    // 20.0 * 3.0 = 60.0, within bounds [10.0, 100.0]
    let result = effective_size(
        120.0,
        20.0,
        3.0,
        1.0,
        ClampRange {
            min: Some(60.0),
            max: Some(200.0),
        },
        ClampRange {
            min: Some(10.0),
            max: Some(100.0),
        },
    );
    assert!(
        (result.x - 200.0).abs() < 1e-3,
        "expected width clamped to 200.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 60.0).abs() < 1e-3,
        "expected height 60.0, got {}",
        result.y,
    );
}

#[test]
fn effective_size_clamps_both_to_max() {
    // 120.0 * 10.0 = 1200.0 -> 200.0, 20.0 * 10.0 = 200.0 -> 100.0
    let result = effective_size(
        120.0,
        20.0,
        10.0,
        1.0,
        ClampRange {
            min: Some(60.0),
            max: Some(200.0),
        },
        ClampRange {
            min: Some(10.0),
            max: Some(100.0),
        },
    );
    assert!(
        (result.x - 200.0).abs() < 1e-3,
        "expected width clamped to 200.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 100.0).abs() < 1e-3,
        "expected height clamped to 100.0, got {}",
        result.y,
    );
}

// Behavior 6: Clamps to min when scale would shrink below it

#[test]
fn effective_size_clamps_to_min_when_scaled_small() {
    // 120.0 * 0.1 = 12.0, clamped to min 60.0
    // 20.0 * 0.1 = 2.0, clamped to min 10.0
    let result = effective_size(
        120.0,
        20.0,
        1.0,
        0.1,
        ClampRange {
            min: Some(60.0),
            max: Some(600.0),
        },
        ClampRange {
            min: Some(10.0),
            max: Some(100.0),
        },
    );
    assert!(
        (result.x - 60.0).abs() < 1e-3,
        "expected width clamped to min 60.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 10.0).abs() < 1e-3,
        "expected height clamped to min 10.0, got {}",
        result.y,
    );
}

#[test]
fn effective_size_exactly_at_min_boundary() {
    // 120.0 * 0.5 = 60.0 == min, 20.0 * 0.5 = 10.0 == min
    let result = effective_size(
        120.0,
        20.0,
        1.0,
        0.5,
        ClampRange {
            min: Some(60.0),
            max: Some(600.0),
        },
        ClampRange {
            min: Some(10.0),
            max: Some(100.0),
        },
    );
    assert!(
        (result.x - 60.0).abs() < 1e-3,
        "expected width at min 60.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 10.0).abs() < 1e-3,
        "expected height at min 10.0, got {}",
        result.y,
    );
}

// Behavior 7: At exactly max boundary stays at max

#[test]
fn effective_size_exactly_at_max_boundary() {
    // 120.0 * 5.0 = 600.0 == max, 20.0 * 5.0 = 100.0 == max
    let result = effective_size(
        120.0,
        20.0,
        5.0,
        1.0,
        ClampRange {
            min: Some(60.0),
            max: Some(600.0),
        },
        ClampRange {
            min: Some(10.0),
            max: Some(100.0),
        },
    );
    assert!(
        (result.x - 600.0).abs() < 1e-3,
        "expected width at max 600.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 100.0).abs() < 1e-3,
        "expected height at max 100.0, got {}",
        result.y,
    );
}

#[test]
fn effective_size_slightly_below_max_not_clamped() {
    // 120.0 * 4.99 = 598.8, 20.0 * 4.99 = 99.8 -- both under max
    let result = effective_size(
        120.0,
        20.0,
        4.99,
        1.0,
        ClampRange {
            min: Some(60.0),
            max: Some(600.0),
        },
        ClampRange {
            min: Some(10.0),
            max: Some(100.0),
        },
    );
    assert!(
        (result.x - 598.8).abs() < 1e-1,
        "expected width 598.8 (not clamped), got {}",
        result.x,
    );
    assert!(
        (result.y - 99.8).abs() < 1e-1,
        "expected height 99.8 (not clamped), got {}",
        result.y,
    );
}

// Behavior 8: None min/max skips clamping entirely

#[test]
fn effective_size_none_constraints_no_clamping() {
    // 120.0 * 10.0 = 1200.0, 20.0 * 10.0 = 200.0 -- unclamped
    let result = effective_size(120.0, 20.0, 10.0, 1.0, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 1200.0).abs() < 1e-3,
        "expected width 1200.0 (unclamped), got {}",
        result.x,
    );
    assert!(
        (result.y - 200.0).abs() < 1e-3,
        "expected height 200.0 (unclamped), got {}",
        result.y,
    );
}

#[test]
fn effective_size_none_constraints_very_small_scale() {
    // 120.0 * 0.01 = 1.2, 20.0 * 0.01 = 0.2 -- unclamped
    let result = effective_size(120.0, 20.0, 1.0, 0.01, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 1.2).abs() < 1e-3,
        "expected width 1.2 (unclamped), got {}",
        result.x,
    );
    assert!(
        (result.y - 0.2).abs() < 1e-3,
        "expected height 0.2 (unclamped), got {}",
        result.y,
    );
}

// Behavior 9: Partial None constraints clamp only specified dimension

#[test]
fn effective_size_partial_constraints_clamps_only_specified() {
    // max_width = Some(200.0), rest None
    // width: 120.0 * 10.0 = 1200.0, clamped to 200.0
    // height: 20.0 * 10.0 = 200.0, unclamped (None)
    let result = effective_size(
        120.0,
        20.0,
        10.0,
        1.0,
        ClampRange {
            min: None,
            max: Some(200.0),
        },
        ClampRange::NONE,
    );
    assert!(
        (result.x - 200.0).abs() < 1e-3,
        "expected width clamped to 200.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 200.0).abs() < 1e-3,
        "expected height 200.0 (unclamped), got {}",
        result.y,
    );
}

#[test]
fn effective_size_only_min_width_specified() {
    // min_width = Some(200.0), rest None, node_scaling_factor = 0.01
    // width: 120.0 * 0.01 = 1.2, clamped to 200.0
    // height: 20.0 * 0.01 = 0.2, unclamped
    let result = effective_size(
        120.0,
        20.0,
        1.0,
        0.01,
        ClampRange {
            min: Some(200.0),
            max: None,
        },
        ClampRange::NONE,
    );
    assert!(
        (result.x - 200.0).abs() < 1e-3,
        "expected width clamped to min 200.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 0.2).abs() < 1e-3,
        "expected height 0.2 (unclamped), got {}",
        result.y,
    );
}

// Behavior 10: Multiple stacked boosts (pre-computed multiplier)

#[test]
fn effective_size_stacked_boosts_precomputed() {
    // multiplier = 3.0 (product of [1.5, 2.0])
    // 120.0 * 3.0 = 360.0, 20.0 * 3.0 = 60.0
    let result = effective_size(120.0, 20.0, 3.0, 1.0, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 360.0).abs() < 1e-3,
        "expected width 360.0, got {}",
        result.x,
    );
    assert!(
        (result.y - 60.0).abs() < 1e-3,
        "expected height 60.0, got {}",
        result.y,
    );
}

#[test]
fn effective_size_empty_boosts_identity_multiplier() {
    // empty boosts => multiplier = 1.0
    let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
    assert!(
        (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
        "empty boosts (multiplier 1.0) should return base, got ({}, {})",
        result.x,
        result.y,
    );
}

// ── Part B: effective_radius pure function tests ────────────────────

// Behavior 1: No boosts, no node scale, no constraints returns base radius

#[test]
fn effective_radius_no_boosts_no_scale_returns_base_radius() {
    let result = effective_radius(8.0, 1.0, 1.0, ClampRange::NONE);
    assert!(
        (result - 8.0).abs() < f32::EPSILON,
        "expected 8.0, got {result}",
    );
}

#[test]
fn effective_radius_zero_base_with_identity_multipliers_returns_zero() {
    let result = effective_radius(0.0, 1.0, 1.0, ClampRange::NONE);
    assert!(result.abs() < f32::EPSILON, "expected 0.0, got {result}");
}

// Behavior 2: Size boost multiplier scales radius

#[test]
fn effective_radius_boost_scales_radius() {
    let result = effective_radius(8.0, 2.0, 1.0, ClampRange::NONE);
    assert!(
        (result - 16.0).abs() < f32::EPSILON,
        "expected 16.0, got {result}",
    );
}

#[test]
fn effective_radius_identity_boost_returns_base() {
    let result = effective_radius(8.0, 1.0, 1.0, ClampRange::NONE);
    assert!(
        (result - 8.0).abs() < f32::EPSILON,
        "identity boost should return 8.0, got {result}",
    );
}

// Behavior 3: Node scaling factor scales radius

#[test]
fn effective_radius_node_scale_scales_radius() {
    let result = effective_radius(8.0, 1.0, 0.5, ClampRange::NONE);
    assert!(
        (result - 4.0).abs() < f32::EPSILON,
        "expected 4.0, got {result}",
    );
}

#[test]
fn effective_radius_identity_node_scale_returns_base() {
    let result = effective_radius(8.0, 1.0, 1.0, ClampRange::NONE);
    assert!(
        (result - 8.0).abs() < f32::EPSILON,
        "identity node scale should return 8.0, got {result}",
    );
}

// Behavior 4: Both boost and node scale multiply together

#[test]
fn effective_radius_boost_and_node_scale_multiply() {
    let result = effective_radius(8.0, 2.0, 0.5, ClampRange::NONE);
    assert!(
        (result - 8.0).abs() < f32::EPSILON,
        "expected 8.0 (8.0 * 2.0 * 0.5), got {result}",
    );
}

#[test]
fn effective_radius_large_boost_with_fractional_scale() {
    let result = effective_radius(14.0, 3.0, 0.7, ClampRange::NONE);
    assert!(
        (result - 29.4).abs() < 1e-3,
        "expected 29.4 (14.0 * 3.0 * 0.7), got {result}",
    );
}

// Behavior 5: Clamps radius to max when boost would exceed it

#[test]
fn effective_radius_clamps_to_max() {
    let result = effective_radius(
        8.0,
        10.0,
        1.0,
        ClampRange {
            min: Some(4.0),
            max: Some(20.0),
        },
    );
    assert!(
        (result - 20.0).abs() < f32::EPSILON,
        "expected 20.0 (80.0 clamped to max), got {result}",
    );
}

#[test]
fn effective_radius_exactly_at_max_stays_at_max() {
    let result = effective_radius(
        10.0,
        2.0,
        1.0,
        ClampRange {
            min: None,
            max: Some(20.0),
        },
    );
    assert!(
        (result - 20.0).abs() < f32::EPSILON,
        "expected 20.0 (exactly at max), got {result}",
    );
}

// Behavior 6: Clamps radius to min when scale would shrink below it

#[test]
fn effective_radius_clamps_to_min() {
    let result = effective_radius(
        8.0,
        1.0,
        0.1,
        ClampRange {
            min: Some(4.0),
            max: Some(20.0),
        },
    );
    assert!(
        (result - 4.0).abs() < f32::EPSILON,
        "expected 4.0 (0.8 clamped to min), got {result}",
    );
}

#[test]
fn effective_radius_exactly_at_min_stays_at_min() {
    let result = effective_radius(
        8.0,
        1.0,
        0.5,
        ClampRange {
            min: Some(4.0),
            max: None,
        },
    );
    assert!(
        (result - 4.0).abs() < f32::EPSILON,
        "expected 4.0 (exactly at min), got {result}",
    );
}

// Behavior 7: None constraints skip clamping entirely

#[test]
fn effective_radius_none_constraints_no_clamping() {
    let result = effective_radius(8.0, 10.0, 1.0, ClampRange::NONE);
    assert!(
        (result - 80.0).abs() < f32::EPSILON,
        "expected 80.0 (unclamped), got {result}",
    );
}

#[test]
fn effective_radius_none_constraints_very_small_scale() {
    let result = effective_radius(8.0, 1.0, 0.01, ClampRange::NONE);
    assert!(
        (result - 0.08).abs() < 1e-6,
        "expected 0.08 (unclamped), got {result}",
    );
}

// Behavior 8: Partial None constraints clamp only specified bound

#[test]
fn effective_radius_max_only_clamps_to_max() {
    let result = effective_radius(
        8.0,
        10.0,
        1.0,
        ClampRange {
            min: None,
            max: Some(20.0),
        },
    );
    assert!(
        (result - 20.0).abs() < f32::EPSILON,
        "expected 20.0 (max clamped, no min), got {result}",
    );
}

#[test]
fn effective_radius_min_only_clamps_to_min() {
    let result = effective_radius(
        8.0,
        1.0,
        0.01,
        ClampRange {
            min: Some(4.0),
            max: None,
        },
    );
    assert!(
        (result - 4.0).abs() < f32::EPSILON,
        "expected 4.0 (min clamped, no max cap), got {result}",
    );
}
