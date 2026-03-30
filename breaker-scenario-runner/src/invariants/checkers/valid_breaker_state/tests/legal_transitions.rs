use bevy::prelude::*;
use breaker::breaker::components::BreakerState;

use super::helpers::*;
use crate::invariants::*;

/// `Idle -> Dashing` is a legal transition. No violation should be recorded.
#[test]
fn valid_breaker_state_does_not_fire_on_idle_to_dashing() {
    let mut app = test_app_valid_breaker_state();

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    // Tick 1: seeds Local with Idle
    tick(&mut app);

    // Change to Dashing (legal: Idle → Dashing)
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Dashing;

    // Tick 2: should NOT fire
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation for Idle->Dashing (legal), got: {:?}",
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

/// `Settling -> Dashing` is legal (breaker can re-dash from settling).
/// No violation should be recorded.
#[test]
fn valid_breaker_state_does_not_fire_on_settling_to_dashing() {
    let mut app = test_app_valid_breaker_state();

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Settling))
        .id();

    // Tick 1: seeds Local with Settling
    tick(&mut app);

    // Transition to Dashing (legal: Settling → Dashing)
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Dashing;

    // Tick 2: Settling → Dashing is legal → no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation for Settling->Dashing (legal), got: {:?}",
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

/// When the state does not change (`Idle -> Idle`), no violation should fire.
#[test]
fn valid_breaker_state_does_not_fire_on_no_state_change() {
    let mut app = test_app_valid_breaker_state();

    app.world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle));

    // Tick 1: seeds Local
    tick(&mut app);
    // Tick 2: same state
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when state does not change"
    );
}

/// On the very first tick (no previous state stored in `Local`), the system must
/// not fire even for `Dashing` — there is no prior state to compare.
#[test]
fn valid_breaker_state_skips_first_frame_with_no_previous() {
    let mut app = test_app_valid_breaker_state();

    // Start directly in Dashing (would be illegal from Idle, but first frame only)
    app.world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Dashing));

    // Only one tick — Local starts empty, no comparison possible
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation on first frame (Local has no previous)"
    );
}

/// `Braking -> Settling` is the natural deceleration completion. It should be
/// legal and produce no violation.
#[test]
fn valid_breaker_state_does_not_fire_on_braking_to_settling() {
    let mut app = test_app_valid_breaker_state();

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Braking))
        .id();

    // Tick 1: seeds Local with Braking
    tick(&mut app);

    // Transition to Settling (legal: Braking → Settling)
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Settling;

    // Tick 2: Braking → Settling should be legal
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation for Braking->Settling (legal), got: {:?}",
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}
