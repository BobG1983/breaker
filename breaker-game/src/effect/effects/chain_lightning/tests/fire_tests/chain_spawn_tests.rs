use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use crate::{
    bolt::resources::DEFAULT_BOLT_BASE_DAMAGE,
    effect::{core::EffectSourceChip, effects::chain_lightning::tests::helpers::*},
    shared::GameRng,
    state::types::NodeState,
};

// ── Behavior 2: fire() spawns a ChainLightningChain entity with correct initial state ──

#[test]
fn fire_spawns_chain_entity_with_correct_initial_state() {
    let mut app = chain_lightning_test_app();
    app.world_mut().insert_resource(GameRng::from_seed(0));

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 60.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningChain>();
    let chains: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        chains.len(),
        1,
        "expected exactly one ChainLightningChain entity, got {}",
        chains.len()
    );

    let chain = chains[0];
    assert_eq!(
        chain.remaining_jumps, 2,
        "remaining_jumps should be 2 (arcs=3 minus 1 for initial target)"
    );

    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 1.0;
    assert!(
        (chain.damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage}, got {}",
        chain.damage
    );

    assert_eq!(
        chain.hit_set.len(),
        1,
        "hit_set should contain exactly the first target"
    );

    assert!(
        matches!(chain.state, ChainState::Idle),
        "initial state should be Idle"
    );

    assert!(
        (chain.range - 25.0).abs() < f32::EPSILON,
        "range should be 25.0, got {}",
        chain.range
    );

    assert!(
        (chain.arc_speed - 200.0).abs() < f32::EPSILON,
        "arc_speed should be 200.0, got {}",
        chain.arc_speed
    );

    assert_eq!(
        chain.source,
        Vec2::new(20.0, 0.0),
        "chain source should be the position of the first target"
    );
}

#[test]
fn fire_chain_entity_has_cleanup_on_node_exit() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningChain>>();
    let chain_entity = query
        .iter(app.world())
        .next()
        .expect("chain entity should exist");

    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(chain_entity)
            .is_some(),
        "ChainLightningChain entity should have CleanupOnExit<NodeState>"
    );
}

#[test]
fn fire_chain_entity_has_effect_source_chip_none_for_empty_chip() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ChainLightningChain>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one chain entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}

#[test]
fn fire_chain_entity_damage_includes_effective_damage_multiplier() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Position2D(Vec2::ZERO),
            crate::effect::effects::damage_boost::ActiveDamageBoosts(vec![2.0]),
        ))
        .id();

    let _cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 25.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningChain>();
    let chains: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(chains.len(), 1);

    // damage = DEFAULT_BOLT_BASE_DAMAGE * 1.0 * 2.0 = 20.0
    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 1.0 * 2.0;
    assert!(
        (chains[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected chain damage {expected_damage}, got {}",
        chains[0].damage
    );
}

// ── Behavior 9: fire() stores EffectSourceChip on chain entity ──

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 20.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "zapper", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ChainLightningChain>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one chain entity");
    assert_eq!(
        results[0].0,
        Some("zapper".to_string()),
        "spawned chain should have EffectSourceChip(Some(\"zapper\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_for_empty_chip_name() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 20.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query_filtered::<&EffectSourceChip, With<ChainLightningChain>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one chain entity");
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}
