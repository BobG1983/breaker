use bevy::prelude::*;
use breaker::breaker::components::BreakerState;

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// Two [`ScenarioTagBreaker`] entities are tracked independently. When entity A
/// makes a legal transition (`Idle -> Dashing`) and entity B makes an illegal
/// transition (`Idle -> Braking`), exactly one violation fires — for entity B.
#[test]
fn valid_breaker_state_tracks_two_breakers_independently_one_illegal() {
    let mut app = test_app_valid_breaker_state();

    // Spawn entity A and entity B, both starting Idle
    let entity_a = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();
    let entity_b = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    // Tick 1: seeds Local for both A (Idle) and B (Idle)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick (no previous state to compare)"
    );

    // Entity A: Idle → Dashing (legal)
    *app.world_mut()
        .entity_mut(entity_a)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Dashing;

    // Entity B: Idle → Braking (illegal — skips Dashing)
    *app.world_mut()
        .entity_mut(entity_b)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Braking;

    // Tick 2: A is legal, B is illegal → exactly 1 violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly 1 ValidBreakerState violation (entity B's Idle->Braking is illegal), got {}",
        log.0.len()
    );
    assert_eq!(log.0[0].invariant, InvariantKind::ValidBreakerState);
}

/// When both [`ScenarioTagBreaker`] entities make legal transitions
/// (`Idle -> Dashing`), no [`ViolationEntry`] should be recorded.
#[test]
fn valid_breaker_state_produces_no_violation_when_both_breakers_transition_legally() {
    let mut app = test_app_valid_breaker_state();

    // Spawn two breakers, both Idle
    let entity_a = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();
    let entity_b = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    // Tick 1: seeds Local for A=Idle, B=Idle
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected on seeding tick"
    );

    // Both transition Idle → Dashing (legal)
    *app.world_mut()
        .entity_mut(entity_a)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Dashing;
    *app.world_mut()
        .entity_mut(entity_b)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Dashing;

    // Tick 2: both legal → no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no ValidBreakerState violation when both breakers transition Idle->Dashing (legal), got: {:?}",
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}
