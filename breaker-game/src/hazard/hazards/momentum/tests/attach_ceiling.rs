//! Group B' — `attach_momentum_ceiling` (Behaviors 61–67, 70).
//!
//! Pins the `hp.max`-lifting contract: on every eligible cell,
//! `hp.max = max(existing, starting * 2.0)`. Idempotent (no `Changed<Hp>`
//! churn). Skips `Dead`/`Invulnerable`. Run-if gated on
//! `hazard_active(Momentum)` AND `in_state(NodeState::Playing)`. Reacts when
//! gates flip mid-run. Defensive: skips `hp.starting == 0.0` cells.

use bevy::prelude::*;

use super::{
    super::system::{attach_momentum_ceiling, register},
    helpers::{
        add_hazard_stacks, add_momentum_stacks, canonical_momentum_config, install_momentum_config,
        run_fixed_update, spawn_cell_at, spawn_cell_at_with_max, spawn_cell_dead_at,
        spawn_cell_invulnerable_at, test_app_not_playing, test_app_playing,
    },
};
use crate::{
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::{Hp, *},
};

// ── Behavior 61 — inserts hp.max = Some(2 * starting) on fresh cell ─────────

#[test]
fn inserts_hp_max_two_times_starting_on_fresh_cell() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "hp.max must be Some(starting * 2.0) = Some(20.0); got {:?}",
        hp.max
    );
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
    assert!((hp.starting - 10.0).abs() < f32::EPSILON);
}

#[test]
fn inserts_hp_max_two_times_starting_larger_starting() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 5.0, 50.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(100.0),
        "larger starting (50.0) must yield hp.max == Some(100.0); got {:?}",
        hp.max
    );
}

// ── Behavior 62 — preserves larger existing hp.max (take max) ───────────────

#[test]
fn preserves_larger_existing_hp_max() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 10.0, 10.0, Some(50.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(50.0),
        "existing max (50.0) > target (20.0) must be preserved; got {:?}",
        hp.max
    );
}

#[test]
fn raises_smaller_existing_hp_max_to_target() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 10.0, 10.0, Some(15.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "existing max (15.0) < target (20.0) must be lifted to 20.0; got {:?}",
        hp.max
    );
}

#[test]
fn existing_hp_max_exactly_at_target_is_unchanged() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 10.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "existing max exactly at target must stay at 20.0; got {:?}",
        hp.max
    );
}

// ── Behavior 63 — idempotent: running twice produces no Changed<Hp> ─────────

#[derive(Resource, Default)]
struct ChangedCount(u32);

#[test]
fn second_tick_does_not_retrigger_change_detection() {
    let mut app = test_app_playing();
    app.init_resource::<ChangedCount>();
    // Follower system must be registered BEFORE tick 1 so its `last_run`
    // advances past tick 1. If added after tick 1, its `last_run = 0` and
    // `Changed<Hp>` on tick 2 would also pick up tick 1's legitimate write.
    app.add_systems(
        FixedUpdate,
        (
            attach_momentum_ceiling,
            (|mut counter: ResMut<ChangedCount>, q: Query<(), Changed<Hp>>| {
                counter.0 += q.iter().count() as u32;
            })
            .after(attach_momentum_ceiling),
        ),
    );
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    // First tick — the ceiling is lifted; Hp IS flagged Changed this tick.
    // The follower counter records this legitimate write.
    run_fixed_update(&mut app);
    assert_eq!(
        app.world().get::<Hp>(cell).unwrap().max,
        Some(20.0),
        "sanity: first tick lifts hp.max"
    );

    // Reset counter AFTER tick 1 so the idempotency check measures ONLY
    // tick-2 activity. The follower's `last_run` has now advanced past tick 1,
    // so its next `Changed<Hp>` query starts from a clean window.
    app.world_mut().resource_mut::<ChangedCount>().0 = 0;

    // Second tick — attach runs, but should NOT mutate (idempotent).
    run_fixed_update(&mut app);

    let changed = app.world().resource::<ChangedCount>().0;
    assert_eq!(
        changed, 0,
        "second tick must not trigger Changed<Hp> (idempotent guard failed); \
         observed {changed} Changed<Hp> flags"
    );
    assert_eq!(
        app.world().get::<Hp>(cell).unwrap().max,
        Some(20.0),
        "hp.max must remain Some(20.0) on the second tick"
    );
}

// ── Behavior 64 — skips Dead-marked cells ───────────────────────────────────

#[test]
fn skips_dead_marked_cells() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_dead_at(&mut app, Vec2::ZERO, 5.0, 10.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max, None,
        "Dead-marked cell must have its hp.max left untouched (still None); \
         got {:?}",
        hp.max
    );
}

#[test]
fn dead_marked_cell_with_existing_max_is_not_lifted() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // Dead cell with pre-existing hp.max = 5.0 (< target 20.0).
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      Some(5.0),
            },
            KilledBy { killer: None },
            crate::prelude::Dead,
        ))
        .id();

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(5.0),
        "Dead cell's hp.max must not be lifted to target; got {:?}",
        hp.max
    );
}

// ── Behavior 65 — skips Invulnerable cells ──────────────────────────────────

#[test]
fn skips_invulnerable_cells() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_invulnerable_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max, None,
        "Invulnerable cell's hp.max must be left None; got {:?}",
        hp.max
    );
}

#[test]
fn invulnerable_cell_with_existing_max_is_not_overwritten() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp {
                current:  10.0,
                starting: 10.0,
                max:      Some(100.0),
            },
            KilledBy { killer: None },
            crate::prelude::Invulnerable,
        ))
        .id();

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(100.0),
        "Invulnerable cell's hp.max must be left at Some(100.0); got {:?}",
        hp.max
    );
}

// ── Behavior 66 — run-if gate (not active / not playing) blocks mutation ────

#[test]
fn gate_off_when_momentum_zero_stacks_does_not_mutate() {
    let mut app = test_app_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    // Different hazard stacked — Momentum stays at 0.
    add_hazard_stacks(&mut app, HazardKind::Cascade, 1);
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Momentum),
        0
    );

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max, None,
        "hazard_active(Momentum) gate off → hp.max must stay None; got {:?}",
        hp.max
    );
}

#[test]
fn gate_off_when_not_in_playing_does_not_mutate() {
    let mut app = test_app_not_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max, None,
        "in_state(NodeState::Playing) gate off → hp.max must stay None; got {:?}",
        hp.max
    );
}

// ── Behavior 67 — reacts when hazard becomes active mid-run ─────────────────

#[test]
fn reacts_when_hazard_becomes_active_mid_run() {
    let mut app = test_app_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    // No Momentum stacks initially.

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    run_fixed_update(&mut app);
    assert_eq!(
        app.world().get::<Hp>(cell).unwrap().max,
        None,
        "gate still off → hp.max remains None"
    );

    // Activate Momentum mid-run.
    add_momentum_stacks(&mut app, 1);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "mid-run activation must lift pre-existing cell's hp.max next tick; got {:?}",
        hp.max
    );
}

#[test]
fn mid_run_activation_lifts_newly_spawned_cell_ceiling_too() {
    let mut app = test_app_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // Already-active. Spawn a new cell AFTER activation.
    let new_cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(new_cell).unwrap();
    assert_eq!(
        hp.max,
        Some(20.0),
        "newly-spawned cell gets its ceiling lifted on the next tick; got {:?}",
        hp.max
    );
}

// ── Bonus coverage — multiple cells each lifted independently ───────────────

#[test]
fn multiple_cells_lifted_independently_based_on_own_starting() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let a = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0), 10.0, 10.0);
    let b = spawn_cell_at(&mut app, Vec2::new(-100.0, 0.0), 5.0, 50.0);
    let c = spawn_cell_at(&mut app, Vec2::new(0.0, 100.0), 2.0, 7.0);

    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<Hp>(a).unwrap().max, Some(20.0));
    assert_eq!(app.world().get::<Hp>(b).unwrap().max, Some(100.0));
    assert_eq!(app.world().get::<Hp>(c).unwrap().max, Some(14.0));
}

#[test]
fn harness_safe_when_config_absent_does_not_panic() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    add_momentum_stacks(&mut app, 1);
    // NO MomentumConfig inserted.

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 10.0, 10.0);

    run_fixed_update(&mut app);

    // No panic. The cell's max should still be None (no config → no-op).
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(hp.max, None);
}

#[test]
fn harness_safe_when_no_cells_does_not_panic() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    // No cells spawned.
    run_fixed_update(&mut app);
    // Reaching here without panic is the assertion.
}

// ── Behavior 70 — zero-starting guard (no hp.max set when starting == 0) ────

#[test]
fn zero_starting_cell_is_not_touched() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at(&mut app, Vec2::ZERO, 0.0, 0.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max, None,
        "zero-starting cell must not have hp.max set; got {:?}",
        hp.max
    );
}

#[test]
fn zero_starting_cell_with_existing_max_is_preserved() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, attach_momentum_ceiling);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let cell = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 0.0, 0.0, Some(5.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert_eq!(
        hp.max,
        Some(5.0),
        "zero-starting cell's pre-existing hp.max must stay Some(5.0); got {:?}",
        hp.max
    );
}
