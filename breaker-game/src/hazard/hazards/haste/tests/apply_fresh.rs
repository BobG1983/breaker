//! Group B — `haste_apply_speed` on bolts without an `EffectStack`.
//!
//! Wires only `haste_apply_speed` in `FixedUpdate` (bypasses run-conditions),
//! so these tests must explicitly install config and stacks.

use super::{
    super::system::HasteConfig,
    helpers::{
        add_haste_stacks, haste_entries, install_haste_config, run_fixed_update, spawn_bolt,
        spawn_bolt_with_stack, test_app_playing, wire_apply_only,
    },
};
use crate::{
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::definition::HazardKind,
    prelude::*,
};

// ── Behavior 8 — bolt without stack gets one with Haste entry ────────────

#[test]
fn bolt_without_stack_gets_one_with_haste_entry() {
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

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("EffectStack inserted");
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}

#[test]
fn bolt_fresh_insert_single_entry_has_haste_source_and_1_20x() {
    // Edge: the single entry's source is `"hazard:haste"` and its
    // multiplier passes a float-tolerance check (bitwise `OrderedFloat::eq`
    // is unreliable against formula output).
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

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    let entries = haste_entries(stack);
    assert_eq!(entries.len(), 1);
    let (source, config) = &entries[0];
    assert_eq!(source, &SourceId::hazard(HazardKind::Haste).build());
    // Tolerance compare — OrderedFloat::eq is bitwise and f32 arithmetic
    // from the formula may produce slightly different bits than `1.20_f32`.
    assert!((config.multiplier.into_inner() - 1.20_f32).abs() < 1e-6);
}

// ── Behavior 9 — two bolts each get independent Haste stacks ─────────────

#[test]
fn two_bolts_each_get_independent_haste_stacks() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 2);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .expect("bolt_a should have its own stack");
    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_b)
        .expect("bolt_b should have its own stack");
    assert_eq!(stack_a.len(), 1);
    assert_eq!(stack_b.len(), 1);
    assert!((stack_a.aggregate() - 1.30).abs() < 1e-6);
    assert!((stack_b.aggregate() - 1.30).abs() < 1e-6);
}

#[test]
fn despawning_one_bolt_does_not_perturb_another_bolts_stack() {
    // Edge: despawn `bolt_b` between two ticks. `bolt_a`'s stack is
    // unchanged.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 2);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    // Despawn bolt_b between ticks.
    app.world_mut().entity_mut(bolt_b).despawn();
    run_fixed_update(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_a)
        .unwrap();
    assert_eq!(stack_a.len(), 1);
    assert!((stack_a.aggregate() - 1.30).abs() < 1e-6);
}

// ── Behavior 10 — zero bolts: system runs cleanly ────────────────────────

#[test]
fn zero_bolts_system_runs_cleanly_no_stack_inserted() {
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
    // No bolts spawned.

    run_fixed_update(&mut app);

    let count = app
        .world_mut()
        .query::<&EffectStack<SpeedBoostConfig>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0, "no EffectStack should appear without bolts");
}

#[test]
fn non_bolt_entity_does_not_get_speed_boost_stack() {
    // Edge: a non-Bolt entity is NOT given an `EffectStack<SpeedBoostConfig>`
    // — the query filters `With<Bolt>`.
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
    let other = app.world_mut().spawn_empty().id();

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(other)
            .is_none(),
        "non-Bolt entity must not receive a SpeedBoostConfig stack"
    );
}

// ── Behavior 10A — bolt with pre-existing EMPTY EffectStack reconciles ──

#[test]
fn bolt_with_empty_pre_existing_stack_reconciles_via_update_path() {
    // The `if let Some(mut stack)` branch handles a present-but-empty
    // stack (distinct from behaviour 8's fresh-insert path).
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
    let bolt = spawn_bolt_with_stack(&mut app, EffectStack::<SpeedBoostConfig>::default());
    // Sanity check: the seed really is present-but-empty.
    assert_eq!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .expect("seeded empty stack must exist")
            .len(),
        0
    );

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}

#[test]
fn empty_pre_existing_stack_reconciliation_is_idempotent() {
    // Edge: a second consecutive tick leaves len == 1, aggregate == 1.20.
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
    let bolt = spawn_bolt_with_stack(&mut app, EffectStack::<SpeedBoostConfig>::default());

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}
