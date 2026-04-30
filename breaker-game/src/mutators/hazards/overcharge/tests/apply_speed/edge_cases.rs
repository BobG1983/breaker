//! Behaviors 30–33 — no bolts, no config, zero stacks, and overflow boundary.

use ordered_float::OrderedFloat;

use super::{
    super::helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, overcharge_entries,
        run_fixed_update, spawn_bolt_with_count, spawn_bolt_with_stack, test_app_playing,
        wire_apply_only,
    },
    helpers::{chip_overclock, hazard_overcharge},
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

// ── Behavior 30 — no bolts in the world is a no-op ───────────────────────

#[test]
fn apply_speed_no_bolts_is_noop() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    run_fixed_update(&mut app);

    let count = app
        .world_mut()
        .query::<&EffectStack<SpeedBoostConfig>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0);
}

#[test]
fn apply_speed_skips_non_bolt_entities() {
    // Edge: a non-Bolt entity does NOT receive an EffectStack — the query
    // is With<Bolt>.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let non_bolt = app.world_mut().spawn_empty().id();

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(non_bolt)
            .is_none()
    );
}

// ── Behavior 31 — OverchargeConfig absent is a no-op (no panic) ──────────

#[test]
fn apply_speed_without_config_is_noop() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    // DO NOT install config.
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 3);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

#[test]
fn apply_speed_without_config_preserves_seeded_chip_entries() {
    // Edge: without config, the early-return fires before the per-entity
    // loop. A seeded chip entry on a second bolt is untouched.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 2, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1, "chip entry untouched by early-return");
    let entries = overcharge_entries(stack);
    assert_eq!(entries[0].0, chip_overclock());
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
}

// ── Behavior 32 — zero Overcharge stacks drives per-kill multiplier to 1 ─

#[test]
fn apply_speed_zero_stacks_collapses_multiplier_to_one() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    // ZERO Overcharge stacks — no add_overcharge_stacks call.
    let bolt = spawn_bolt_with_count(&mut app, 5);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.0).abs() < 1e-6);
    let entries = overcharge_entries(stack);
    assert_eq!(entries[0].0, hazard_overcharge());
}

#[test]
fn apply_speed_zero_stacks_with_zero_kills_inserts_no_entry() {
    // Edge: zero stacks + zero kills → kills > 0 guard fires first,
    // no entry inserted.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    let bolt = spawn_bolt_with_count(&mut app, 0);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

// ── Behavior 33 — u32::MAX kill count produces a finite positive value ──

#[test]
fn apply_speed_u32_max_kills_is_finite_and_positive() {
    // REGRESSION PIN: `kills.cast_signed()` wraps u32::MAX to -1_i32, so
    // powi(-1) gives a reciprocal (≈ 1/1.05 ≈ 0.9524). This test pins
    // the current wrap-to-reciprocal behavior as a known boundary: any
    // future change must be a deliberate, spec'd decision.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, u32::MAX);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    let agg = stack.aggregate();
    assert!(agg.is_finite(), "aggregate must be finite at u32::MAX");
    assert!(agg > 0.0, "aggregate must be strictly positive");
}

#[test]
fn apply_speed_u32_max_minus_one_kills_is_finite_and_positive() {
    // Edge: u32::MAX - 1 wraps to -2_i32, aggregate ≈ 1/1.05^2 ≈ 0.907.
    // Confirms the cast_signed wrap is not a single-value accident.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, u32::MAX - 1);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    let agg = stack.aggregate();
    assert!(agg.is_finite());
    assert!(agg > 0.0);
}
