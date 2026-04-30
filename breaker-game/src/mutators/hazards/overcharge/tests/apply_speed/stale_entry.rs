//! Behaviors 26 & 28 — stale Overcharge entry is replaced on reset / not duplicated.

use ordered_float::OrderedFloat;

use super::{
    super::{
        super::system::OverchargeKillCount,
        helpers::{
            add_overcharge_stacks, canonical_config, install_overcharge_config, overcharge_entries,
            run_fixed_update, spawn_bolt_with_count, spawn_bolt_with_stack, test_app_playing,
            wire_apply_only,
        },
    },
    helpers::{chip_overclock, hazard_overcharge},
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

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
