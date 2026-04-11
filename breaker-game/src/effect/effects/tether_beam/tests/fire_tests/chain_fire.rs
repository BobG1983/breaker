use crate::effect::{
    core::EffectSourceChip,
    effects::{damage_boost::ActiveDamageBoosts, tether_beam::tests::helpers::*},
};

// ══════════════════════════════════════════════════════════════════
// Section B: Standard mode fire (chain: false) — new assertions
// ══════════════════════════════════════════════════════════════════

// Behavior 4: fire() with chain=false does not insert TetherChainActive resource
#[test]
fn fire_chain_false_does_not_insert_tether_chain_active_resource() {
    let mut world = world_with_bolt_registry();
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
    let mut world = world_with_bolt_registry();
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

// Behavior 6: fire() with chain=true and 3 existing bolts creates 2 chain beams
#[test]
fn fire_chain_true_with_three_bolts_creates_two_chain_beams() {
    let mut world = World::new();
    world.spawn(Bolt);
    world.spawn(Bolt);
    world.spawn(Bolt);
    let entity = world.spawn(ActiveDamageBoosts(vec![1.5])).id();

    fire(entity, 2.0, true, "arcwelder", &mut world);

    let mut chain_query =
        world.query_filtered::<Entity, (With<TetherBeamComponent>, With<TetherChainBeam>)>();
    let chain_beam_count = chain_query.iter(&world).count();
    assert_eq!(
        chain_beam_count, 2,
        "fire with chain=true and 3 bolts should create 2 chain beams, got {chain_beam_count}"
    );

    // Verify beam properties
    let mut beam_query = world.query::<(
        &TetherBeamComponent,
        &EffectSourceChip,
        &CleanupOnExit<NodeState>,
    )>();
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

// Behavior 6 edge case: entity without ActiveDamageBoosts uses default 1.0
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
            "chain beam EDM should default to 1.0 when entity has no ActiveDamageBoosts, got {}",
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
    let entity = world.spawn(ActiveDamageBoosts(vec![2.0])).id();

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
        base_damage: DEFAULT_BOLT_BASE_DAMAGE,
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
    let entity = world.spawn(ActiveDamageBoosts(vec![2.0])).id();

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
