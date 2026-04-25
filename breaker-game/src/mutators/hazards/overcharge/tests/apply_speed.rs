//! Group D — `overcharge_apply_speed` reconcile.
//!
//! Every test wires ONLY `overcharge_apply_speed` via `wire_apply_only`.
//! Install `canonical_config()` and add stacks per test. The reconcile
//! reads `ActiveHazards.stacks(HazardKind::Overcharge)` directly.

use ordered_float::OrderedFloat;

use super::{
    super::system::OverchargeKillCount,
    helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, overcharge_entries,
        run_fixed_update, spawn_bolt, spawn_bolt_with_count, spawn_bolt_with_stack,
        test_app_playing, wire_apply_only,
    },
};
use crate::{
    chips::definition::Rarity,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    mutators::hazards::definition::HazardKind,
    prelude::*,
};

fn hazard_overcharge() -> SourceId {
    SourceId::hazard(HazardKind::Overcharge).build()
}

fn hazard_haste() -> SourceId {
    SourceId::hazard(HazardKind::Haste).build()
}

fn chip_overclock() -> SourceId {
    SourceId::chip("Overclock").rarity(Rarity::Common).build()
}

fn chip_feedback_loop() -> SourceId {
    SourceId::chip("FeedbackLoop")
        .rarity(Rarity::Common)
        .build()
}

// ── Behavior 22 — zero kills / no pre-existing stack → no entry ─────────

#[test]
fn apply_speed_with_zero_kills_inserts_no_entry() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "bolts with zero kills should not receive an Overcharge entry"
    );
}

#[test]
fn apply_speed_with_explicit_zero_kill_component_still_inserts_no_entry() {
    // Edge: bolt HAS OverchargeKillCount(0). Still no stack inserted.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 0);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );
}

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

// ── Behavior 26 — stale Overcharge entry is replaced on reset ────────────

#[test]
fn apply_speed_reconciles_on_reset() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 4);

    run_fixed_update(&mut app);
    // Simulate a bump reset: kill count goes to 0.
    *app.world_mut()
        .get_mut::<OverchargeKillCount>(bolt)
        .unwrap() = OverchargeKillCount(0);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 0, "Overcharge entry must be removed on reset");
}

#[test]
fn apply_speed_stays_empty_after_reset_across_third_tick() {
    // Edge: with kills still 0, a third tick still leaves len == 0.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 4);

    run_fixed_update(&mut app);
    *app.world_mut()
        .get_mut::<OverchargeKillCount>(bolt)
        .unwrap() = OverchargeKillCount(0);
    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 0);
}

// ── Behavior 27 — non-Overcharge entries are preserved ───────────────────

#[test]
fn apply_speed_preserves_non_overcharge_entries() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
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
    assert_eq!(stack.len(), 2);
    // 1.5 (chip) * 1.05^2 (overcharge) = 1.65375
    let expected = 1.5 * 1.05_f32.powi(2);
    assert!((stack.aggregate() - expected).abs() < 1e-5);
}

#[test]
fn apply_speed_preserves_three_foreign_entries() {
    // Edge: seed three non-Overcharge entries + Overcharge.
    // Aggregate = 1.5 * 1.25 * 1.20 * 1.05^2 = 2.480625.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        chip_feedback_loop(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    seed.push(
        hazard_haste(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.20),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 2, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 4);
    let expected = 1.5 * 1.25 * 1.20 * 1.05_f32.powi(2);
    assert!((stack.aggregate() - expected).abs() < 1e-4);

    // Foreign sources retain their original multipliers.
    let entries = overcharge_entries(stack);
    let chip = entries
        .iter()
        .find(|(s, _)| s == &chip_overclock())
        .expect("chip:Overclock survives");
    assert_eq!(chip.1.multiplier, OrderedFloat(1.5));
    let feedback = entries
        .iter()
        .find(|(s, _)| s == &chip_feedback_loop())
        .expect("chip:FeedbackLoop survives");
    assert_eq!(feedback.1.multiplier, OrderedFloat(1.25));
    let haste = entries
        .iter()
        .find(|(s, _)| s == &hazard_haste())
        .expect("hazard:haste survives");
    assert_eq!(haste.1.multiplier, OrderedFloat(1.20));
}

// ── Behavior 28 — stale Overcharge entry is replaced, not duplicated ────

#[test]
fn apply_speed_replaces_stale_overcharge_entry_not_duplicate() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        hazard_overcharge(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(9.99),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 2, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1, "stale entry must be replaced, not added");
    let entries = overcharge_entries(stack);
    let entry_mult = entries[0].1.multiplier.into_inner();
    assert!(1.05_f32.mul_add(-1.05_f32, entry_mult).abs() < 1e-6);
    assert!(1.05_f32.mul_add(-1.05_f32, stack.aggregate()).abs() < 1e-5);
}

#[test]
fn apply_speed_multiple_stale_overcharge_entries_removed_chip_preserved() {
    // Edge: TWO stale Overcharge entries plus a chip entry. After tick:
    // len == 2 (chip + fresh Overcharge), aggregate = 1.5 * 1.05^2.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        hazard_overcharge(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(5.0),
        },
    );
    seed.push(
        hazard_overcharge(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(3.0),
        },
    );
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
    assert_eq!(stack.len(), 2);
    let expected = 1.5 * 1.05_f32.powi(2);
    assert!((stack.aggregate() - expected).abs() < 1e-5);
    // Chip entry preserved.
    let entries = overcharge_entries(stack);
    let chip = entries
        .iter()
        .find(|(s, _)| s == &chip_overclock())
        .expect("chip survives");
    assert_eq!(chip.1.multiplier, OrderedFloat(1.5));
}

// ── Behavior 29 — reconciliation is idempotent across multiple ticks ────

#[test]
fn apply_speed_is_idempotent_across_five_ticks() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 4);

    let expected = 1.05_f32.powi(4);

    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - expected).abs() < 1e-6);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - expected).abs() < 1e-6);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - expected).abs() < 1e-6);
}

#[test]
fn apply_speed_catches_up_after_mid_sequence_kill_count_mutation() {
    // Edge: after three ticks, mutate kill count to 7. Next tick the
    // reconcile updates in one tick to 1.05^7.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 4);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    *app.world_mut()
        .get_mut::<OverchargeKillCount>(bolt)
        .unwrap() = OverchargeKillCount(7);

    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.05_f32.powi(7)).abs() < 1e-5);

    // One more tick — still idempotent.
    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.05_f32.powi(7)).abs() < 1e-5);
}

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
