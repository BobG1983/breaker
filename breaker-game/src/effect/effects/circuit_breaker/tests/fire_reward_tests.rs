//! Tests for reward spawning (bolts, shockwave, effects).

use super::helpers::*;

// -- Behavior 5: fire() at remaining=1 fires SpawnBolts and Shockwave rewards then resets --

#[test]
fn fire_at_remaining_1_spawns_bolts_and_shockwave_then_resets_counter() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::new(100.0, 200.0)),
            CircuitBreakerCounter {
                remaining: 1,
                bumps_required: 3,
                spawn_count: 2,
                inherit: true,
                shockwave_range: 160.0,
                shockwave_speed: 500.0,
            },
        ))
        .id();

    let config = CircuitBreakerConfig {
        bumps_required: 3,
        spawn_count: 2,
        inherit: true,
        shockwave_range: 160.0,
        shockwave_speed: 500.0,
    };
    fire(entity, &config, "circuit_breaker", &mut world);

    // (a) Check spawned bolts
    let mut bolt_query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt_count = bolt_query.iter(&world).count();
    assert_eq!(
        bolt_count, 2,
        "expected 2 extra bolts spawned (spawn_count=2), got {bolt_count}"
    );

    // (b) Check shockwave entity
    let mut sw_query = world.query::<(
        &ShockwaveSource,
        &ShockwaveMaxRadius,
        &ShockwaveSpeed,
        &Position2D,
    )>();
    let shockwaves: Vec<_> = sw_query.iter(&world).collect();
    assert_eq!(
        shockwaves.len(),
        1,
        "expected 1 shockwave entity spawned, got {}",
        shockwaves.len()
    );
    let (_source, max_radius, speed, pos) = shockwaves[0];
    assert!(
        (max_radius.0 - 160.0).abs() < f32::EPSILON,
        "shockwave max radius should be 160.0, got {}",
        max_radius.0
    );
    assert!(
        (speed.0 - 500.0).abs() < f32::EPSILON,
        "shockwave speed should be 500.0, got {}",
        speed.0
    );
    assert!(
        (pos.0.x - 100.0).abs() < f32::EPSILON,
        "shockwave position x should be 100.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 200.0).abs() < f32::EPSILON,
        "shockwave position y should be 200.0, got {}",
        pos.0.y
    );

    // (c) Check counter reset
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 3,
        "counter remaining should reset to bumps_required (3), got {}",
        counter.remaining
    );
}

// -- Behavior 5 edge case: spawn_count=0 still fires shockwave and resets counter --

#[test]
fn fire_at_remaining_1_with_spawn_count_zero_fires_shockwave_but_no_bolts() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::new(100.0, 200.0)),
            CircuitBreakerCounter {
                remaining: 1,
                bumps_required: 3,
                spawn_count: 0,
                inherit: true,
                shockwave_range: 160.0,
                shockwave_speed: 500.0,
            },
        ))
        .id();

    let config = CircuitBreakerConfig {
        bumps_required: 3,
        spawn_count: 0,
        inherit: true,
        shockwave_range: 160.0,
        shockwave_speed: 500.0,
    };
    fire(entity, &config, "circuit_breaker", &mut world);

    // No bolts spawned
    let mut bolt_query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt_count = bolt_query.iter(&world).count();
    assert_eq!(
        bolt_count, 0,
        "spawn_count=0 should spawn no bolts, got {bolt_count}"
    );

    // Shockwave still fires
    let mut sw_query = world.query::<&ShockwaveSource>();
    let sw_count = sw_query.iter(&world).count();
    assert_eq!(
        sw_count, 1,
        "shockwave should still fire even with spawn_count=0, got {sw_count}"
    );

    // Counter resets
    let counter = world.get::<CircuitBreakerCounter>(entity).unwrap();
    assert_eq!(
        counter.remaining, 3,
        "counter should still reset to bumps_required (3), got {}",
        counter.remaining
    );
}

// -- Behavior 6: shockwave uses EffectSourceChip from fire() call --

#[test]
fn fire_reward_spawns_shockwave_with_effect_source_chip() {
    let mut world = world_with_bolt_config();
    // Give the source entity BoundEffects -- inherit=false should prevent copying
    let bound = BoundEffects(vec![(
        "should_not_copy".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(5.0)),
    )]);
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            bound,
            CircuitBreakerCounter {
                remaining: 1,
                bumps_required: 3,
                spawn_count: 1,
                inherit: false,
                shockwave_range: 80.0,
                shockwave_speed: 300.0,
            },
        ))
        .id();

    let config = CircuitBreakerConfig {
        bumps_required: 3,
        spawn_count: 1,
        inherit: false,
        shockwave_range: 80.0,
        shockwave_speed: 300.0,
    };
    fire(entity, &config, "circuit_breaker", &mut world);

    // (a) Check shockwave has correct EffectSourceChip
    let mut query = world.query::<(&ShockwaveSource, &EffectSourceChip)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected 1 shockwave with EffectSourceChip"
    );
    let (_source, esc) = results[0];
    assert_eq!(
        esc.0,
        Some("circuit_breaker".to_string()),
        "shockwave should have EffectSourceChip(Some(\"circuit_breaker\")), got {:?}",
        esc.0
    );

    // (b) Verify inherit=false: spawned bolts should NOT have BoundEffects from source
    let mut bolt_query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = bolt_query.iter(&world).collect();
    assert_eq!(bolts.len(), 1, "expected 1 spawned bolt");
    let bolt_effects = world.get::<BoundEffects>(bolts[0]);
    assert!(
        bolt_effects.is_none_or(|e| e.0.is_empty()),
        "spawned bolt should NOT have BoundEffects when inherit=false, but got {:?}",
        bolt_effects.map(|e| e.0.len())
    );
}

// -- Behavior 6 edge case: empty source_chip produces EffectSourceChip(None) --

#[test]
fn fire_reward_with_empty_source_chip_produces_none_on_shockwave() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            CircuitBreakerCounter {
                remaining: 1,
                bumps_required: 3,
                spawn_count: 1,
                inherit: false,
                shockwave_range: 80.0,
                shockwave_speed: 300.0,
            },
        ))
        .id();

    let config = CircuitBreakerConfig {
        bumps_required: 3,
        spawn_count: 1,
        inherit: false,
        shockwave_range: 80.0,
        shockwave_speed: 300.0,
    };
    fire(entity, &config, "", &mut world);

    let mut query = world.query::<(&ShockwaveSource, &EffectSourceChip)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected 1 shockwave entity");
    let (_source, esc) = results[0];
    assert_eq!(
        esc.0, None,
        "empty source_chip should produce EffectSourceChip(None), got {:?}",
        esc.0
    );
}

// -- Behavior 7: inherit=true clones BoundEffects to spawned bolts --

#[test]
fn fire_reward_with_inherit_true_clones_bound_effects_to_spawned_bolts() {
    let mut world = world_with_bolt_config();
    let bound = BoundEffects(vec![(
        "test".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(2.0)),
    )]);
    // Primary bolt with BoundEffects — SpawnBolts(inherit: true) copies from here
    world.spawn((Bolt, Position2D(Vec2::ZERO), bound));
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
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

    let config = default_config();
    fire(entity, &config, "", &mut world);

    let mut bolt_query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = bolt_query.iter(&world).collect();
    assert_eq!(bolts.len(), 1, "expected 1 spawned bolt");

    let effects = world
        .get::<BoundEffects>(bolts[0])
        .expect("spawned bolt should have BoundEffects when inherit=true");
    assert_eq!(
        effects.0.len(),
        1,
        "BoundEffects should have 1 entry, got {}",
        effects.0.len()
    );
    assert_eq!(
        effects.0[0].0, "test",
        "BoundEffects chip name should be 'test'"
    );
    assert_eq!(
        effects.0[0].1,
        EffectNode::Do(EffectKind::DamageBoost(2.0)),
        "BoundEffects effect should be DamageBoost(2.0)"
    );
}

// -- Behavior 7 edge case: entity has no BoundEffects -- no panic --

#[test]
fn fire_reward_with_inherit_true_and_no_bound_effects_does_not_panic() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
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

    // Should not panic even though entity has no BoundEffects
    let config = default_config();
    fire(entity, &config, "", &mut world);

    let mut bolt_query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = bolt_query.iter(&world).collect();
    assert_eq!(bolts.len(), 1, "bolt should still be spawned");

    // Spawned bolt should have no BoundEffects
    let effects = world.get::<BoundEffects>(bolts[0]);
    assert!(
        effects.is_none_or(|e| e.0.is_empty()),
        "spawned bolt should have no BoundEffects when source entity has none"
    );
}
