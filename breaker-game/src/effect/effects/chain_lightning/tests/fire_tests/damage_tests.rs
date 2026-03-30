use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::helpers::*;
use crate::{bolt::BASE_BOLT_DAMAGE, cells::messages::DamageCell};

// ── Behavior 1: fire() damages the first valid target cell immediately via DamageCell ──

#[test]
fn fire_damages_first_target_immediately_via_damage_cell() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    let cell = spawn_test_cell(&mut app, 120.0, 200.0);

    // Tick to populate quadtree
    tick(&mut app);

    fire(entity, 3, 50.0, 1.5, 200.0, "", app.world_mut());

    // DamageCell should be present immediately after fire(), without needing a tick
    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "fire() should write exactly 1 DamageCell for the first target, got {}",
        written.len()
    );
    assert_eq!(
        written[0].cell, cell,
        "DamageCell should target the spawned cell"
    );

    let expected_damage = BASE_BOLT_DAMAGE * 1.5;
    assert!(
        (written[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage}, got {}",
        written[0].damage
    );
    assert_eq!(
        written[0].source_chip, None,
        "source_chip should be None for empty chip name"
    );
}

#[test]
fn fire_scales_damage_by_effective_damage_multiplier() {
    let mut app = chain_lightning_test_app();

    let entity = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(100.0, 200.0)),
            crate::effect::EffectiveDamageMultiplier(2.0),
        ))
        .id();

    let _cell = spawn_test_cell(&mut app, 120.0, 200.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.5, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(written.len(), 1, "expected 1 DamageCell");

    // damage = BASE_BOLT_DAMAGE * 1.5 * 2.0 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 1.5 * 2.0;
    assert!(
        (written[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage} (10.0 * 1.5 * 2.0), got {}",
        written[0].damage
    );
}

// ── Behavior 10b: fire() with damage_mult=0.0 ──

#[test]
fn fire_with_zero_damage_mult_sends_damage_cell_with_zero_damage() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 0.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(
        written.len(),
        1,
        "should still write DamageCell with 0 damage"
    );
    assert_eq!(written[0].cell, cell);
    assert!(
        (written[0].damage - 0.0).abs() < f32::EPSILON,
        "damage should be 0.0, got {}",
        written[0].damage
    );
}

#[test]
fn fire_with_zero_damage_mult_and_multiple_arcs_spawns_chain_with_zero_damage() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 30.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 0.0, 200.0, "", app.world_mut());

    let mut query = app.world_mut().query::<&ChainLightningChain>();
    let chains: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(chains.len(), 1, "should spawn chain entity with damage=0.0");
    assert!(
        (chains[0].damage - 0.0).abs() < f32::EPSILON,
        "chain damage should be 0.0"
    );
}

// ── Behavior 21: DamageCell from fire() includes source_chip ──

#[test]
fn fire_damage_cell_includes_source_chip() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.5, 200.0, "zapper", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(written.len(), 1);

    let expected_damage = BASE_BOLT_DAMAGE * 1.5;
    assert!(
        (written[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage}"
    );
    assert_eq!(
        written[0].source_chip,
        Some("zapper".to_string()),
        "DamageCell should include source_chip"
    );
}
