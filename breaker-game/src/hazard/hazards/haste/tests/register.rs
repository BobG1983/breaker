//! Group D — `register` wiring: scheduling, run-condition gating, state
//! gating, and the stack-0 persistence pin.
//!
//! These use `register(&mut app)` to test the wiring, NOT hand-wired
//! systems.

use ordered_float::OrderedFloat;

use super::{
    super::system::{HasteConfig, register},
    helpers::{
        add_haste_stacks, canonical_config, install_haste_config, run_fixed_update, spawn_bolt,
        spawn_bolt_with_stack, test_app_not_playing, test_app_playing,
    },
};
use crate::{
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{definition::HazardKind, resources::ActiveHazards},
};

// ── Behavior 16 — register-wired system applies Haste in FixedUpdate ────

#[test]
fn register_wired_system_applies_haste_entry_when_both_gates_open() {
    let mut app = test_app_playing();
    register(&mut app);
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
        .expect("register-wired system must apply Haste in FixedUpdate");
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}

#[test]
fn register_wired_system_is_idempotent_across_two_ticks() {
    // Edge: a second consecutive tick leaves len == 1 and aggregate == 1.20.
    let mut app = test_app_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());
    add_haste_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}

// ── Behavior 17 — hazard_active(Haste) false → system does not run ─────

#[test]
fn system_skipped_when_haste_run_condition_false() {
    let mut app = test_app_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());
    // Stack a DIFFERENT hazard — Haste remains inactive.
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);
    }
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Haste),
        0
    );
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "gated-off system must not insert an EffectStack"
    );
}

#[test]
fn system_runs_once_haste_stack_added_after_initial_skip() {
    // Edge: with 0 Haste stacks, no stack inserted. Add 1 Haste stack and
    // tick again — NOW the stack appears with aggregate == 1.20.
    let mut app = test_app_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);
    }
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);
    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none()
    );

    add_haste_stacks(&mut app, 1);
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("adding Haste stack must toggle the gate on");
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.20).abs() < 1e-6);
}

// ── Behavior 18 — NOT in NodeState::Playing → system does not run ──────

#[test]
fn system_skipped_when_not_in_node_playing() {
    let mut app = test_app_not_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());
    add_haste_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "state gate must suppress the system outside NodeState::Playing"
    );
}

// ── Behavior 19 — zero Haste stacks + empty ActiveHazards: no stack ────

#[test]
fn register_wired_zero_stacks_empty_active_does_not_insert_stack() {
    // Distinguishes from behaviour 17: ActiveHazards is completely empty
    // (no other hazard stacks either). Pins that empty ActiveHazards
    // gates the system off just like a zero-stack Haste entry does.
    let mut app = test_app_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());
    // Zero stacks of any hazard.
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "empty ActiveHazards must gate the system off"
    );
}

#[test]
fn register_wired_zero_stacks_stays_off_across_five_ticks() {
    // Edge: repeat across 5 consecutive ticks — no stack ever appears.
    // Guards against a bug where accumulated empty-gate ticks eventually
    // produce a false stack insertion.
    let mut app = test_app_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());
    let bolt = spawn_bolt(&mut app);

    for _ in 0..5 {
        run_fixed_update(&mut app);
    }

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .is_none(),
        "no stack must appear across repeated empty-gate ticks"
    );
}

// ── Behavior 20 — pre-existing HASTE_SOURCE entry NOT removed at stack 0 ─
//
// REGRESSION PIN: with 0 Haste stacks, the run-condition gates the system
// off, so nobody calls `retain_by_source("hazard:haste")`. Any pre-existing
// HASTE_SOURCE entry on a bolt's stack persists. Flagged as a design
// concern in the test spec.

#[test]
fn stack_zero_preserves_pre_existing_haste_source_entry() {
    let mut app = test_app_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());
    // Zero current Haste stacks — no `add_haste_stacks` call.

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.40),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("seeded EffectStack must remain on the bolt");
    assert_eq!(stack.len(), 1);
    assert!(
        (stack.aggregate() - 1.40).abs() < 1e-6,
        "stack-0 with hazard_active=false must leave the stale Haste entry untouched"
    );
}

#[test]
fn stack_zero_preserves_chip_and_pre_existing_haste_entries() {
    // Edge: identical setup plus a chip:overclock entry in the seed.
    // After the tick: len == 2, aggregate == 1.40 * 1.5 = 2.10. The
    // system simply doesn't run — both entries are preserved.
    let mut app = test_app_playing();
    register(&mut app);
    install_haste_config(&mut app, canonical_config());

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.40),
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
    assert!((stack.aggregate() - 2.10).abs() < 1e-5);
}
