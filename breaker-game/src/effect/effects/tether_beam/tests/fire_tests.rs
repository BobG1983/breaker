use super::*;

#[test]
fn fire_spawns_two_tether_bolts_with_full_physics_components() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    fire(entity, 1.5, false, "", &mut world);

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

    fire(entity, 1.5, false, "", &mut world);

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

    fire(entity, 1.5, false, "", &mut world);

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

    fire(entity, 1.5, false, "", &mut world);

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

    fire(entity, 1.5, false, "", &mut world);

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

    fire(entity, 0.0, false, "", &mut world);

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

    fire(entity, 1.5, false, "", &mut world);

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

    fire(entity, 1.5, false, "", &mut world);

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

    fire(entity, 1.5, false, "", &mut world);

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

// ── reverse() with chain=false — no-op ──────────────────────────────────────────

#[test]
fn reverse_does_not_despawn_tether_entities() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, false, "", &mut world);

    let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let bolt_count_before = bolt_query.iter(&world).count();
    let mut beam_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
    let beam_count_before = beam_query.iter(&world).count();

    reverse(entity, 1.5, false, "", &mut world);

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
    reverse(entity, 1.5, false, "", &mut world);
}

// -- Section H: EffectSourceChip attribution on fire() ───────────────────

use crate::effect::core::EffectSourceChip;

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(0.0, 0.0))).id();

    fire(entity, 2.0, false, "tether", &mut world);

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

    fire(entity, 2.0, false, "", &mut world);

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

// ══════════════════════════════════════════════════════════════════
// Section B: Standard mode fire (chain: false) — new assertions
// ══════════════════════════════════════════════════════════════════

// Behavior 4: fire() with chain=false does not insert TetherChainActive resource
#[test]
fn fire_chain_false_does_not_insert_tether_chain_active_resource() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, false, "", &mut world);

    assert!(
        !world.contains_resource::<TetherChainActive>(),
        "fire with chain=false should NOT insert TetherChainActive resource"
    );
}

// Behavior 5: fire() with chain=false does not spawn TetherChainBeam entities
#[test]
fn fire_chain_false_does_not_spawn_tether_chain_beam_entities() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();
    // Pre-spawn 3 bolt entities
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);

    fire(entity, 1.5, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<TetherChainBeam>>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 0,
        "fire with chain=false should NOT spawn TetherChainBeam entities, got {count}"
    );
}

// ══════════════════════════════════════════════════════════════════
// Section C: Chain mode fire (chain: true) — core behavior
// ══════════════════════════════════════════════════════════════════

use crate::effect::EffectiveDamageMultiplier;

// Behavior 6: fire() with chain=true and 3 existing bolts creates 2 chain beams
#[test]
fn fire_chain_true_with_three_bolts_creates_two_chain_beams() {
    let mut world = World::new();
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn(EffectiveDamageMultiplier(1.5)).id();

    fire(entity, 2.0, true, "arcwelder", &mut world);

    let mut chain_query =
        world.query_filtered::<Entity, (With<TetherBeamComponent>, With<TetherChainBeam>)>();
    let chain_beam_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_beam_count, 2,
        "fire with chain=true and 3 bolts should create 2 chain beams, got {chain_beam_count}"
    );

    // Verify beam properties
    let mut beam_query =
        world.query::<(&TetherBeamComponent, &EffectSourceChip, &CleanupOnNodeExit)>();
    for (beam, esc, _cleanup) in beam_query.iter(&world) {
        assert!(
            (beam.damage_mult - 2.0).abs() < f32::EPSILON,
            "chain beam damage_mult should be 2.0, got {}",
            beam.damage_mult
        );
        assert!(
            (beam.effective_damage_multiplier - 1.5).abs() < f32::EPSILON,
            "chain beam EDM should be 1.5, got {}",
            beam.effective_damage_multiplier
        );
        assert_eq!(
            esc.0,
            Some("arcwelder".to_string()),
            "chain beam should have EffectSourceChip(Some(\"arcwelder\"))"
        );
    }
}

// Behavior 6 edge case: entity without EffectiveDamageMultiplier uses default 1.0
#[test]
fn fire_chain_true_without_edm_uses_default_one() {
    let mut world = World::new();
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn_empty().id();

    fire(entity, 2.0, true, "", &mut world);

    let mut beam_query = world.query::<&TetherBeamComponent>();
    for beam in beam_query.iter(&world) {
        assert!(
            (beam.effective_damage_multiplier - 1.0).abs() < f32::EPSILON,
            "chain beam EDM should default to 1.0 when entity has no EffectiveDamageMultiplier, got {}",
            beam.effective_damage_multiplier
        );
    }
}

// Behavior 7: fire() with chain=true and 4 existing bolts creates 3 chain beams
#[test]
fn fire_chain_true_with_four_bolts_creates_three_chain_beams() {
    let mut world = World::new();
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn_empty().id();

    fire(entity, 1.0, true, "", &mut world);

    let mut chain_query =
        world.query_filtered::<Entity, (With<TetherBeamComponent>, With<TetherChainBeam>)>();
    let chain_beam_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_beam_count, 3,
        "fire with chain=true and 4 bolts should create 3 chain beams, got {chain_beam_count}"
    );
}

// Behavior 8: fire() with chain=true and 2 existing bolts creates 1 chain beam
#[test]
fn fire_chain_true_with_two_bolts_creates_one_chain_beam() {
    let mut world = World::new();
    let bolt_a = world.spawn(Bolt).id();
    let bolt_b = world.spawn(Bolt).id();
    let entity = world.spawn_empty().id();

    fire(entity, 1.0, true, "", &mut world);

    let mut chain_query =
        world.query_filtered::<Entity, (With<TetherBeamComponent>, With<TetherChainBeam>)>();
    let chain_beam_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_beam_count, 1,
        "fire with chain=true and 2 bolts should create 1 chain beam, got {chain_beam_count}"
    );

    // Verify beam references the two bolt entities
    let mut beam_query = world.query::<&TetherBeamComponent>();
    let beam = beam_query.iter(&world).next().expect("should have 1 beam");
    let beam_bolts: HashSet<Entity> = [beam.bolt_a, beam.bolt_b].into();
    let expected_bolts: HashSet<Entity> = [bolt_a, bolt_b].into();
    assert_eq!(
        beam_bolts, expected_bolts,
        "chain beam should connect the two existing bolt entities"
    );
}

// Behavior 9: fire() with chain=true and 1 existing bolt creates 0 chain beams
#[test]
fn fire_chain_true_with_one_bolt_creates_zero_chain_beams() {
    let mut world = World::new();
    world.spawn(Bolt);
    let entity = world.spawn_empty().id();

    fire(entity, 1.0, true, "", &mut world);

    let mut chain_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
    let beam_count = chain_query.iter(&world).count();
    assert_eq!(
        beam_count, 0,
        "fire with chain=true and 1 bolt should create 0 beams, got {beam_count}"
    );

    // TetherChainActive resource should still be inserted with last_bolt_count == 1
    let chain_active = world
        .get_resource::<TetherChainActive>()
        .expect("TetherChainActive should be inserted even with 1 bolt");
    assert_eq!(
        chain_active.last_bolt_count, 1,
        "last_bolt_count should be 1, got {}",
        chain_active.last_bolt_count
    );
}

// Behavior 10: fire() with chain=true and 0 existing bolts creates 0 chain beams
#[test]
fn fire_chain_true_with_zero_bolts_creates_zero_chain_beams() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, 1.0, true, "", &mut world);

    let mut chain_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
    let beam_count = chain_query.iter(&world).count();
    assert_eq!(
        beam_count, 0,
        "fire with chain=true and 0 bolts should create 0 beams, got {beam_count}"
    );

    let chain_active = world
        .get_resource::<TetherChainActive>()
        .expect("TetherChainActive should be inserted even with 0 bolts");
    assert_eq!(
        chain_active.last_bolt_count, 0,
        "last_bolt_count should be 0, got {}",
        chain_active.last_bolt_count
    );
}

// Behavior 11: fire() with chain=true does NOT spawn new bolts
#[test]
fn fire_chain_true_does_not_spawn_new_bolts() {
    let mut world = World::new();
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn_empty().id();

    fire(entity, 1.0, true, "", &mut world);

    let mut bolt_query = world.query_filtered::<Entity, With<Bolt>>();
    let bolt_count = bolt_query.iter(&world).count();
    assert_eq!(
        bolt_count, 2,
        "fire with chain=true should not spawn new bolts, expected 2, got {bolt_count}"
    );

    let mut marker_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let marker_count = marker_query.iter(&world).count();
    assert_eq!(
        marker_count, 0,
        "fire with chain=true should not use TetherBoltMarker, got {marker_count}"
    );
}

// Behavior 12: fire() with chain=true inserts TetherChainActive resource
#[test]
fn fire_chain_true_inserts_tether_chain_active_resource() {
    let mut world = World::new();
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn(EffectiveDamageMultiplier(2.0)).id();

    fire(entity, 1.5, true, "arcwelder", &mut world);

    let chain_active = world
        .get_resource::<TetherChainActive>()
        .expect("TetherChainActive should be inserted by fire with chain=true");
    assert!(
        (chain_active.damage_mult - 1.5).abs() < f32::EPSILON,
        "damage_mult should be 1.5, got {}",
        chain_active.damage_mult
    );
    assert!(
        (chain_active.effective_damage_multiplier - 2.0).abs() < f32::EPSILON,
        "effective_damage_multiplier should be 2.0, got {}",
        chain_active.effective_damage_multiplier
    );
    assert_eq!(
        chain_active.source_chip,
        Some("arcwelder".to_string()),
        "source_chip should be Some(\"arcwelder\")"
    );
    assert_eq!(
        chain_active.last_bolt_count, 3,
        "last_bolt_count should be 3, got {}",
        chain_active.last_bolt_count
    );
}

// Behavior 13: fire() with chain=true and empty source_chip stores None
#[test]
fn fire_chain_true_with_empty_source_chip_stores_none() {
    let mut world = World::new();
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn_empty().id();

    fire(entity, 1.0, true, "", &mut world);

    let chain_active = world
        .get_resource::<TetherChainActive>()
        .expect("TetherChainActive should be inserted");
    assert_eq!(
        chain_active.source_chip, None,
        "empty source_chip should produce None, got {:?}",
        chain_active.source_chip
    );
}

// Behavior 14: fire() with chain=true despawns existing TetherChainBeam entities before creating new ones
#[test]
fn fire_chain_true_despawns_existing_chain_beams_before_creating_new() {
    let mut world = World::new();
    // Pre-existing chain beams (from a hypothetical previous fire)
    world.spawn(TetherChainBeam);
    world.spawn(TetherChainBeam);
    // 3 existing bolts
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn_empty().id();

    fire(entity, 1.0, true, "", &mut world);

    let mut chain_query = world.query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_beam_count, 2,
        "should have exactly 2 chain beams (old 2 removed, 2 new for 3 bolts), got {chain_beam_count}"
    );
}

// Behavior 15: fire() with chain=true connects bolts in Entity sort order
#[test]
fn fire_chain_true_connects_bolts_in_entity_sort_order() {
    let mut world = World::new();
    let bolt_a = world.spawn(Bolt).id();
    let bolt_b = world.spawn(Bolt).id();
    let bolt_c = world.spawn(Bolt).id();
    let entity = world.spawn_empty().id();

    // Verify entity index ordering (lower index = earlier in sort)
    assert!(
        bolt_a.index() < bolt_b.index(),
        "bolt_a should sort before bolt_b by index"
    );
    assert!(
        bolt_b.index() < bolt_c.index(),
        "bolt_b should sort before bolt_c by index"
    );

    fire(entity, 1.0, true, "", &mut world);

    let mut beam_query = world.query::<&TetherBeamComponent>();
    let mut beam_pairs: Vec<(Entity, Entity)> = beam_query
        .iter(&world)
        .map(|b| {
            if b.bolt_a.index() < b.bolt_b.index() {
                (b.bolt_a, b.bolt_b)
            } else {
                (b.bolt_b, b.bolt_a)
            }
        })
        .collect();
    beam_pairs.sort_by_key(|(a, _)| a.index());

    assert_eq!(
        beam_pairs,
        vec![(bolt_a, bolt_b), (bolt_b, bolt_c)],
        "chain beams should connect consecutive bolt pairs in Entity sort order"
    );
}

// Behavior 16: fire() with chain=true overwrites existing TetherChainActive resource on re-fire
#[test]
fn fire_chain_true_overwrites_existing_tether_chain_active() {
    let mut world = World::new();
    // Insert a pre-existing TetherChainActive resource
    world.insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 3,
    });
    // Pre-existing chain beams
    world.spawn(TetherChainBeam);
    world.spawn(TetherChainBeam);
    // 3 existing bolts
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn(EffectiveDamageMultiplier(2.0)).id();

    fire(entity, 2.5, true, "arcwelder", &mut world);

    let chain_active = world
        .get_resource::<TetherChainActive>()
        .expect("TetherChainActive should exist after re-fire");
    assert!(
        (chain_active.damage_mult - 2.5).abs() < f32::EPSILON,
        "damage_mult should be overwritten to 2.5, got {}",
        chain_active.damage_mult
    );
    assert!(
        (chain_active.effective_damage_multiplier - 2.0).abs() < f32::EPSILON,
        "EDM should be overwritten to 2.0, got {}",
        chain_active.effective_damage_multiplier
    );
    assert_eq!(
        chain_active.source_chip,
        Some("arcwelder".to_string()),
        "source_chip should be overwritten to Some(\"arcwelder\")"
    );
    assert_eq!(
        chain_active.last_bolt_count, 3,
        "last_bolt_count should be 3, got {}",
        chain_active.last_bolt_count
    );

    // Old chain beams should be replaced by 2 new ones
    let mut chain_query = world.query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_beam_count, 2,
        "should have 2 new chain beams after re-fire, got {chain_beam_count}"
    );
}

// ══════════════════════════════════════════════════════════════════
// Section D: Standard mode reverse (chain: false) — existing behavior preserved
// ══════════════════════════════════════════════════════════════════

// Behavior 17: reverse() with chain=false is still a no-op
#[test]
fn reverse_chain_false_is_still_a_noop() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1.5, false, "", &mut world);

    let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let bolt_count_before = bolt_query.iter(&world).count();
    let mut beam_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
    let beam_count_before = beam_query.iter(&world).count();

    reverse(entity, 1.5, false, "", &mut world);

    let bolt_count_after = bolt_query.iter(&world).count();
    let beam_count_after = beam_query.iter(&world).count();
    assert_eq!(
        bolt_count_before, bolt_count_after,
        "reverse(chain=false) should not despawn bolts"
    );
    assert_eq!(
        beam_count_before, beam_count_after,
        "reverse(chain=false) should not despawn beam"
    );
    assert!(
        !world.contains_resource::<TetherChainActive>(),
        "reverse(chain=false) should not leave a TetherChainActive resource"
    );
}

// ══════════════════════════════════════════════════════════════════
// Section E: Chain mode reverse (chain: true)
// ══════════════════════════════════════════════════════════════════

// Behavior 18: reverse() with chain=true removes TetherChainActive resource
#[test]
fn reverse_chain_true_removes_tether_chain_active_resource() {
    let mut world = World::new();
    world.insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 2,
    });
    let entity = world.spawn_empty().id();

    reverse(entity, 1.0, true, "", &mut world);

    assert!(
        !world.contains_resource::<TetherChainActive>(),
        "reverse with chain=true should remove TetherChainActive resource"
    );
}

// Behavior 19: reverse() with chain=true despawns all TetherChainBeam entities
#[test]
fn reverse_chain_true_despawns_all_chain_beam_entities() {
    let mut world = World::new();
    world.insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 3,
    });
    // Spawn 3 chain beam entities
    world.spawn(TetherChainBeam);
    world.spawn(TetherChainBeam);
    world.spawn(TetherChainBeam);
    // Bolt entities should survive
    let bolt_a = world.spawn(Bolt).id();
    let bolt_b = world.spawn(Bolt).id();
    let entity = world.spawn_empty().id();

    reverse(entity, 1.0, true, "", &mut world);

    let mut chain_query = world.query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_count, 0,
        "reverse with chain=true should despawn all TetherChainBeam entities, got {chain_count}"
    );

    // Bolt entities should still exist
    assert!(
        world.get_entity(bolt_a).is_ok(),
        "bolt entities should survive chain reverse"
    );
    assert!(
        world.get_entity(bolt_b).is_ok(),
        "bolt entities should survive chain reverse"
    );
}

// Behavior 20: reverse() with chain=true when no TetherChainActive exists does not panic
#[test]
fn reverse_chain_true_without_tether_chain_active_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Should not panic
    reverse(entity, 1.0, true, "", &mut world);
}

// Behavior 21: reverse() with chain=true does not despawn standard tether beams
#[test]
fn reverse_chain_true_does_not_despawn_standard_tether_beams() {
    let mut world = World::new();
    world.insert_resource(TetherChainActive {
        damage_mult: 1.0,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 2,
    });
    // 1 standard beam (no TetherChainBeam marker)
    let standard_beam = world
        .spawn(TetherBeamComponent {
            bolt_a: Entity::PLACEHOLDER,
            bolt_b: Entity::PLACEHOLDER,
            damage_mult: 1.0,
            effective_damage_multiplier: 1.0,
        })
        .id();
    // 2 chain beams (with TetherChainBeam marker)
    world.spawn((
        TetherBeamComponent {
            bolt_a: Entity::PLACEHOLDER,
            bolt_b: Entity::PLACEHOLDER,
            damage_mult: 1.0,
            effective_damage_multiplier: 1.0,
        },
        TetherChainBeam,
    ));
    world.spawn((
        TetherBeamComponent {
            bolt_a: Entity::PLACEHOLDER,
            bolt_b: Entity::PLACEHOLDER,
            damage_mult: 1.0,
            effective_damage_multiplier: 1.0,
        },
        TetherChainBeam,
    ));
    let entity = world.spawn_empty().id();

    reverse(entity, 1.0, true, "", &mut world);

    // Standard beam should still exist
    assert!(
        world.get_entity(standard_beam).is_ok(),
        "standard TetherBeamComponent (without TetherChainBeam) should survive chain reverse"
    );
    assert!(
        world.get::<TetherBeamComponent>(standard_beam).is_some(),
        "standard beam should retain TetherBeamComponent"
    );

    // Chain beams should be gone
    let mut chain_query = world.query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_count, 0,
        "chain beams should be despawned, got {chain_count}"
    );
}

// ══════════════════════════════════════════════════════════════════
// Section G: Dispatch integration
// ══════════════════════════════════════════════════════════════════

use crate::effect::EffectKind;

// Behavior 30: EffectKind::fire dispatches chain=false to tether_beam::fire with chain=false
#[test]
fn dispatch_fire_chain_false_spawns_standard_tether_bolts() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    let effect = EffectKind::TetherBeam {
        damage_mult: 1.5,
        chain: false,
    };
    effect.fire(entity, "", &mut world);

    let mut marker_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let tether_bolt_count = marker_query.iter(&world).count();
    assert_eq!(
        tether_bolt_count, 2,
        "EffectKind::fire with chain=false should spawn 2 TetherBoltMarker entities, got {tether_bolt_count}"
    );
}

// Behavior 31: EffectKind::fire dispatches chain=true to tether_beam::fire with chain=true
#[test]
fn dispatch_fire_chain_true_spawns_chain_beams() {
    let mut world = world_with_bolt_config();
    // 3 existing bolts
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    let effect = EffectKind::TetherBeam {
        damage_mult: 1.5,
        chain: true,
    };
    effect.fire(entity, "", &mut world);

    let mut marker_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let tether_bolt_count = marker_query.iter(&world).count();
    assert_eq!(
        tether_bolt_count, 0,
        "EffectKind::fire with chain=true should NOT spawn TetherBoltMarker entities, got {tether_bolt_count}"
    );

    let mut chain_query = world.query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_beam_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_beam_count, 2,
        "EffectKind::fire with chain=true and 3 bolts should spawn 2 TetherChainBeam entities, got {chain_beam_count}"
    );

    assert!(
        world.contains_resource::<TetherChainActive>(),
        "EffectKind::fire with chain=true should insert TetherChainActive resource"
    );
}

// Behavior 32: EffectKind::reverse dispatches chain=true to tether_beam::reverse with chain=true
#[test]
fn dispatch_reverse_chain_true_removes_chain_state() {
    let mut world = World::new();
    world.insert_resource(TetherChainActive {
        damage_mult: 1.5,
        effective_damage_multiplier: 1.0,
        source_chip: None,
        last_bolt_count: 2,
    });
    world.spawn(TetherChainBeam);
    world.spawn(TetherChainBeam);
    let entity = world.spawn_empty().id();

    let effect = EffectKind::TetherBeam {
        damage_mult: 1.5,
        chain: true,
    };
    effect.reverse(entity, "", &mut world);

    assert!(
        !world.contains_resource::<TetherChainActive>(),
        "EffectKind::reverse with chain=true should remove TetherChainActive resource"
    );

    let mut chain_query = world.query_filtered::<Entity, With<TetherChainBeam>>();
    let chain_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_count, 0,
        "EffectKind::reverse with chain=true should despawn TetherChainBeam entities, got {chain_count}"
    );
}

// Behavior 33: EffectKind::reverse dispatches chain=false — still a no-op
#[test]
fn dispatch_reverse_chain_false_is_noop() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Fire standard mode first
    fire(entity, 1.5, false, "", &mut world);

    let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
    let bolt_count_before = bolt_query.iter(&world).count();
    let mut beam_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
    let beam_count_before = beam_query.iter(&world).count();

    let effect = EffectKind::TetherBeam {
        damage_mult: 1.5,
        chain: false,
    };
    effect.reverse(entity, "", &mut world);

    let bolt_count_after = bolt_query.iter(&world).count();
    let beam_count_after = beam_query.iter(&world).count();
    assert_eq!(
        bolt_count_before, bolt_count_after,
        "EffectKind::reverse with chain=false should not despawn tether bolts"
    );
    assert_eq!(
        beam_count_before, beam_count_after,
        "EffectKind::reverse with chain=false should not despawn beam"
    );
}
