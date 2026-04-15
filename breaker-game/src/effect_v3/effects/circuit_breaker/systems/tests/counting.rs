use bevy::prelude::*;

use super::helpers::*;
use crate::{
    effect_v3::effects::{
        circuit_breaker::components::CircuitBreakerCounter, shockwave::components::ShockwaveSource,
    },
    shared::test_utils::tick,
};

// ── C11-1: Each BumpPerformed decrements the counter ──

#[test]
fn bump_decrements_counter() {
    let mut app = circuit_breaker_app();

    let entity = spawn_counter(&mut app, 3, 3);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 2,
        "counter should decrement from 3 to 2 after one bump, got {}",
        counter.remaining,
    );
}

#[test]
fn two_bumps_decrement_by_two() {
    let mut app = circuit_breaker_app();

    let entity = spawn_counter(&mut app, 3, 3);
    queue_bump(&mut app);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 1,
        "counter should decrement from 3 to 1 after two bumps, got {}",
        counter.remaining,
    );
}

// ── C11-2: Counter reaching zero fires reward and resets ──

#[test]
fn counter_reaching_zero_resets_to_bumps_required() {
    let mut app = circuit_breaker_app();

    let entity = spawn_counter(&mut app, 1, 3);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 3,
        "counter should reset to bumps_required (3) after reaching zero, got {}",
        counter.remaining,
    );
}

#[test]
fn counter_reaching_zero_dispatches_shockwave() {
    let mut app = circuit_breaker_app();

    spawn_counter(&mut app, 1, 3);
    queue_bump(&mut app);

    tick(&mut app);

    // Verify a ShockwaveSource entity was spawned by fire_dispatch
    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert!(
        shockwave_count > 0,
        "shockwave should be spawned when circuit breaker fires, got 0",
    );
}

#[test]
fn bumps_required_one_fires_every_bump() {
    let mut app = circuit_breaker_app();

    let entity = app
        .world_mut()
        .spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  1,
            spawn_count:     2,
            inherit:         false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
        })
        .id();
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 1,
        "bumps_required=1: single bump should fire and reset to 1, got {}",
        counter.remaining,
    );
}

// ── C11-3: Multiple entities all decrement on same bump ──

#[test]
fn multiple_entities_each_decrement_on_same_bump() {
    let mut app = circuit_breaker_app();

    let entity_a = spawn_counter(&mut app, 2, 2);
    let entity_b = app
        .world_mut()
        .spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  3,
            spawn_count:     1,
            inherit:         false,
            shockwave_range: 50.0,
            shockwave_speed: 150.0,
        })
        .id();

    queue_bump(&mut app);

    tick(&mut app);

    let counter_a = app.world().get::<CircuitBreakerCounter>(entity_a).unwrap();
    assert_eq!(
        counter_a.remaining, 1,
        "entity A should decrement from 2 to 1, got {}",
        counter_a.remaining,
    );

    let counter_b = app.world().get::<CircuitBreakerCounter>(entity_b).unwrap();
    assert_eq!(
        counter_b.remaining, 3,
        "entity B should reset to 3 after reaching zero, got {}",
        counter_b.remaining,
    );
}

// ── C11-4: No BumpPerformed — counter unchanged ──

#[test]
fn no_bumps_leaves_counter_unchanged() {
    let mut app = circuit_breaker_app();

    let entity = spawn_counter(&mut app, 2, 3);

    tick(&mut app);

    let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 2,
        "counter should remain 2 with no bumps, got {}",
        counter.remaining,
    );
}

#[test]
fn bump_with_no_counter_entities_does_not_panic() {
    let mut app = circuit_breaker_app();

    queue_bump(&mut app);

    // Should not panic
    tick(&mut app);
}

// ── C11-5: Counter wrapping — exactly enough bumps to fire twice ──

#[test]
fn three_bumps_fire_twice_with_bumps_required_two_remaining_one() {
    let mut app = circuit_breaker_app();

    let entity = app
        .world_mut()
        .spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  2,
            spawn_count:     1,
            inherit:         false,
            shockwave_range: 50.0,
            shockwave_speed: 150.0,
        })
        .id();

    queue_bump(&mut app);
    queue_bump(&mut app);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
    // 1st bump: remaining 1->0, fire, reset to 2
    // 2nd bump: remaining 2->1
    // 3rd bump: remaining 1->0, fire, reset to 2
    // Final: 2
    assert_eq!(
        counter.remaining, 2,
        "after 3 bumps (fire twice), remaining should be 2, got {}",
        counter.remaining,
    );

    // Should have 2 shockwave entities
    let shockwave_count = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>()
        .iter(app.world())
        .count();
    assert_eq!(
        shockwave_count, 2,
        "should have dispatched 2 shockwaves, got {shockwave_count}",
    );
}

#[test]
fn bumps_required_one_with_two_bumps_fires_twice() {
    let mut app = circuit_breaker_app();

    let entity = app
        .world_mut()
        .spawn(CircuitBreakerCounter {
            remaining:       1,
            bumps_required:  1,
            spawn_count:     1,
            inherit:         false,
            shockwave_range: 50.0,
            shockwave_speed: 150.0,
        })
        .id();

    queue_bump(&mut app);
    queue_bump(&mut app);

    tick(&mut app);

    let counter = app.world().get::<CircuitBreakerCounter>(entity).unwrap();
    // Each bump fires and resets: final remaining = 1
    assert_eq!(
        counter.remaining, 1,
        "bumps_required=1 with 2 bumps: fires twice, final remaining should be 1, got {}",
        counter.remaining,
    );
}
