use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;

use super::helpers::*;
use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    shared::{GameRng, GameState, PlayingState},
};

// ── Behavior 11: reverse() is a no-op ──

#[test]
fn reverse_is_noop() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

    reverse(entity, &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "entity should still exist after no-op reverse"
    );
}

#[test]
fn reverse_on_empty_entity_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    reverse(entity, &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "empty entity should still exist after no-op reverse"
    );
}

// ── Behavior 12: process_chain_lightning sends DamageCell for each target ──

#[test]
fn process_chain_lightning_sends_damage_for_each_target_and_despawns() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = app.world_mut().spawn_empty().id();
    let cell_b = app.world_mut().spawn_empty().id();

    let request = app
        .world_mut()
        .spawn(ChainLightningRequest {
            targets: vec![(cell_a, 15.0), (cell_b, 15.0)],
            source: Vec2::new(0.0, 0.0),
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages, got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(damaged_cells.contains(&cell_a), "cell_a should be damaged");
    assert!(damaged_cells.contains(&cell_b), "cell_b should be damaged");

    for msg in &collector.0 {
        assert!(
            (msg.damage - 15.0).abs() < f32::EPSILON,
            "expected damage 15.0, got {}",
            msg.damage
        );
        assert!(msg.source_chip.is_none(), "source_chip should be None");
    }

    // Request entity should be despawned
    assert!(
        app.world().get_entity(request).is_err(),
        "ChainLightningRequest entity should be despawned after processing"
    );
}

// ── Behavior 13: process with empty targets — despawns without damage ──

#[test]
fn process_chain_lightning_handles_empty_targets_despawns_without_damage() {
    let mut app = chain_lightning_damage_test_app();

    let request = app
        .world_mut()
        .spawn(ChainLightningRequest {
            targets: vec![],
            source: Vec2::ZERO,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "empty targets should produce zero DamageCell messages"
    );

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned even with empty targets"
    );
}

#[test]
fn process_chain_lightning_multiple_empty_requests_all_despawned() {
    let mut app = chain_lightning_damage_test_app();

    let req1 = app
        .world_mut()
        .spawn(ChainLightningRequest {
            targets: vec![],
            source: Vec2::ZERO,
        })
        .id();

    let req2 = app
        .world_mut()
        .spawn(ChainLightningRequest {
            targets: vec![],
            source: Vec2::ZERO,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no damage from empty-target requests"
    );

    assert!(
        app.world().get_entity(req1).is_err(),
        "first empty request should be despawned"
    );
    assert!(
        app.world().get_entity(req2).is_err(),
        "second empty request should be despawned"
    );
}

// ── Behavior 14: Multiple requests processed independently ──

#[test]
fn multiple_chain_lightning_requests_processed_independently() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = app.world_mut().spawn_empty().id();
    let cell_b = app.world_mut().spawn_empty().id();

    let req1 = app
        .world_mut()
        .spawn(ChainLightningRequest {
            targets: vec![(cell_a, 10.0)],
            source: Vec2::ZERO,
        })
        .id();

    let req2 = app
        .world_mut()
        .spawn(ChainLightningRequest {
            targets: vec![(cell_b, 20.0)],
            source: Vec2::ZERO,
        })
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages total, got {}",
        collector.0.len()
    );

    let damage_a: Vec<f32> = collector
        .0
        .iter()
        .filter(|m| m.cell == cell_a)
        .map(|m| m.damage)
        .collect();
    assert_eq!(damage_a.len(), 1);
    assert!(
        (damage_a[0] - 10.0).abs() < f32::EPSILON,
        "cell_a should receive damage 10.0"
    );

    let damage_b: Vec<f32> = collector
        .0
        .iter()
        .filter(|m| m.cell == cell_b)
        .map(|m| m.damage)
        .collect();
    assert_eq!(damage_b.len(), 1);
    assert!(
        (damage_b[0] - 20.0).abs() < f32::EPSILON,
        "cell_b should receive damage 20.0"
    );

    assert!(
        app.world().get_entity(req1).is_err(),
        "first request should be despawned"
    );
    assert!(
        app.world().get_entity(req2).is_err(),
        "second request should be despawned"
    );
}

#[test]
fn both_requests_targeting_same_cell_produce_separate_damage_messages() {
    let mut app = chain_lightning_damage_test_app();

    let cell = app.world_mut().spawn_empty().id();

    app.world_mut().spawn(ChainLightningRequest {
        targets: vec![(cell, 10.0)],
        source: Vec2::ZERO,
    });

    app.world_mut().spawn(ChainLightningRequest {
        targets: vec![(cell, 20.0)],
        source: Vec2::ZERO,
    });

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "both requests target same cell — should produce 2 separate DamageCell messages"
    );

    let mut damages: Vec<f32> = collector.0.iter().map(|m| m.damage).collect();
    damages.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        (damages[0] - 10.0).abs() < f32::EPSILON,
        "expected damage 10.0"
    );
    assert!(
        (damages[1] - 20.0).abs() < f32::EPSILON,
        "expected damage 20.0"
    );
}

// ── Damage scaling: end-to-end chain lightning damage includes EDM ──

#[test]
fn chain_lightning_end_to_end_damage_includes_effective_damage_multiplier() {
    // End-to-end test: fire() pre-computes damage with EDM, process sends DamageCell
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.insert_resource(GameRng::from_seed(42));
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, process_chain_lightning);
    app.add_systems(Update, collect_damage_cells.after(process_chain_lightning));

    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::effect::EffectiveDamageMultiplier(2.0),
        ))
        .id();

    let _cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // Tick to populate quadtree
    tick(&mut app);

    // fire() should read EDM and pre-compute scaled damage
    fire(entity, 1, 100.0, 1.5, app.world_mut());

    // Tick again to run process_chain_lightning
    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell message, got {}",
        collector.0.len()
    );

    // damage = BASE_BOLT_DAMAGE * damage_mult * EDM = 10.0 * 1.5 * 2.0 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 1.5 * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "end-to-end damage should be {} (10.0 * 1.5 * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
    assert!(
        collector.0[0].source_chip.is_none(),
        "source_chip should be None"
    );
}

// ── Behavior 15: register() wires the process system ──

#[test]
fn register_wires_process_chain_lightning_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(RantzPhysics2dPlugin);
    app.init_state::<GameState>();
    app.add_sub_state::<PlayingState>();
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, collect_damage_cells);

    register(&mut app);

    // Transition to PlayingState::Active so the run_if guard passes
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Spawn a request — if register() wires the system, it should be processed
    let request = app
        .world_mut()
        .spawn(ChainLightningRequest {
            targets: vec![],
            source: Vec2::ZERO,
        })
        .id();

    tick(&mut app);

    // The request should be despawned after processing
    assert!(
        app.world().get_entity(request).is_err(),
        "register() should wire process_chain_lightning — request should be despawned after tick"
    );
}
