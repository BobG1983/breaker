use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::*;
use crate::{bolt::BASE_BOLT_DAMAGE, effect::effects::shockwave::system::*};

// =========================================================================
// Part A: Observer spawns entity
// =========================================================================

/// Behavior 1: Observer spawns `ShockwaveRadius` entity at bolt position
/// with correct components. Max = `base_range` + (stacks-1) * `range_per_level`.
#[test]
fn observer_spawns_shockwave_entity_at_bolt_position() {
    use crate::effect::typed_events::ShockwaveFired;

    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 50.0, 100.0);

    // base_range=64, range_per_level=32, stacks=2 -> max = 64 + (2-1)*32 = 96
    app.world_mut().commands().trigger(ShockwaveFired {
        base_range: 64.0,
        range_per_level: 32.0,
        stacks: 2,
        speed: 400.0,
        targets: vec![crate::effect::definition::EffectTarget::Entity(bolt)],
        source_chip: None,
    });
    app.world_mut().flush();
    tick(&mut app);

    // A ShockwaveRadius entity should exist
    assert_eq!(
        shockwave_entity_count(&mut app),
        1,
        "observer should spawn exactly one ShockwaveRadius entity"
    );

    let sw_entity = get_shockwave_entity(&mut app);
    let world = app.world();

    // Position2D should match the bolt's position
    let pos = world
        .get::<Position2D>(sw_entity)
        .expect("shockwave entity should have Position2D");
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "shockwave x should be 50.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 100.0).abs() < f32::EPSILON,
        "shockwave y should be 100.0, got {}",
        pos.0.y
    );

    // ShockwaveRadius should start at 0.0 with max = 96.0
    let radius = world
        .get::<ShockwaveRadius>(sw_entity)
        .expect("shockwave entity should have ShockwaveRadius");
    assert!(
        radius.current.abs() < f32::EPSILON,
        "initial radius.current should be 0.0, got {}",
        radius.current
    );
    assert!(
        (radius.max - 96.0).abs() < f32::EPSILON,
        "radius.max should be 96.0, got {}",
        radius.max
    );

    // ShockwaveSpeed
    let speed = world
        .get::<ShockwaveSpeed>(sw_entity)
        .expect("shockwave entity should have ShockwaveSpeed");
    assert!(
        (speed.0 - 400.0).abs() < f32::EPSILON,
        "ShockwaveSpeed should be 400.0, got {}",
        speed.0
    );

    // ShockwaveDamage with damage = BASE_BOLT_DAMAGE * 1.0 = 10.0
    let dmg = world
        .get::<ShockwaveDamage>(sw_entity)
        .expect("shockwave entity should have ShockwaveDamage");
    assert!(
        (dmg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
        "damage should be {} (no boost), got {}",
        BASE_BOLT_DAMAGE,
        dmg.damage
    );
    // source_bolt field was commented out from ShockwaveDamage

    assert_standard_shockwave_components(app.world(), sw_entity);

    // NO DamageCell messages should have been written by the observer
    let captured = app.world().resource::<CapturedDamage>();
    assert_eq!(
        captured.0.len(),
        0,
        "observer should NOT write DamageCell — collision system handles that; got {}",
        captured.0.len()
    );
}

/// Behavior 2: Without `DamageBoost`, damage = `BASE_BOLT_DAMAGE` * 1.0.
#[test]
fn observer_calculates_damage_without_boost() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0);

    trigger_shockwave(&mut app, bolt, 96.0, 400.0);

    let sw_entity = get_shockwave_entity(&mut app);
    let dmg = app
        .world()
        .get::<ShockwaveDamage>(sw_entity)
        .expect("shockwave entity should have ShockwaveDamage");
    assert!(
        (dmg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
        "without DamageBoost, damage should be {} (BASE_BOLT_DAMAGE * 1.0), got {}",
        BASE_BOLT_DAMAGE,
        dmg.damage
    );
}

/// Behavior 2 variant: With `DamageBoost(0.5)`, damage = 10.0 * 1.5 = 15.0.
#[test]
fn observer_calculates_damage_with_boost() {
    let mut app = test_app();
    let bolt = spawn_bolt_with_damage_boost(&mut app, 0.0, 0.0, 0.5);

    trigger_shockwave(&mut app, bolt, 96.0, 400.0);

    let sw_entity = get_shockwave_entity(&mut app);
    let dmg = app
        .world()
        .get::<ShockwaveDamage>(sw_entity)
        .expect("shockwave entity should have ShockwaveDamage");
    // damage = BASE_BOLT_DAMAGE * (1.0 + 0.5) = 10.0 * 1.5 = 15.0
    assert!(
        (dmg.damage - 15.0).abs() < f32::EPSILON,
        "with DamageBoost(0.5), damage should be 15.0 (10.0 * 1.5), got {}",
        dmg.damage
    );
}

/// Behavior 3: Zero speed means do not spawn (return early).
#[test]
fn observer_does_not_spawn_when_speed_is_zero() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0);

    trigger_shockwave(&mut app, bolt, 96.0, 0.0);

    assert_eq!(
        shockwave_entity_count(&mut app),
        0,
        "zero speed should result in no ShockwaveRadius entity"
    );
}

// =========================================================================
// B12c: handle_shockwave observes ShockwaveFired (not EffectFired) (behavior 21)
// =========================================================================

#[test]
fn shockwave_fired_spawns_entity_at_bolt_position() {
    use crate::effect::typed_events::ShockwaveFired;

    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 50.0, 100.0);

    app.world_mut().commands().trigger(ShockwaveFired {
        base_range: 64.0,
        range_per_level: 0.0,
        stacks: 1,
        speed: 400.0,
        targets: vec![crate::effect::definition::EffectTarget::Entity(bolt)],
        source_chip: None,
    });
    app.world_mut().flush();
    tick(&mut app);

    assert_eq!(
        shockwave_entity_count(&mut app),
        1,
        "ShockwaveFired should spawn a ShockwaveRadius entity"
    );
    let sw = get_shockwave_entity(&mut app);
    let radius = app.world().get::<ShockwaveRadius>(sw).unwrap();
    assert!(
        (radius.max - 64.0).abs() < f32::EPSILON,
        "ShockwaveRadius.max should be 64.0, got {}",
        radius.max
    );
}

#[test]
fn shockwave_fired_no_bolt_does_not_spawn() {
    use crate::effect::typed_events::ShockwaveFired;

    let mut app = test_app();

    app.world_mut().commands().trigger(ShockwaveFired {
        base_range: 64.0,
        range_per_level: 0.0,
        stacks: 1,
        speed: 400.0,
        targets: vec![],
        source_chip: None,
    });
    app.world_mut().flush();
    tick(&mut app);

    assert_eq!(
        shockwave_entity_count(&mut app),
        0,
        "ShockwaveFired with bolt: None should not spawn a shockwave"
    );
}
