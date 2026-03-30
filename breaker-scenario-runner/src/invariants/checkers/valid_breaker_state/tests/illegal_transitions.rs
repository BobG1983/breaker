use breaker::breaker::components::BreakerState;

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// `Idle -> Braking` is illegal (must go through `Dashing`). The system must
/// append a [`ViolationEntry`] with [`InvariantKind::ValidBreakerState`].
///
/// Tick 1 seeds `Local` with `Idle`. Tick 2 sees `Braking` -> violation.
#[test]
fn valid_breaker_state_fires_on_idle_to_braking() {
    let mut app = test_app_valid_breaker_state();

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    // Tick 1: system stores Idle in Local, no previous to compare → no violation
    tick(&mut app);

    let log_after_tick1 = app.world().resource::<ViolationLog>();
    assert!(
        log_after_tick1.0.is_empty(),
        "no violation expected on first tick (no previous state)"
    );

    // Mutate to Braking (illegal: Idle → Braking)
    *app.world_mut()
        .entity_mut(entity)
        .get_mut::<BreakerState>()
        .unwrap() = BreakerState::Braking;

    // Tick 2: system compares Braking vs previous Idle → should fire
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly one ValidBreakerState violation, got {}",
        log.0.len()
    );
    assert_eq!(log.0[0].invariant, InvariantKind::ValidBreakerState);
}
