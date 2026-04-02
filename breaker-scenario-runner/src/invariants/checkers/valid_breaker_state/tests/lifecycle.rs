use bevy::prelude::*;
use breaker::{breaker::components::DashState, shared::GameState};

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// Spawn a breaker with `DashState::Braking`, tick once to seed the Local
/// `HashMap`, despawn the entity, then spawn a new breaker with
/// `DashState::Idle`. If the `HashMap` is not cleaned up on despawn, the
/// new entity (which may recycle the ID) will be compared against the stale
/// `Braking` entry and fire a false `ValidDashState` violation.
///
/// After the despawn+respawn cycle, no violation must fire.
#[test]
fn valid_breaker_state_no_violation_after_despawn_and_respawn() {
    let mut app = test_app_valid_breaker_state();

    // Spawn first breaker in Braking state
    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, DashState::Braking))
        .id();

    // Tick 1: system inserts entity → DashState::Braking into Local HashMap
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected on first tick (no previous state to compare)"
    );

    // Despawn the breaker — system must remove it from Local HashMap
    app.world_mut().entity_mut(entity).despawn();

    // Tick 2: entity is gone; system should prune stale HashMap entries
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected when tagged entity is despawned"
    );

    // Spawn a new breaker with Idle state — may receive a recycled entity ID
    app.world_mut().spawn((ScenarioTagBreaker, DashState::Idle));

    // Tick 3: new entity appears for first time — no previous state in HashMap → no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::ValidDashState),
        "expected no ValidDashState violation after despawn+respawn cycle, \
        got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::ValidDashState)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// `Dashing -> Settling` is the dash-cancel transition triggered by a perfect
/// bump. It should be legal and produce no violation.
#[test]
fn valid_breaker_state_does_not_fire_on_dashing_to_settling_dash_cancel() {
    let mut app = test_app_valid_breaker_state();

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, DashState::Dashing))
        .id();

    // Tick 1: seeds Local with Dashing
    tick(&mut app);

    // Transition to Settling (dash cancel — legal)
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<DashState>()
        .unwrap() = DashState::Settling;

    // Tick 2: Dashing → Settling should be legal
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation for Dashing->Settling (dash cancel is legal), got: {:?}",
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

/// When `GameState` transitions (e.g., re-entering `Playing` for a new node),
/// the breaker state tracker clears. A breaker that was `Braking` before the
/// transition and is now `Idle` (from `reset_breaker`) should not fire.
#[test]
fn valid_breaker_state_clears_tracking_on_game_state_transition() {
    let mut app = test_app_valid_breaker_state();

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, DashState::Braking))
        .id();

    // Tick 1: seeds tracking with Braking (GameState starts at Loading)
    tick(&mut app);

    assert!(app.world().resource::<ViolationLog>().0.is_empty());

    // Simulate node transition: change GameState so the tracker clears
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    app.update(); // process state transition

    // Change breaker to Idle (what reset_breaker does)
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<DashState>()
        .unwrap() = DashState::Idle;

    // Tick 2: GameState changed Loading→MainMenu → tracking was cleared
    // → Idle is treated as first frame, no comparison → no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation after GameState transition clears tracking, got: {:?}",
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}
