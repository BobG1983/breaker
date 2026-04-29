//! Behaviors 1–5: unit tests for `update_previous_dash_state`.
//!
//! These tests build a minimal `App` with the system registered in `Update`
//! (one `app.update()` == one run of the system) and do NOT use
//! `BreakerPlugin` — they validate the system's body in isolation.

use bevy::prelude::*;

use super::super::update_previous_dash_state;
use crate::breaker::components::{DashState, PreviousDashState};

/// Builds a minimal `App` that registers `update_previous_dash_state` in
/// `Update`. Each `app.update()` runs the system exactly once.
fn update_previous_dash_state_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, update_previous_dash_state);
    app
}

// ── Behavior 1: System copies current DashState into PreviousDashState on first run ──

#[test]
fn updates_previous_to_dashing_after_one_tick() {
    let mut app = update_previous_dash_state_test_app();

    // Spawn an entity with DashState::Idle and PreviousDashState::default()
    // (which equals PreviousDashState(DashState::Idle)).
    let entity = app
        .world_mut()
        .spawn((DashState::Idle, PreviousDashState::default()))
        .id();

    // Mutate DashState to Dashing BEFORE the system runs.
    *app.world_mut()
        .get_mut::<DashState>(entity)
        .expect("entity should have DashState") = DashState::Dashing;

    app.update();

    let prev = app
        .world()
        .get::<PreviousDashState>(entity)
        .expect("entity should still have PreviousDashState");
    assert_eq!(
        prev.0,
        DashState::Dashing,
        "PreviousDashState.0 should be Dashing after one tick, got {:?}",
        prev.0,
    );

    // Edge case: DashState itself must not have been mutated by the system.
    let state = app
        .world()
        .get::<DashState>(entity)
        .expect("entity should still have DashState");
    assert_eq!(
        *state,
        DashState::Dashing,
        "DashState should remain Dashing (system must not mutate it), got {state:?}",
    );
}

// ── Behavior 2: System updates PreviousDashState across multiple ticks
//                 (state walks Idle → Dashing → Idle, then idempotent third tick) ──

#[test]
fn walks_dashing_then_idle_across_two_ticks() {
    let mut app = update_previous_dash_state_test_app();

    let entity = app
        .world_mut()
        .spawn((DashState::Idle, PreviousDashState::default()))
        .id();

    // Tick 1: mutate to Dashing, run system.
    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Dashing;
    app.update();
    assert_eq!(
        app.world().get::<PreviousDashState>(entity).unwrap().0,
        DashState::Dashing,
        "after tick 1, PreviousDashState.0 should be Dashing",
    );

    // Tick 2: mutate back to Idle, run system.
    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Idle;
    app.update();
    assert_eq!(
        app.world().get::<PreviousDashState>(entity).unwrap().0,
        DashState::Idle,
        "after tick 2, PreviousDashState.0 should be Idle",
    );

    // Edge case: tick 3 — leave DashState untouched at Idle, run system again.
    // Both PreviousDashState and DashState must remain Idle (idempotent).
    app.update();
    assert_eq!(
        app.world().get::<PreviousDashState>(entity).unwrap().0,
        DashState::Idle,
        "after idempotent tick 3, PreviousDashState.0 should stay Idle",
    );
    assert_eq!(
        *app.world().get::<DashState>(entity).unwrap(),
        DashState::Idle,
        "after idempotent tick 3, DashState should stay Idle (system must not mutate it)",
    );
}

// ── Behavior 3: System leaves PreviousDashState unchanged when DashState equals it ──

#[test]
fn idempotent_when_state_unchanged_for_settling() {
    let mut app = update_previous_dash_state_test_app();

    let entity = app
        .world_mut()
        .spawn((DashState::Settling, PreviousDashState(DashState::Dashing)))
        .id();

    app.update();

    let prev = app.world().get::<PreviousDashState>(entity).unwrap();
    assert_eq!(
        prev.0,
        DashState::Settling,
        "PreviousDashState.0 should remain Settling, got {:?}",
        prev.0,
    );
    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(
        *state,
        DashState::Settling,
        "DashState should remain Settling, got {state:?}",
    );
}

// ── Behavior 4: System updates PreviousDashState independently for multiple breakers ──

#[test]
fn updates_three_breakers_independently_in_one_tick() {
    let mut app = update_previous_dash_state_test_app();

    let entity_a = app
        .world_mut()
        .spawn((DashState::Idle, PreviousDashState(DashState::Idle)))
        .id();
    let entity_b = app
        .world_mut()
        .spawn((DashState::Dashing, PreviousDashState(DashState::Idle)))
        .id();
    let entity_c = app
        .world_mut()
        .spawn((DashState::Braking, PreviousDashState(DashState::Dashing)))
        .id();

    app.update();

    assert_eq!(
        app.world().get::<PreviousDashState>(entity_a).unwrap().0,
        DashState::Idle,
        "A's PreviousDashState.0 should be Idle",
    );
    assert_eq!(
        app.world().get::<PreviousDashState>(entity_b).unwrap().0,
        DashState::Dashing,
        "B's PreviousDashState.0 should be Dashing",
    );
    assert_eq!(
        app.world().get::<PreviousDashState>(entity_c).unwrap().0,
        DashState::Braking,
        "C's PreviousDashState.0 should be Braking",
    );

    // Edge case: exactly 3 entities with PreviousDashState — system must not
    // spawn or despawn anything.
    let mut query = app.world_mut().query::<&PreviousDashState>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 3,
        "expected exactly 3 entities with PreviousDashState, got {count}",
    );
}

// ── Behavior 5: System ignores entities missing PreviousDashState (Query filter) ──

#[test]
fn ignores_entity_missing_previous_dash_state() {
    let mut app = update_previous_dash_state_test_app();

    // Entity A: DashState only — no PreviousDashState. The system must not
    // panic and must not insert one.
    let entity_a = app.world_mut().spawn(DashState::Dashing).id();

    // Entity B: both components, current Braking, previous Idle.
    let entity_b = app
        .world_mut()
        .spawn((DashState::Braking, PreviousDashState(DashState::Idle)))
        .id();

    app.update();

    // A: DashState unchanged, no PreviousDashState inserted.
    assert_eq!(
        *app.world().get::<DashState>(entity_a).unwrap(),
        DashState::Dashing,
        "entity A's DashState should remain Dashing",
    );
    assert!(
        app.world().get::<PreviousDashState>(entity_a).is_none(),
        "entity A must NOT have a PreviousDashState component (system must not insert one)",
    );

    // B: PreviousDashState updated to current.
    assert_eq!(
        app.world().get::<PreviousDashState>(entity_b).unwrap().0,
        DashState::Braking,
        "entity B's PreviousDashState.0 should be Braking",
    );
}
