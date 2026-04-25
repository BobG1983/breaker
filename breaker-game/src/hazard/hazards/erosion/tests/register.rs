//! Group D — `register` wiring: scheduling, run-condition gating, state
//! gating, intra-frame ordering.
//!
//! Uses `register(&mut app)` to test the wiring, NOT hand-wired systems.
//! `BumpPerformed` must be registered separately because `register` does not.

use std::time::Duration;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{
    super::system::{ErosionConfig, ErosionState, register},
    helpers::{
        add_erosion_stacks, canonical_config, install_erosion_config, spawn_breaker,
        spawn_breaker_with_stack, test_app_not_playing, test_app_playing, tick_with_dt,
    },
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    chips::definition::Rarity,
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

fn write_bump(app: &mut App, grade: BumpGrade) {
    app.world_mut().write_message(BumpPerformed {
        grade,
        bolt: None,
        breaker: Entity::PLACEHOLDER,
    });
}

// ── D25 — register-wired chain produces reconciled entry at stack 1 ────

#[test]
fn register_wired_chain_fires_and_accumulates_over_two_ticks() {
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState::default());
    add_erosion_stacks(&mut app, 1);
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    {
        let state = app.world().resource::<ErosionState>();
        // shrink 1.0 × 0.05 × 1.0 = 0.05 → width 0.95
        assert!((state.width_fraction - 0.95).abs() < 1e-5);
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .expect("register-wired chain must install EffectStack");
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.95).abs() < 1e-5);
    }

    // Edge: two consecutive 1.0s ticks → width 0.90.
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 0.90).abs() < 1e-5);
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.90).abs() < 1e-5);
}

// ── D26 — register-wired chain gated off when no Erosion stacks ───────

#[test]
fn register_wired_chain_gated_off_when_no_erosion_stacks() {
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState::default());
    // Stack a DIFFERENT hazard to prove gating is Erosion-specific.
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);
    }
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 1.0).abs() < f32::EPSILON,
            "width_fraction should be 1.0"
        );
        assert!(
            app.world()
                .get::<EffectStack<SizeBoostConfig>>(breaker)
                .is_none(),
            "apply_width must not run when gated"
        );
    }

    // Edge: add 1 Erosion stack — chain fires.
    add_erosion_stacks(&mut app, 1);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 0.95).abs() < 1e-5);
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.95).abs() < 1e-5);
}

// ── D27 — register-wired chain gated off outside NodeState::Playing ────

#[test]
fn register_wired_chain_gated_off_when_not_in_node_playing() {
    let mut app = test_app_not_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState::default());
    add_erosion_stacks(&mut app, 1);
    let breaker = spawn_breaker(&mut app);

    // Main: one tick — no shrink, no stack.
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    {
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 1.0).abs() < f32::EPSILON,
            "width_fraction should be 1.0"
        );
        assert!(
            app.world()
                .get::<EffectStack<SizeBoostConfig>>(breaker)
                .is_none()
        );
    }

    // Edge: five consecutive ticks — still no shrink, still no stack.
    for _ in 0..5 {
        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    }
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 1.0).abs() < f32::EPSILON,
        "width_fraction should be 1.0"
    );
    assert!(
        app.world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .is_none()
    );
}

// ── D28 — intra-frame ordering: shrink before restore ─────────────────

#[test]
fn intra_frame_ordering_shrink_before_restore_with_perfect_bump() {
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 1.0,
    });
    add_erosion_stacks(&mut app, 1);
    let breaker = spawn_breaker(&mut app);

    // Write bump BEFORE the tick — the restore system reads it.
    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    // shrink: 1.0 - 0.05 = 0.95. restore sees lost 0.05, restores 0.05 * 0.5 = 0.025.
    // final state: 0.975. apply_width: aggregate ≈ 0.975.
    // If order reversed: restore would see lost=0.0 (no restore), then shrink → 0.95.
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert!(
            (stack.aggregate() - 0.975).abs() < 1e-5,
            "expected 0.975 (shrink-then-restore), got {}",
            stack.aggregate()
        );
    }

    // Edge: on a fresh setup, a tick with NO bump → aggregate ≈ 0.95 (shrink only).
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 1.0,
    });
    add_erosion_stacks(&mut app, 1);
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert!(
        (stack.aggregate() - 0.95).abs() < 1e-5,
        "expected 0.95 (shrink only, no bump), got {}",
        stack.aggregate()
    );
}

// ── D29 — intra-frame ordering: restore before apply_width ────────────

#[test]
fn intra_frame_ordering_restore_before_apply_width_with_full_restore() {
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  1.0, // 100% restore
        },
    );
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.50,
    });
    add_erosion_stacks(&mut app, 1);
    let breaker = spawn_breaker(&mut app);

    write_bump(&mut app, BumpGrade::Perfect);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    // shrink: 0.50 - 0.05 = 0.45. restore: lost 0.55 × 1.0 = 0.55 → min(1.0, 1.0) = 1.0.
    // apply_width reads 1.0: aggregate ≈ 1.0.
    // If apply_width ran before restore: aggregate would be ≈ 0.45.
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert!(
            (stack.aggregate() - 1.0).abs() < f32::EPSILON,
            "expected 1.0 (restore-then-apply), got {}",
            stack.aggregate()
        );
    }

    // Edge: fresh setup, no bump — shrink still runs, aggregate ≈ 0.45.
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(
        &mut app,
        ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  1.0,
        },
    );
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.50,
    });
    add_erosion_stacks(&mut app, 1);
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let state = app.world().resource::<ErosionState>();
    assert!((state.width_fraction - 0.45).abs() < 1e-5);
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert!((stack.aggregate() - 0.45).abs() < 1e-5);
}

// ── D30 — zero Erosion stacks across five ticks leaves world clean ────

#[test]
fn zero_erosion_stacks_across_five_ticks_leaves_world_clean() {
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState::default());
    let breaker = spawn_breaker(&mut app);

    for _ in 0..5 {
        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    }

    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 1.0).abs() < f32::EPSILON,
        "width_fraction should be 1.0"
    );
    assert!(
        app.world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .is_none()
    );
}

// ── D31 — pre-existing EROSION_SOURCE entry PERSISTS at stack 0 ───────

#[test]
fn preexisting_erosion_source_entry_persists_at_stack_zero() {
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState::default());
    // No Erosion stacks.

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        SourceId::hazard(HazardKind::Erosion).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.40),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!(
            (stack.aggregate() - 1.40).abs() < 1e-6,
            "pre-existing erosion entry must persist when gated off, got {}",
            stack.aggregate()
        );
    }

    // Edge: fresh app with seeded erosion + chip entries; both persist.
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState::default());

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        SourceId::hazard(HazardKind::Erosion).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.40),
        },
    );
    seed.push(
        SourceId::chip("heavy").rarity(Rarity::Common).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 2);
    // 1.40 * 1.25 = 1.75
    assert!((stack.aggregate() - 1.75).abs() < 1e-5);
}

// ── MB1 — active-to-zero transition leaves lingering stack ────────────

#[test]
fn active_to_zero_transition_lingering_stack_persists() {
    let mut app = test_app_playing();
    register(&mut app);
    app.add_message::<BumpPerformed>();
    install_erosion_config(&mut app, canonical_config());
    app.world_mut().insert_resource(ErosionState::default());
    add_erosion_stacks(&mut app, 1);
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    {
        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 0.95).abs() < 1e-5);
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.95).abs() < 1e-5);
    }

    // Drain Erosion stacks via the backdoor.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .force_insert_entry(HazardKind::Erosion, 0);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    {
        // Chain gated off — nothing moves.
        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 0.95).abs() < 1e-5,
            "state must persist when gated off"
        );
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1, "lingering stack must persist");
        assert!((stack.aggregate() - 0.95).abs() < 1e-5);
    }

    // Edge: three more ticks at stack 0 — no drift, no regrowth, no cleanup.
    for _ in 0..3 {
        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 0.95).abs() < 1e-5);
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.95).abs() < 1e-5);
    }
}
