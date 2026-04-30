//! Behaviors 23–25 — kill count → compounding multiplier across stack tiers.

use super::{
    super::helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, overcharge_entries,
        run_fixed_update, spawn_bolt_with_count, test_app_playing, wire_apply_only,
    },
    helpers::hazard_overcharge,
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

// ── Behavior 23 — kill count 3 at stack 1 → multiplier 1.05^3 ────────────

#[test]
fn apply_speed_pushes_compounding_multiplier() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 3);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("EffectStack inserted");
    assert_eq!(stack.len(), 1);
    assert!(
        (stack.aggregate() - 1.05_f32.powi(3)).abs() < 1e-5,
        "expected 1.05^3 = {}, got {}",
        1.05_f32.powi(3),
        stack.aggregate()
    );
    // Source is the Overcharge tag.
    let entries = overcharge_entries(stack);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, hazard_overcharge());
}

#[test]
fn apply_speed_is_idempotent_across_two_ticks() {
    // Edge: a second fixed update leaves len == 1 and the aggregate
    // unchanged.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 3);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.05_f32.powi(3)).abs() < 1e-5);
}

// ── Behavior 24 — kill 5 at stack 2 → multiplier 1.08^5 ──────────────────

#[test]
fn apply_speed_stack_two_five_kills_pins_design_doc_value() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 2);
    let bolt = spawn_bolt_with_count(&mut app, 5);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.08_f32.powi(5)).abs() < 1e-5);
}

#[test]
fn apply_speed_stack_two_five_kills_entry_multiplier_matches_aggregate() {
    // Edge: the single entry's multiplier equals the aggregate for a
    // single-entry stack — proves the reconciliation writes the
    // compounded value, not the per-kill value.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 2);
    let bolt = spawn_bolt_with_count(&mut app, 5);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    let entries = overcharge_entries(stack);
    assert_eq!(entries.len(), 1);
    let entry_mult = entries[0].1.multiplier.into_inner();
    assert!((entry_mult - 1.08_f32.powi(5)).abs() < 1e-5);
}

#[test]
fn apply_speed_stack_two_three_kills_pins_design_doc_value() {
    // Edge: stack 2 with 3 kills → 1.08^3 ≈ 1.259712.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 2);
    let bolt = spawn_bolt_with_count(&mut app, 3);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert!((stack.aggregate() - 1.08_f32.powi(3)).abs() < 1e-5);
}

// ── Behavior 25 — kill 10 at stack 3 → multiplier 1.11^10 ────────────────

#[test]
fn apply_speed_stack_three_ten_kills_pins_design_doc_value() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 3);
    let bolt = spawn_bolt_with_count(&mut app, 10);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.11_f32.powi(10)).abs() < 1e-4);
}

#[test]
fn apply_speed_stack_three_ten_kills_aggregate_is_finite_and_bracketed() {
    // Edge: aggregate is finite and bracketed between 2.5 and 3.1 —
    // distinguishes compounding (≈ 2.84) from additive aggregation
    // (≈ 11.1) or from a buggy `1.0 + 1.1`.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 3);
    let bolt = spawn_bolt_with_count(&mut app, 10);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    let agg = stack.aggregate();
    assert!(agg.is_finite(), "aggregate must be finite, got {agg}");
    assert!(
        agg > 2.5 && agg < 3.1,
        "expected 1.11^10 ≈ 2.84, got {agg} (check for additive bug)"
    );
}
