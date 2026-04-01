use bevy::prelude::*;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed,
};

use super::super::core::*;
use crate::bolt::{
    components::{
        Bolt, BoltInitialAngle, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY,
        BoltSpawnOffsetY,
    },
    resources::BoltConfig,
};

// ── Section C: config() Convenience ────────────────────────────

// Behavior 9: config() satisfies Speed + Angle
#[test]
fn from_config_transitions_speed_and_angle() {
    let config = BoltConfig::default();
    let _builder: BoltBuilder<NoPosition, HasSpeed, HasAngle, NoMotion, NoRole> =
        Bolt::builder().config(&config);
}

#[test]
fn from_config_stores_default_speed_values() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&BoltConfig::default())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 400.0).abs() < f32::EPSILON,
        "BaseSpeed from default config should be 400.0, got {}",
        base.0
    );
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!(
        (min.0 - 200.0).abs() < f32::EPSILON,
        "MinSpeed from default config should be 200.0, got {}",
        min.0
    );
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max.0 - 800.0).abs() < f32::EPSILON,
        "MaxSpeed from default config should be 800.0, got {}",
        max.0
    );
}

#[test]
fn from_config_custom_speed_values_propagate() {
    let config = BoltConfig {
        base_speed: 100.0,
        min_speed: 50.0,
        max_speed: 150.0,
        min_angle_horizontal: 10.0,
        min_angle_vertical: 10.0,
        ..BoltConfig::default()
    };
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&config)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 100.0).abs() < f32::EPSILON,
        "BaseSpeed from custom config should be 100.0, got {}",
        base.0
    );
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!(
        (min.0 - 50.0).abs() < f32::EPSILON,
        "MinSpeed from custom config should be 50.0, got {}",
        min.0
    );
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max.0 - 150.0).abs() < f32::EPSILON,
        "MaxSpeed from custom config should be 150.0, got {}",
        max.0
    );
}

// Behavior 10: config() stores bolt-specific params
#[test]
fn from_config_stores_bolt_params_default() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&BoltConfig::default())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 8.0).abs() < f32::EPSILON,
        "BoltRadius from default config should be 8.0, got {}",
        radius.0
    );
    let spawn_offset = world.get::<BoltSpawnOffsetY>(entity).unwrap();
    assert!(
        (spawn_offset.0 - 30.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY from default config should be 30.0, got {}",
        spawn_offset.0
    );
    let respawn_offset = world.get::<BoltRespawnOffsetY>(entity).unwrap();
    assert!(
        (respawn_offset.0 - 30.0).abs() < f32::EPSILON,
        "BoltRespawnOffsetY from default config should be 30.0, got {}",
        respawn_offset.0
    );
    let respawn_angle = world.get::<BoltRespawnAngleSpread>(entity).unwrap();
    assert!(
        (respawn_angle.0 - 0.524).abs() < f32::EPSILON,
        "BoltRespawnAngleSpread from default config should be 0.524, got {}",
        respawn_angle.0
    );
    let initial_angle = world.get::<BoltInitialAngle>(entity).unwrap();
    assert!(
        (initial_angle.0 - 0.26).abs() < f32::EPSILON,
        "BoltInitialAngle from default config should be 0.26, got {}",
        initial_angle.0
    );
}

#[test]
fn from_config_stores_bolt_params_custom() {
    let config = BoltConfig {
        radius: 12.0,
        spawn_offset_y: 40.0,
        respawn_offset_y: 35.0,
        respawn_angle_spread: 0.6,
        initial_angle: 0.3,
        ..BoltConfig::default()
    };
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&config)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 12.0).abs() < f32::EPSILON,
        "BoltRadius should be 12.0, got {}",
        radius.0
    );
    let spawn_offset = world.get::<BoltSpawnOffsetY>(entity).unwrap();
    assert!(
        (spawn_offset.0 - 40.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 40.0, got {}",
        spawn_offset.0
    );
    let respawn_offset = world.get::<BoltRespawnOffsetY>(entity).unwrap();
    assert!(
        (respawn_offset.0 - 35.0).abs() < f32::EPSILON,
        "BoltRespawnOffsetY should be 35.0, got {}",
        respawn_offset.0
    );
    let respawn_angle = world.get::<BoltRespawnAngleSpread>(entity).unwrap();
    assert!(
        (respawn_angle.0 - 0.6).abs() < f32::EPSILON,
        "BoltRespawnAngleSpread should be 0.6, got {}",
        respawn_angle.0
    );
    let initial_angle = world.get::<BoltInitialAngle>(entity).unwrap();
    assert!(
        (initial_angle.0 - 0.3).abs() < f32::EPSILON,
        "BoltInitialAngle should be 0.3, got {}",
        initial_angle.0
    );
}

// Behavior 11: config() converts angle degrees to radians
#[test]
fn from_config_converts_angles_to_radians() {
    let config = BoltConfig {
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        ..BoltConfig::default()
    };
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&config)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let expected_h = 5.0_f32.to_radians();
    assert!(
        (h.0 - expected_h).abs() < 1e-5,
        "MinAngleHorizontal should be {} (5 degrees in radians), got {}",
        expected_h,
        h.0
    );
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    let expected_v = 5.0_f32.to_radians();
    assert!(
        (v.0 - expected_v).abs() < 1e-5,
        "MinAngleVertical should be {} (5 degrees in radians), got {}",
        expected_v,
        v.0
    );
}

#[test]
fn from_config_zero_angles_produce_zero_radians() {
    let config = BoltConfig {
        min_angle_horizontal: 0.0,
        min_angle_vertical: 0.0,
        ..BoltConfig::default()
    };
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&config)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!(
        h.0.abs() < f32::EPSILON,
        "MinAngleHorizontal(0.0) should be 0.0, got {}",
        h.0
    );
    assert!(
        v.0.abs() < f32::EPSILON,
        "MinAngleVertical(0.0) should be 0.0, got {}",
        v.0
    );
}
