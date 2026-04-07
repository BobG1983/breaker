use crate::effect::{EffectKind, effects::tether_beam::tests::helpers::*};

// ══════════════════════════════════════════════════════════════════
// Section G: Dispatch integration
// ══════════════════════════════════════════════════════════════════

// Behavior 30: EffectKind::fire dispatches chain=false to tether_beam::fire with chain=false
#[test]
fn dispatch_fire_chain_false_spawns_standard_tether_bolts() {
    let mut world = world_with_bolt_registry();
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
    let mut world = world_with_bolt_registry();
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
        base_damage: DEFAULT_BOLT_BASE_DAMAGE,
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
    let mut world = world_with_bolt_registry();
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
