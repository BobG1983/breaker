use super::*;

#[test]
fn fire_spawns_two_tether_bolts_with_full_physics_components() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    fire(entity, 1.5, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(
        bolts.len(),
        2,
        "fire should spawn exactly 2 tether bolts, got {}",
        bolts.len()
    );

    for bolt in &bolts {
        // Bolt marker
        assert!(
            world.get::<Bolt>(*bolt).is_some(),
            "tether bolt should have Bolt"
        );

        // ExtraBolt
        assert!(
            world.get::<ExtraBolt>(*bolt).is_some(),
            "tether bolt should have ExtraBolt"
        );

        // Position2D from owner
        let pos = world
            .get::<Position2D>(*bolt)
            .expect("tether bolt should have Position2D");
        assert_eq!(pos.0, Vec2::new(100.0, 200.0));

        // Velocity2D — magnitude at base_speed
        let vel = world
            .get::<Velocity2D>(*bolt)
            .expect("tether bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 400.0).abs() < 1.0,
            "tether bolt velocity magnitude should be base_speed (400.0), got {}",
            vel.0.length()
        );

        // Scale2D
        let scale = world
            .get::<Scale2D>(*bolt)
            .expect("tether bolt should have Scale2D");
        assert!((scale.x - 8.0).abs() < f32::EPSILON);
        assert!((scale.y - 8.0).abs() < f32::EPSILON);

        // Aabb2D
        let aabb = world
            .get::<Aabb2D>(*bolt)
            .expect("tether bolt should have Aabb2D");
        assert_eq!(aabb.center, Vec2::ZERO);
        assert_eq!(aabb.half_extents, Vec2::new(8.0, 8.0));

        // CollisionLayers
        let layers = world
            .get::<CollisionLayers>(*bolt)
            .expect("tether bolt should have CollisionLayers");
        assert_eq!(layers.membership, BOLT_LAYER);
        assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);

        // Speed components
        assert!((world.get::<BoltBaseSpeed>(*bolt).unwrap().0 - 400.0).abs() < f32::EPSILON);
        assert!((world.get::<BoltMinSpeed>(*bolt).unwrap().0 - 200.0).abs() < f32::EPSILON);
        assert!((world.get::<BoltMaxSpeed>(*bolt).unwrap().0 - 800.0).abs() < f32::EPSILON);
        assert!((world.get::<BoltRadius>(*bolt).unwrap().0 - 8.0).abs() < f32::EPSILON);

        // CleanupOnNodeExit
        assert!(world.get::<CleanupOnNodeExit>(*bolt).is_some());

        // GameDrawLayer::Bolt
        assert!(world.get::<GameDrawLayer>(*bolt).is_some());
    }
}

#[test]
fn fire_spawns_tether_bolt_marker_storing_beam_entity() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, "", &mut world);

    // Verify beam entity exists
    let mut beam_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
    let beam_count = beam_query.iter(&world).count();
    assert_eq!(beam_count, 1, "should spawn exactly 1 beam entity");

    // Both tether bolts should have TetherBoltMarker
    let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let marked_count = bolt_query.iter(&world).count();
    assert_eq!(
        marked_count, 2,
        "Both tether bolts should have TetherBoltMarker, got {marked_count}"
    );
}

#[test]
fn fire_spawns_two_bolts_with_different_velocity_directions() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, "", &mut world);

    let mut query = world.query::<(&TetherBoltMarker, &Velocity2D)>();
    let velocities: Vec<Vec2> = query.iter(&world).map(|(_, v)| v.0).collect();
    assert_eq!(velocities.len(), 2);

    for vel in &velocities {
        assert!(
            (vel.length() - 400.0).abs() < 1.0,
            "each tether bolt velocity should be ~400.0, got {}",
            vel.length()
        );
    }

    // Probabilistically different directions (each gets independent random angle)
    let dir_a = velocities[0].normalize();
    let dir_b = velocities[1].normalize();
    // With independent random angles, they should differ
    assert!(
        (dir_a - dir_b).length() > 0.001,
        "two tether bolts should have different velocity directions"
    );
}

#[test]
fn fire_does_not_spawn_distance_constraint() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, "", &mut world);

    // Gate: fire() must actually spawn tether bolts for this negative assertion to be meaningful
    let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let tether_bolt_count = bolt_query.iter(&world).count();
    assert!(
        tether_bolt_count >= 1,
        "gate: fire() must spawn tether bolts for DistanceConstraint check to be meaningful, got {tether_bolt_count}"
    );

    // No DistanceConstraint should exist — unlike ChainBolt
    let mut query = world.query::<&rantzsoft_physics2d::constraint::DistanceConstraint>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 0,
        "TetherBeam should NOT spawn DistanceConstraint, got {count}"
    );
}

#[test]
fn fire_spawns_tether_beam_component_linking_both_bolts() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

    fire(entity, 1.5, "", &mut world);

    let mut beam_query = world.query::<&TetherBeamComponent>();
    let beams: Vec<&TetherBeamComponent> = beam_query.iter(&world).collect();
    assert_eq!(beams.len(), 1, "should spawn exactly 1 TetherBeamComponent");

    let beam = beams[0];
    assert!(
        (beam.damage_mult - 1.5).abs() < f32::EPSILON,
        "damage_mult should be 1.5, got {}",
        beam.damage_mult
    );

    // Copy beam fields into owned locals so the immutable borrow on world is dropped
    let beam_bolt_a = beam.bolt_a;
    let beam_bolt_b = beam.bolt_b;
    drop(beams);

    // bolt_a and bolt_b should reference the tether bolt entities
    let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let bolt_entities: HashSet<Entity> = bolt_query.iter(&world).collect();
    assert!(
        bolt_entities.contains(&beam_bolt_a),
        "beam.bolt_a should reference a tether bolt entity"
    );
    assert!(
        bolt_entities.contains(&beam_bolt_b),
        "beam.bolt_b should reference a tether bolt entity"
    );
    assert_ne!(
        beam_bolt_a, beam_bolt_b,
        "bolt_a and bolt_b should be different entities"
    );
}

#[test]
fn fire_with_zero_damage_mult_spawns_beam() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 0.0, "", &mut world);

    let mut beam_query = world.query::<&TetherBeamComponent>();
    let beam = beam_query.iter(&world).next().expect("beam should exist");
    assert!(
        (beam.damage_mult - 0.0).abs() < f32::EPSILON,
        "damage_mult=0.0 should be stored, got {}",
        beam.damage_mult
    );
}

#[test]
fn fire_spawns_bolts_with_extra_bolt_and_cleanup_on_node_exit() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    for bolt in query.iter(&world) {
        assert!(
            world.get::<ExtraBolt>(bolt).is_some(),
            "tether bolt should have ExtraBolt"
        );
        assert!(
            world.get::<CleanupOnNodeExit>(bolt).is_some(),
            "tether bolt should have CleanupOnNodeExit"
        );
        assert!(
            world.get::<CleanupOnRunEnd>(bolt).is_none(),
            "tether bolt should NOT have CleanupOnRunEnd"
        );
    }
}

#[test]
fn fire_reads_position_from_position2d_not_transform() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::new(30.0, 40.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ))
        .id();

    fire(entity, 1.5, "", &mut world);

    let mut query = world.query::<(&TetherBoltMarker, &Position2D)>();
    for (_marker, pos) in query.iter(&world) {
        assert_eq!(
            pos.0,
            Vec2::new(30.0, 40.0),
            "tether bolt should use Position2D (30, 40), not Transform (999, 999)"
        );
    }
}

#[test]
fn fire_spawns_bolts_at_zero_when_owner_has_no_position2d() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn_empty().id();

    fire(entity, 1.5, "", &mut world);

    // Gate: fire() must actually spawn tether bolts for position check to be meaningful
    let mut count_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let tether_bolt_count = count_query.iter(&world).count();
    assert!(
        tether_bolt_count >= 2,
        "expected tether bolts to be spawned, got {tether_bolt_count}"
    );

    let mut query = world.query::<(&TetherBoltMarker, &Position2D)>();
    for (_marker, pos) in query.iter(&world) {
        assert_eq!(
            pos.0,
            Vec2::ZERO,
            "tether bolt should default to Vec2::ZERO when owner has no Position2D"
        );
    }
}

// ── reverse() — no-op ──────────────────────────────────────────

#[test]
fn reverse_does_not_despawn_tether_entities() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, "", &mut world);

    let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let bolt_count_before = bolt_query.iter(&world).count();
    let mut beam_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
    let beam_count_before = beam_query.iter(&world).count();

    reverse(entity, 1.5, "", &mut world);

    let bolt_count_after = bolt_query.iter(&world).count();
    let beam_count_after = beam_query.iter(&world).count();
    assert_eq!(
        bolt_count_before, bolt_count_after,
        "reverse should not despawn tether bolts"
    );
    assert_eq!(
        beam_count_before, beam_count_after,
        "reverse should not despawn beam"
    );
}

#[test]
fn reverse_with_no_tether_entities_does_not_panic() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Should not panic
    reverse(entity, 1.5, "", &mut world);
}

// -- Section H: EffectSourceChip attribution on fire() ───────────────────

use crate::effect::core::EffectSourceChip;

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(0.0, 0.0))).id();

    fire(entity, 2.0, "tether", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip (on beam entity)"
    );
    assert_eq!(
        results[0].0,
        Some("tether".to_string()),
        "spawned TetherBeamComponent entity should have EffectSourceChip(Some(\"tether\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_with_empty_chip_name() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(0.0, 0.0))).id();

    fire(entity, 2.0, "", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}
