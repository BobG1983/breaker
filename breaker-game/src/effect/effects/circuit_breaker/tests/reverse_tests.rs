//! Tests for `reverse()` and `register()` behavior.

use super::helpers::*;

// -- Behavior 8: reverse() removes CircuitBreakerCounter --

#[test]
fn reverse_removes_circuit_breaker_counter() {
    let mut world = World::new();
    let entity = world
        .spawn(CircuitBreakerCounter {
            remaining: 2,
            bumps_required: 3,
            spawn_count: 1,
            inherit: true,
            shockwave_range: 160.0,
            shockwave_speed: 500.0,
        })
        .id();

    reverse(entity, "circuit_breaker", &mut world);

    assert!(
        world.get::<CircuitBreakerCounter>(entity).is_none(),
        "CircuitBreakerCounter should be removed after reverse()"
    );
}

// -- Behavior 8 edge case: reverse on entity without counter is a no-op --

#[test]
fn reverse_on_entity_without_counter_is_noop_no_panic() {
    let mut world = World::new();
    let entity = world
        .spawn(CircuitBreakerCounter {
            remaining: 2,
            bumps_required: 3,
            spawn_count: 1,
            inherit: true,
            shockwave_range: 160.0,
            shockwave_speed: 500.0,
        })
        .id();

    // First: call reverse on entity WITH counter, assert it's removed
    reverse(entity, "circuit_breaker", &mut world);
    assert!(
        world.get::<CircuitBreakerCounter>(entity).is_none(),
        "reverse should remove CircuitBreakerCounter from entity that has one"
    );

    // Then: call reverse AGAIN on same entity (now without counter) -- should not panic
    reverse(entity, "circuit_breaker", &mut world);
    assert!(
        world.get::<CircuitBreakerCounter>(entity).is_none(),
        "entity should still have no CircuitBreakerCounter after second reverse"
    );
}

// -- Behavior 10: register() is a no-op --

#[test]
fn register_is_noop_and_does_not_panic() {
    let mut app = App::new();

    // Should compile and not panic
    register(&mut app);
}
