//! Group C — `haste_apply_speed` reconciliation on existing stacks.
//!
//! Wires only `haste_apply_speed` in `FixedUpdate` (bypasses run-conditions).

use ordered_float::OrderedFloat;

use super::{
    super::system::HasteConfig,
    helpers::{
        add_haste_stacks, haste_entries, install_haste_config, run_fixed_update, spawn_bolt,
        spawn_bolt_with_stack, test_app_playing, wire_apply_only,
    },
};
use crate::{
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{definition::HazardKind, resources::ActiveHazards},
};

// ── Behavior 11 — reconciles to a single Haste entry across three ticks ─

#[test]
fn reconciles_to_single_haste_entry_across_ticks() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1, "must not accumulate duplicate entries");
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}

#[test]
fn reconciles_three_ticks_leaves_single_haste_source_entry() {
    // Edge: after three ticks, haste_entries returns exactly one tuple
    // with source == "hazard:haste".
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    let entries = haste_entries(stack);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "hazard:haste");
}

// ── Behavior 12 — multiplier updates when stacks increase ───────────────

#[test]
fn updates_multiplier_when_stacks_increase() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    // Bump stacks to 3
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Haste);
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Haste);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.40).abs() < 1e-6);
}

#[test]
fn reconciliation_idempotent_at_new_stack_count_on_third_tick() {
    // Edge: after a third tick with no further stack change, aggregate is
    // still 1.40 and len == 1 — reconciliation is idempotent at the new
    // stack count.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    add_haste_stacks(&mut app, 2); // stacks = 3
    run_fixed_update(&mut app);
    run_fixed_update(&mut app); // third tick, no further stack change

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.40).abs() < 1e-6);
}

// ── Behavior 13 — preserves non-Haste entries on an existing stack ──────

#[test]
fn preserves_non_haste_entries_on_existing_stack() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    // Seed with a chip-owned boost.
    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    // 1.5 (chip) * 1.20 (haste) = 1.80
    assert_eq!(stack.len(), 2);
    assert!((stack.aggregate() - 1.80).abs() < 1e-5);
}

#[test]
fn chip_entry_survives_second_tick_and_retains_multiplier() {
    // Edge: after a second tick, len == 2, aggregate == 1.80, and the
    // chip:overclock entry's multiplier is still OrderedFloat(1.5).
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 2);
    assert!((stack.aggregate() - 1.80).abs() < 1e-5);
    let entries = haste_entries(stack);
    let chip = entries
        .iter()
        .find(|(s, _)| s == "chip:overclock")
        .expect("chip:overclock entry must survive reconciliation");
    assert_eq!(chip.1.multiplier, OrderedFloat(1.5));
}

// ── Behavior 14 — three non-Haste entries survive reconciliation ────────

#[test]
fn three_non_haste_entries_survive_reconciliation() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        "chip:feedback_loop".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    seed.push(
        "protocol:velocity_bias".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.10),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    // 1.5 * 1.25 * 1.10 * 1.20 = 2.475
    assert_eq!(stack.len(), 4);
    assert!((stack.aggregate() - 2.475).abs() < 1e-4);

    let entries = haste_entries(stack);
    for src in [
        "chip:overclock",
        "chip:feedback_loop",
        "protocol:velocity_bias",
    ] {
        let count = entries.iter().filter(|(s, _)| s == src).count();
        assert_eq!(count, 1, "source {src} must appear exactly once");
    }
}

#[test]
fn three_non_haste_entries_survive_second_tick_with_tolerance_check() {
    // Edge: after a second tick, all four sources remain. The Haste entry
    // is a single tuple with `source == "hazard:haste"` and its multiplier
    // satisfies `(m - 1.20_f32).abs() < 1e-6` (tolerance — not bitwise).
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        "chip:feedback_loop".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    seed.push(
        "protocol:velocity_bias".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.10),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 4);

    let entries = haste_entries(stack);
    let haste_hits: Vec<_> = entries
        .iter()
        .filter(|(s, _)| s == "hazard:haste")
        .collect();
    assert_eq!(haste_hits.len(), 1, "haste entry must not duplicate");
    let (_, cfg) = haste_hits[0];
    // Tolerance compare — OrderedFloat::eq is bitwise.
    assert!((cfg.multiplier.into_inner() - 1.20_f32).abs() < 1e-6);
}

// ── Behavior 15 — replaces stale HASTE_SOURCE entry, no duplicates ─────

#[test]
fn replaces_stale_haste_source_entry_rather_than_duplicating() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    // Seed with a stale HASTE_SOURCE entry — deliberately unlike anything
    // the current config would produce.
    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(9.99),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    // Must be 1, NOT 2. If reconciliation allowed duplicates, aggregate
    // would be 9.99 * 1.20 ≈ 11.988 — the tripwire value.
    assert_eq!(stack.len(), 1);
    let entries = haste_entries(stack);
    assert_eq!(entries[0].0, "hazard:haste");
    assert!((entries[0].1.multiplier.into_inner() - 1.20_f32).abs() < 1e-6);
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}

#[test]
fn two_stale_haste_entries_plus_chip_reconciles_to_one_haste_plus_chip() {
    // Edge: seed TWO stale Haste entries + one chip entry. After one
    // tick, len == 2, aggregate == 1.5 * 1.20 = 1.80. Both stale Haste
    // entries are gone; the chip is intact.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(5.0),
        },
    );
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(3.0),
        },
    );
    seed.push(
        "chip:overclock".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 2);
    assert!((stack.aggregate() - 1.80).abs() < 1e-5);

    let entries = haste_entries(stack);
    let haste_count = entries.iter().filter(|(s, _)| s == "hazard:haste").count();
    let chip_count = entries
        .iter()
        .filter(|(s, _)| s == "chip:overclock")
        .count();
    assert_eq!(haste_count, 1, "exactly one haste entry must remain");
    assert_eq!(chip_count, 1, "chip entry must survive");
}
