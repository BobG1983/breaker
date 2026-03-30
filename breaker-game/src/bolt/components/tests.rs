use super::definitions::*;
use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

// ── Bolt #[require] tests ────────────────────────────────────

#[test]
fn bolt_require_inserts_spatial2d() {
    use rantzsoft_spatial2d::components::Spatial2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<Spatial2D>(entity).is_some(),
        "Bolt should auto-insert Spatial2D via #[require]"
    );
}

#[test]
fn bolt_require_inserts_interpolate_transform2d() {
    use rantzsoft_spatial2d::components::InterpolateTransform2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<InterpolateTransform2D>(entity).is_some(),
        "Bolt should auto-insert InterpolateTransform2D via #[require]"
    );
}

#[test]
fn bolt_require_inserts_bolt_velocity_default() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    let velocity = app
        .world()
        .get::<Velocity2D>(entity)
        .expect("Bolt should auto-insert Velocity2D via #[require]");
    assert_eq!(
        velocity.0,
        Vec2::ZERO,
        "default Velocity2D should have value Vec2::ZERO"
    );
}

#[test]
fn bolt_explicit_values_override_require_defaults() {
    use rantzsoft_spatial2d::components::Position2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    app.update();
    let velocity = app
        .world()
        .get::<Velocity2D>(entity)
        .expect("Velocity2D should be present");
    assert!(
        (velocity.0.x - 0.0).abs() < f32::EPSILON
            && (velocity.0.y - 400.0).abs() < f32::EPSILON,
        "explicit Velocity2D(0.0, 400.0) should override the default, got {:?}",
        velocity.0
    );
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("Position2D should be present");
    assert_eq!(
        position.0,
        Vec2::new(10.0, 20.0),
        "explicit Position2D(10.0, 20.0) should override the default"
    );
}

#[test]
fn bolt_require_does_not_insert_cleanup_on_run_end() {
    use crate::shared::CleanupOnRunEnd;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<CleanupOnRunEnd>(entity).is_none(),
        "Bolt #[require] should NOT auto-insert CleanupOnRunEnd"
    );
}

#[test]
fn bolt_require_does_not_insert_cleanup_on_node_exit() {
    use crate::shared::CleanupOnNodeExit;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<CleanupOnNodeExit>(entity).is_none(),
        "Bolt #[require] should NOT auto-insert CleanupOnNodeExit"
    );
}

// ── Velocity2D migration tests ────────────────────────────────

#[test]
fn bolt_require_inserts_velocity2d_default() {
    use rantzsoft_spatial2d::components::Velocity2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    let velocity = app
        .world()
        .get::<Velocity2D>(entity)
        .expect("Bolt should auto-insert Velocity2D via #[require]");
    assert_eq!(
        velocity.0,
        Vec2::ZERO,
        "default Velocity2D should be Vec2::ZERO"
    );
}

// ── enforce_min_angle free function tests ─────────────────────

#[test]
fn free_enforce_min_angle_corrects_shallow() {
    use std::f32::consts::FRAC_PI_4;
    let mut velocity = Vec2::new(10.0, 0.01);
    let speed_before = velocity.length();
    enforce_min_angle(&mut velocity, FRAC_PI_4);
    let speed_after = velocity.length();
    assert!(
        (speed_before - speed_after).abs() < 1e-4,
        "speed should be preserved: before={speed_before}, after={speed_after}"
    );
    let angle = velocity.y.abs().atan2(velocity.x.abs());
    assert!(
        angle >= FRAC_PI_4 - 1e-4,
        "angle {angle} should be >= PI/4 ({FRAC_PI_4})"
    );
}

#[test]
fn free_enforce_min_angle_preserves_signs() {
    use std::f32::consts::FRAC_PI_4;
    let mut velocity = Vec2::new(-10.0, -0.01);
    enforce_min_angle(&mut velocity, FRAC_PI_4);
    assert!(
        velocity.x < 0.0,
        "x sign should be negative, got {}",
        velocity.x
    );
    assert!(
        velocity.y < 0.0,
        "y sign should be negative, got {}",
        velocity.y
    );
}

#[test]
fn free_enforce_min_angle_leaves_steep_unchanged() {
    use crate::breaker::resources::BreakerConfig;
    let mut velocity = Vec2::new(1.0, 5.0);
    let original = velocity;
    enforce_min_angle(
        &mut velocity,
        BreakerConfig::default()
            .min_angle_from_horizontal
            .to_radians(),
    );
    assert!(
        (velocity.x - original.x).abs() < 1e-6,
        "steep velocity x should be unchanged"
    );
    assert!(
        (velocity.y - original.y).abs() < 1e-6,
        "steep velocity y should be unchanged"
    );
}

#[test]
fn free_enforce_min_angle_zero_velocity_unchanged() {
    use std::f32::consts::FRAC_PI_4;
    let mut velocity = Vec2::ZERO;
    enforce_min_angle(&mut velocity, FRAC_PI_4);
    assert_eq!(velocity, Vec2::ZERO, "zero velocity should remain zero");
}

// ── CollisionLayers tests ──────────────────────────────────────

#[test]
fn bolt_collision_layers_have_correct_values() {
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        ))
        .id();
    app.update();
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("Bolt should have CollisionLayers");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "Bolt membership should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "Bolt mask should be CELL|WALL|BREAKER (0x{:02X}), got 0x{:02X}",
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        layers.mask
    );
}
