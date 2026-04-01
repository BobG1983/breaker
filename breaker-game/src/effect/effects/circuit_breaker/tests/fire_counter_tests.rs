//! Tests for counter decrement and cycle completion.

use super::helpers::*;

// -- Behavior 1: fire() inserts CircuitBreakerCounter with remaining = bumps_required - 1 --

#[test]
fn fire_inserts_counter_with_remaining_bumps_required_minus_one_when_no_counter() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let config = default_config();
    fire(entity, &config, "circuit_breaker", &mut world);

    let counter = world
        .get::<CircuitBreakerCounter>(entity)
        .expect("fire should insert CircuitBreakerCounter when absent");
    assert_eq!(
        counter.remaining, 2,
        "remaining should be bumps_required - 1 = 2, got {}",
        counter.remaining
    );
    assert_eq!(
        counter.bumps_required, 3,
        "bumps_required should be 3, got {}",
        counter.bumps_required
    );
    assert_eq!(
        counter.spawn_count, 1,
        "spawn_count should be 1, got {}",
        counter.spawn_count
    );
    assert!(counter.inherit, "inherit should be true");
    assert!(
        (counter.shockwave_range - 160.0).abs() < f32::EPSILON,
        "shockwave_range should be 160.0, got {}",
        counter.shockwave_range
    );
    assert!(
        (counter.shockwave_speed - 500.0).abs() < f32::EPSILON,
        "shockwave_speed should be 500.0, got {}",
        counter.shockwave_speed
    );
}

// -- Behavior 1 edge case: bumps_required=1 triggers reward immediately --

#[test]
fn fire_with_bumps_required_one_triggers_reward_immediately_and_resets() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

    let config = CircuitBreakerConfig {
        bumps_required: 1,
        spawn_count: 1,
        inherit: false,
        shockwave_range: 100.0,
        shockwave_speed: 400.0,
    };
    fire(entity, &config, "", &mut world);

    // Counter should exist with remaining reset to bumps_required (1)
    let counter = world
        .get::<CircuitBreakerCounter>(entity)
        .expect("counter should exist after immediate trigger + reset");
    assert_eq!(
        counter.remaining, 1,
        "after immediate trigger, remaining should reset to bumps_required (1), got {}",
        counter.remaining
    );
}

// -- Behavior 2: fire() decrements existing counter by 1 --

#[test]
fn fire_decrements_existing_counter_remaining_by_one() {
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

    let config = default_config();
    fire(entity, &config, "circuit_breaker", &mut world);

    let counter = world
        .get::<CircuitBreakerCounter>(entity)
        .expect("counter should still exist after decrement");
    assert_eq!(
        counter.remaining, 1,
        "remaining should be 1 (was 2, decremented by 1), got {}",
        counter.remaining
    );
}

// -- Behavior 2 edge case: config fields remain unchanged --

#[test]
fn fire_decrement_does_not_change_counter_config_fields() {
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

    let config = default_config();
    fire(entity, &config, "circuit_breaker", &mut world);

    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(counter.bumps_required, 3, "bumps_required should remain 3");
    assert_eq!(counter.spawn_count, 1, "spawn_count should remain 1");
    assert!(counter.inherit, "inherit should remain true");
    assert!(
        (counter.shockwave_range - 160.0).abs() < f32::EPSILON,
        "shockwave_range should remain 160.0"
    );
    assert!(
        (counter.shockwave_speed - 500.0).abs() < f32::EPSILON,
        "shockwave_speed should remain 500.0"
    );
}

// -- Behavior 3: fire() with different params does not overwrite existing counter config --

#[test]
fn fire_with_different_params_does_not_overwrite_existing_counter_config() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 50.0)),
            CircuitBreakerCounter {
                remaining: 1,
                bumps_required: 3,
                spawn_count: 1,
                inherit: true,
                shockwave_range: 160.0,
                shockwave_speed: 500.0,
            },
        ))
        .id();

    // Call fire with entirely different parameter values.
    // remaining=1 means this decrement reaches 0, triggering reward + reset.
    // Reset should use the ORIGINAL bumps_required (3), not the new one (5).
    let config = CircuitBreakerConfig {
        bumps_required: 5,
        spawn_count: 4,
        inherit: false,
        shockwave_range: 200.0,
        shockwave_speed: 999.0,
    };
    fire(entity, &config, "different_chip", &mut world);

    let counter = world
        .get::<CircuitBreakerCounter>(entity)
        .expect("counter should still exist");
    // Config fields should be unchanged from the original insert
    assert_eq!(
        counter.bumps_required, 3,
        "bumps_required should remain 3 (not overwritten to 5), got {}",
        counter.bumps_required
    );
    assert_eq!(
        counter.spawn_count, 1,
        "spawn_count should remain 1 (not overwritten to 4), got {}",
        counter.spawn_count
    );
    assert!(
        counter.inherit,
        "inherit should remain true (not overwritten to false)"
    );
    assert!(
        (counter.shockwave_range - 160.0).abs() < f32::EPSILON,
        "shockwave_range should remain 160.0 (not overwritten to 200.0)"
    );
    assert!(
        (counter.shockwave_speed - 500.0).abs() < f32::EPSILON,
        "shockwave_speed should remain 500.0 (not overwritten to 999.0)"
    );
    // remaining was 1 -> decremented to 0 -> reward fired -> reset using ORIGINAL bumps_required (3)
    assert_eq!(
        counter.remaining, 3,
        "remaining should reset to original bumps_required (3), not the new params value (5), got {}",
        counter.remaining
    );
}

// -- Behavior 4: full countdown with bumps_required=3 takes exactly 3 calls --

#[test]
fn fire_full_countdown_bumps_required_3_takes_three_calls_to_trigger() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    let config = default_config();

    // Call 1: inserts counter with remaining = 2
    fire(entity, &config, "circuit_breaker", &mut world);
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 2,
        "after call 1: remaining should be 2, got {}",
        counter.remaining
    );

    // Call 2: decrements to remaining = 1
    fire(entity, &config, "circuit_breaker", &mut world);
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 1,
        "after call 2: remaining should be 1, got {}",
        counter.remaining
    );

    // Call 3: remaining reaches 0, reward fires, counter resets to bumps_required (3)
    fire(entity, &config, "circuit_breaker", &mut world);
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 3,
        "after call 3: remaining should reset to bumps_required (3), got {}",
        counter.remaining
    );
}

// -- Behavior 4 edge case: after 4th call, remaining = 2 again --

#[test]
fn fire_fourth_call_starts_new_cycle_with_remaining_2() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    let config = default_config();

    // Complete first cycle (3 calls)
    fire(entity, &config, "", &mut world);
    fire(entity, &config, "", &mut world);
    fire(entity, &config, "", &mut world);

    // 4th call: new cycle, decrements from 3 to 2
    fire(entity, &config, "", &mut world);
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 2,
        "after call 4: remaining should be 2 (new cycle started), got {}",
        counter.remaining
    );
}

// -- Behavior 9: bumps_required=1 triggers reward on every call --

#[test]
fn fire_with_bumps_required_1_triggers_reward_on_every_call() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

    let config = CircuitBreakerConfig {
        bumps_required: 1,
        spawn_count: 1,
        inherit: false,
        shockwave_range: 100.0,
        shockwave_speed: 400.0,
    };

    // Call 1: reward fires (shockwave + bolt spawned), counter resets to remaining=1
    fire(entity, &config, "", &mut world);

    let mut sw_query = world.query::<&ShockwaveSource>();
    assert_eq!(
        sw_query.iter(&world).count(),
        1,
        "after call 1: expected 1 shockwave entity"
    );
    let mut bolt_query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    assert_eq!(
        bolt_query.iter(&world).count(),
        1,
        "after call 1: expected 1 extra bolt"
    );
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 1,
        "after call 1: remaining should reset to bumps_required (1), got {}",
        counter.remaining
    );

    // Call 2: reward fires again (2nd shockwave + 2nd bolt)
    fire(entity, &config, "", &mut world);

    assert_eq!(
        sw_query.iter(&world).count(),
        2,
        "after call 2: expected 2 shockwave entities"
    );
    assert_eq!(
        bolt_query.iter(&world).count(),
        2,
        "after call 2: expected 2 extra bolts"
    );
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 1,
        "after call 2: remaining should reset to bumps_required (1), got {}",
        counter.remaining
    );
}

// -- Behavior 11: fire() on a despawned entity does not panic --

#[test]
fn fire_on_despawned_entity_does_not_panic() {
    let mut world = World::new();

    let config = default_config();

    // First: fire on a LIVE entity and verify counter IS inserted
    let live_entity = world.spawn_empty().id();
    fire(live_entity, &config, "circuit_breaker", &mut world);
    let counter = world
        .get::<CircuitBreakerCounter>(live_entity)
        .expect("fire on live entity should insert CircuitBreakerCounter");
    assert_eq!(
        counter.remaining, 2,
        "live entity counter remaining should be bumps_required - 1 = 2, got {}",
        counter.remaining
    );

    // Then: fire on a DESPAWNED entity -- should not panic
    let doomed_entity = world.spawn_empty().id();
    world.despawn(doomed_entity);
    fire(doomed_entity, &config, "circuit_breaker", &mut world);

    // The live entity's counter should still be at remaining=2 (unchanged)
    let counter = world
        .get::<CircuitBreakerCounter>(live_entity)
        .expect("live entity counter should still exist");
    assert_eq!(
        counter.remaining, 2,
        "live entity counter should be unchanged at 2 after fire on despawned entity, got {}",
        counter.remaining
    );

    // Only 1 counter total should exist in the world (the live entity's)
    let mut query = world.query::<&CircuitBreakerCounter>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 1,
        "only the live entity should have a CircuitBreakerCounter, got {count} total"
    );
}
