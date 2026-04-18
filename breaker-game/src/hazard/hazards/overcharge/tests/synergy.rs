//! Group G — Multi-bolt, Haste synergy, and cleanup.
//!
//! Pins the multiplicative semantics of Overcharge against chip- and
//! Haste-owned speed boosts, and Bevy's natural entity cleanup on despawn.

use ordered_float::OrderedFloat;

use super::{
    super::system::{OverchargeKillCount, register},
    helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, overcharge_entries,
        run_fixed_update, spawn_bolt, spawn_bolt_with_stack, spawn_cell, test_app_playing,
        wire_apply_only, write_bump, write_cell_destroyed,
    },
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

// ── Behavior 45 — two bolts accumulate kills independently ──────────────

#[test]
fn two_bolts_accumulate_kills_independently() {
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));

    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 2);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 1);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .unwrap();
    assert_eq!(stack_a.len(), 1);
    assert!(1.05_f32.mul_add(-1.05_f32, stack_a.aggregate()).abs() < 1e-5);

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .unwrap();
    assert_eq!(stack_b.len(), 1);
    assert!((stack_b.aggregate() - 1.05).abs() < 1e-6);
}

#[test]
fn two_bolts_converge_to_same_multiplier_under_same_kill_count() {
    // Edge: after one more bolt_b kill in a later tick, both converge
    // to OverchargeKillCount(2) and aggregate ≈ 1.05^2.
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));
    run_fixed_update(&mut app);

    // Next tick: one more kill by bolt_b.
    let cell_3 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_3, Some(bolt_b));
    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 2);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 2);
    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .unwrap();
    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .unwrap();
    assert!(1.05_f32.mul_add(-1.05_f32, stack_a.aggregate()).abs() < 1e-5);
    assert!(1.05_f32.mul_add(-1.05_f32, stack_b.aggregate()).abs() < 1e-5);
}

// ── Behavior 46 — bump on one bolt resets only that bolt ────────────────

#[test]
fn bump_on_one_bolt_resets_only_that_bolt() {
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));
    run_fixed_update(&mut app);

    write_bump(&mut app, Some(bolt_a));
    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 0);
    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .unwrap();
    assert_eq!(stack_a.len(), 0);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 1);
    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .unwrap();
    assert!((stack_b.aggregate() - 1.05).abs() < 1e-6);
}

#[test]
fn bumps_on_both_bolts_in_same_tick_reset_both() {
    // Edge: both bolts bumped in the same tick; both drop to count 0 and
    // have zero-length stacks.
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));
    run_fixed_update(&mut app);

    write_bump(&mut app, Some(bolt_a));
    write_bump(&mut app, Some(bolt_b));
    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 0);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 0);
    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .unwrap();
    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .unwrap();
    assert_eq!(stack_a.len(), 0);
    assert_eq!(stack_b.len(), 0);
}

// ── Behavior 47 — Haste + Overcharge synergy: multiplicative ────────────

#[test]
fn haste_and_overcharge_entries_multiply_via_distinct_sources() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.20),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 3, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 2);
    let expected = 1.20 * 1.05_f32.powi(3);
    assert!((stack.aggregate() - expected).abs() < 1e-5);

    let entries = overcharge_entries(stack);
    let haste = entries
        .iter()
        .find(|(s, _)| s == "hazard:haste")
        .expect("haste entry preserved");
    assert_eq!(haste.1.multiplier, OrderedFloat(1.20));
    let over = entries
        .iter()
        .find(|(s, _)| s == "hazard:overcharge")
        .expect("overcharge entry present");
    assert!((over.1.multiplier.into_inner() - 1.05_f32.powi(3)).abs() < 1e-6);
}

#[test]
fn bumping_overcharge_stacks_updates_only_overcharge_entry_in_synergy() {
    // Edge: bump Overcharge from 1 → 2 stacks (multiplier 1.05 → 1.08).
    // Aggregate moves to 1.20 * 1.08^3. Haste entry is untouched.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.20),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 3, seed);

    run_fixed_update(&mut app);
    // Bump stacks to 2 → multiplier = 1.08.
    add_overcharge_stacks(&mut app, 1);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 2);
    let expected = 1.20 * 1.08_f32.powi(3);
    assert!((stack.aggregate() - expected).abs() < 1e-5);

    // Haste entry untouched.
    let entries = overcharge_entries(stack);
    let haste = entries
        .iter()
        .find(|(s, _)| s == "hazard:haste")
        .expect("haste entry persists");
    assert_eq!(haste.1.multiplier, OrderedFloat(1.20));
}

// ── Behavior 48 — bolt despawn cleans up its components ─────────────────

#[test]
fn bolt_despawn_cleans_up_without_affecting_other_bolt() {
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));
    run_fixed_update(&mut app);

    // Despawn bolt_a; run a quiescent tick.
    app.world_mut().despawn(bolt_a);
    run_fixed_update(&mut app);

    assert!(
        app.world().get_entity(bolt_a).is_err(),
        "bolt_a must be gone"
    );
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 1);
    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .unwrap();
    assert!((stack_b.aggregate() - 1.05).abs() < 1e-6);
}

#[test]
fn despawned_bolt_killer_id_is_rejected_without_contamination() {
    // Edge: after despawning bolt_a, a bogus Destroyed<Cell> with the
    // stale id is filtered out; bolt_b state is unchanged.
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));
    run_fixed_update(&mut app);

    app.world_mut().despawn(bolt_a);
    let cell_stale = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_stale, Some(bolt_a));
    run_fixed_update(&mut app);

    assert_eq!(
        app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0,
        1,
        "bolt_b unchanged by stale id kill"
    );
    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .unwrap();
    assert!((stack_b.aggregate() - 1.05).abs() < 1e-6);
}

// ── Behavior 49 — full chain on a bolt with seed chip + haste entries ───

#[test]
fn full_chain_with_seeded_chip_and_haste_entries_aggregates_correctly() {
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 2);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.20),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 0, seed);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 1);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 3);
    // 1.5 * 1.20 * 1.08^1 = 1.944
    let expected = 1.5 * 1.20 * 1.08_f32.powi(1);
    assert!((stack.aggregate() - expected).abs() < 1e-5);

    let entries = overcharge_entries(stack);
    let chip = entries
        .iter()
        .find(|(s, _)| s == "chip:overclock")
        .expect("chip preserved");
    assert_eq!(chip.1.multiplier, OrderedFloat(1.5));
    let haste = entries
        .iter()
        .find(|(s, _)| s == "hazard:haste")
        .expect("haste preserved");
    assert_eq!(haste.1.multiplier, OrderedFloat(1.20));
}

#[test]
fn full_chain_with_seeded_entries_updates_only_overcharge_across_ticks() {
    // Edge: a second kill in a later tick updates the Overcharge entry to
    // 1.08^2; chip and haste entries keep their original multipliers.
    let mut app = test_app_playing();
    register(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 2);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.20),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 0, seed);

    let cell_a = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_a, Some(bolt));
    run_fixed_update(&mut app);

    let cell_b = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_b, Some(bolt));
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 2);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 3);
    // 1.5 * 1.20 * 1.08^2 ≈ 2.09952
    let expected = 1.5 * 1.20 * 1.08_f32.powi(2);
    assert!((stack.aggregate() - expected).abs() < 1e-5);

    let entries = overcharge_entries(stack);
    let chip = entries.iter().find(|(s, _)| s == "chip:overclock").unwrap();
    assert_eq!(chip.1.multiplier, OrderedFloat(1.5));
    let haste = entries.iter().find(|(s, _)| s == "hazard:haste").unwrap();
    assert_eq!(haste.1.multiplier, OrderedFloat(1.20));
}
