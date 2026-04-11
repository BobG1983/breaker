use crate::effect::effects::tether_beam::tests::helpers::*;

// ══════════════════════════════════════════════════════════════════
// Section D: Standard mode reverse (chain: false) — existing behavior preserved
// ══════════════════════════════════════════════════════════════════

// Behavior 17: reverse() with chain=false is still a no-op
#[test]
fn reverse_chain_false_is_still_a_noop() {
    let mut world = world_with_bolt_registry();
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
        base_damage: DEFAULT_BOLT_BASE_DAMAGE,
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
        base_damage: DEFAULT_BOLT_BASE_DAMAGE,
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
        base_damage: DEFAULT_BOLT_BASE_DAMAGE,
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
            base_damage: DEFAULT_BOLT_BASE_DAMAGE,
        })
        .id();
    // 2 chain beams (with TetherChainBeam marker)
    world.spawn((
        TetherBeamComponent {
            bolt_a: Entity::PLACEHOLDER,
            bolt_b: Entity::PLACEHOLDER,
            damage_mult: 1.0,
            effective_damage_multiplier: 1.0,
            base_damage: DEFAULT_BOLT_BASE_DAMAGE,
        },
        TetherChainBeam,
    ));
    world.spawn((
        TetherBeamComponent {
            bolt_a: Entity::PLACEHOLDER,
            bolt_b: Entity::PLACEHOLDER,
            damage_mult: 1.0,
            effective_damage_multiplier: 1.0,
            base_damage: DEFAULT_BOLT_BASE_DAMAGE,
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
