//! Group C — `erosion_apply_width` (`EffectStack` reconciliation).
//!
//! Wires only `erosion_apply_width` in `FixedUpdate` (bypasses
//! run-conditions). Pins the `EffectStack<SizeBoostConfig>` reconciliation
//! path — source-tagged retain+push, fresh-insert path, multi-breaker
//! independence, stale-entry replacement, tick-to-tick updates,
//! empty-world cleanness, and the no-state guard.

use std::time::Duration;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{
    super::system::{ErosionState, erosion_apply_width},
    helpers::{
        erosion_entries, install_erosion_state, spawn_breaker, spawn_breaker_with_stack,
        test_app_playing, tick_with_dt, wire_apply_width_only,
    },
};
use crate::{
    breaker::components::Breaker,
    chips::definition::Rarity,
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    mutators::hazards::definition::HazardKind,
    prelude::*,
};

// ── Preserved ──────────────────────────────────────────────────────────

#[test]
fn apply_width_inserts_stack_and_pushes_entry() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, erosion_apply_width);
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.75,
    });
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .expect("EffectStack inserted");
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.75).abs() < 1e-6);
}

#[test]
fn apply_width_reconciles_to_single_entry_across_ticks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, erosion_apply_width);
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.80,
    });
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.80).abs() < 1e-6);
}

#[test]
fn apply_width_preserves_non_erosion_entries() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, erosion_apply_width);
    app.world_mut().insert_resource(ErosionState {
        width_fraction: 0.80,
    });

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        SourceId::chip("heavy").rarity(Rarity::Common).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    let breaker = app.world_mut().spawn((Breaker, seed)).id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    // 1.25 (chip) * 0.80 (erosion) = 1.0 aggregate
    assert_eq!(stack.len(), 2);
    assert!((stack.aggregate() - 1.0).abs() < 1e-5);
}

// ── C18 — two Breakers each get independent EROSION_SOURCE entries ─────

#[test]
fn two_breakers_each_get_independent_erosion_entry() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.70);
    let b1 = spawn_breaker(&mut app);
    let b2 = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let stack1 = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(b1)
            .expect("breaker 1 gets a stack");
        let stack2 = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(b2)
            .expect("breaker 2 gets a stack");
        assert_eq!(stack1.len(), 1);
        assert_eq!(stack2.len(), 1);
        assert!((stack1.aggregate() - 0.70).abs() < 1e-6);
        assert!((stack2.aggregate() - 0.70).abs() < 1e-6);
    }

    // Edge: despawn one Breaker — survivor remains clean across next tick.
    app.world_mut().entity_mut(b2).despawn();
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let stack1 = app.world().get::<EffectStack<SizeBoostConfig>>(b1).unwrap();
    assert_eq!(stack1.len(), 1);
    assert!((stack1.aggregate() - 0.70).abs() < 1e-6);
}

// ── C19 — stale EROSION_SOURCE entry is replaced, not duplicated ───────

#[test]
fn stale_erosion_source_entry_is_replaced_not_duplicated() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.80);

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        SourceId::hazard(HazardKind::Erosion).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(9.99),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1);
        let entries = erosion_entries(stack);
        assert_eq!(entries[0].0, SourceId::hazard(HazardKind::Erosion).build());
        let m = entries[0].1.multiplier.into_inner();
        assert!(
            (m - 0.80_f32).abs() < 1e-6,
            "expected multiplier ≈ 0.80, got {m}"
        );
        // Tripwire: aggregate must be 0.80, NOT 0.80 * 9.99 ≈ 7.992.
        assert!((stack.aggregate() - 0.80).abs() < 1e-6);
    }

    // Edge: a second tick; still len == 1, aggregate still ≈ 0.80.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.80).abs() < 1e-6);
}

// ── C20 — two stale EROSION entries reduce to one fresh entry ──────────

#[test]
fn two_stale_erosion_entries_reduce_to_one_fresh_entry() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.80);

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        SourceId::hazard(HazardKind::Erosion).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(5.0),
        },
    );
    seed.push(
        SourceId::hazard(HazardKind::Erosion).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(3.0),
        },
    );
    seed.push(
        SourceId::chip("heavy").rarity(Rarity::Common).build(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 2, "one erosion + one chip entry");
    // 1.25 * 0.80 = 1.0
    assert!((stack.aggregate() - 1.0).abs() < 1e-5);

    // Edge: iterate entries — exactly one Erosion, exactly one chip with seeded multiplier.
    let entries = erosion_entries(stack);
    let erosion_src = SourceId::hazard(HazardKind::Erosion).build();
    let chip_src = SourceId::chip("heavy").rarity(Rarity::Common).build();
    let erosion_count = entries
        .iter()
        .filter(|(src, _)| src == &erosion_src)
        .count();
    let chip_count = entries.iter().filter(|(src, _)| src == &chip_src).count();
    assert_eq!(erosion_count, 1);
    assert_eq!(chip_count, 1);
    let chip = entries.iter().find(|(src, _)| src == &chip_src).unwrap();
    assert_eq!(chip.1.multiplier, OrderedFloat(1.25_f32));
}

// ── C21 — width_fraction change between ticks updates entry ───────────

#[test]
fn width_fraction_change_between_ticks_updates_erosion_entry() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.75);
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Mutate ErosionState directly (simulating what erosion_shrink would do).
    app.world_mut()
        .resource_mut::<ErosionState>()
        .width_fraction = 0.50;
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.50).abs() < 1e-6);
        let entries = erosion_entries(stack);
        let m = entries[0].1.multiplier.into_inner();
        assert!((m - 0.50_f32).abs() < 1e-6);
    }

    // Edge: identity-multiplier case still updates the entry rather than removing it.
    app.world_mut()
        .resource_mut::<ErosionState>()
        .width_fraction = 1.0;
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.0).abs() < 1e-6);
}

// ── C22 — no Breaker in world: system runs cleanly ────────────────────

#[test]
fn no_breaker_in_world_runs_cleanly_across_many_ticks() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.75);
    // No Breaker spawned.

    // Main: single tick runs cleanly.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let count = app
            .world_mut()
            .query::<&EffectStack<SizeBoostConfig>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    // Edge: five consecutive ticks still produce no stacks.
    for _ in 0..5 {
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    }
    let count = app
        .world_mut()
        .query::<&EffectStack<SizeBoostConfig>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0);
    let state = app.world().resource::<ErosionState>();
    assert!(
        (state.width_fraction - 0.75).abs() < f32::EPSILON,
        "width_fraction should be 0.75"
    );
}

// ── C23 — Breaker with pre-existing empty EffectStack ─────────────────

#[test]
fn breaker_with_preexisting_empty_stack_reconciles_idempotently() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.70);
    let breaker = spawn_breaker_with_stack(&mut app, EffectStack::<SizeBoostConfig>::default());

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.70).abs() < 1e-6);
    }

    // Edge: a second tick is idempotent.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 0.70).abs() < 1e-6);
}

// ── C24 — Breaker query filters out non-Breaker entities ──────────────

#[test]
fn non_breaker_entity_is_filtered_out_of_query() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.70);
    let breaker = spawn_breaker(&mut app);
    let non_breaker = app.world_mut().spawn_empty().id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(
        app.world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .is_some(),
        "Breaker must receive an EffectStack"
    );
    assert!(
        app.world()
            .get::<EffectStack<SizeBoostConfig>>(non_breaker)
            .is_none(),
        "non-Breaker entities must not receive an EffectStack"
    );
    let count = app
        .world_mut()
        .query::<&EffectStack<SizeBoostConfig>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

// ── MB2 — no-state guard (Option<Res<ErosionState>> absent) ───────────

#[test]
fn no_state_guard_runs_cleanly_and_does_not_insert_effect_stack() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    // Intentionally no install_erosion_state.
    let breaker = spawn_breaker(&mut app);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        assert!(
            app.world()
                .get::<EffectStack<SizeBoostConfig>>(breaker)
                .is_none(),
            "system should early-return when ErosionState is absent"
        );
    }

    // Edge: five ticks without ErosionState — still no panic, still no stack.
    for _ in 0..5 {
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    }
    assert!(
        app.world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .is_none()
    );
}
