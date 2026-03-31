use bevy::prelude::*;

use super::definitions::*;

// ── Bolt component tests (no #[require] — builder handles insertion) ────────

#[test]
fn bolt_does_not_auto_insert_spatial2d() {
    use rantzsoft_spatial2d::components::Spatial2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<Spatial2D>(entity).is_none(),
        "Bolt should NOT auto-insert Spatial2D (builder handles this)"
    );
}

#[test]
fn bolt_does_not_auto_insert_interpolate_transform2d() {
    use rantzsoft_spatial2d::components::InterpolateTransform2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<InterpolateTransform2D>(entity).is_none(),
        "Bolt should NOT auto-insert InterpolateTransform2D (builder handles this)"
    );
}

#[test]
fn bolt_does_not_auto_insert_velocity2d() {
    use rantzsoft_spatial2d::components::Velocity2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<Velocity2D>(entity).is_none(),
        "Bolt should NOT auto-insert Velocity2D (builder handles this)"
    );
}

#[test]
fn bolt_explicit_components_still_work() {
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};
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
        .expect("Velocity2D should be present when explicitly added");
    assert!(
        (velocity.0.x - 0.0).abs() < f32::EPSILON && (velocity.0.y - 400.0).abs() < f32::EPSILON,
        "explicit Velocity2D(0.0, 400.0) should be preserved, got {:?}",
        velocity.0
    );
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("Position2D should be present when explicitly added");
    assert_eq!(
        position.0,
        Vec2::new(10.0, 20.0),
        "explicit Position2D(10.0, 20.0) should be preserved"
    );
}

#[test]
fn bolt_does_not_insert_cleanup_on_run_end() {
    use crate::shared::CleanupOnRunEnd;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<CleanupOnRunEnd>(entity).is_none(),
        "Bolt should NOT auto-insert CleanupOnRunEnd"
    );
}

#[test]
fn bolt_does_not_insert_cleanup_on_node_exit() {
    use crate::shared::CleanupOnNodeExit;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Bolt).id();
    app.update();
    assert!(
        app.world().get::<CleanupOnNodeExit>(entity).is_none(),
        "Bolt should NOT auto-insert CleanupOnNodeExit"
    );
}

// ── PrimaryBolt component tests ───────────────────────────────

#[test]
fn primary_bolt_is_zero_sized() {
    assert_eq!(
        std::mem::size_of::<PrimaryBolt>(),
        0,
        "PrimaryBolt should be a zero-sized type"
    );
}

#[test]
fn primary_bolt_has_debug() {
    let marker = PrimaryBolt;
    let debug_str = format!("{marker:?}");
    assert!(
        !debug_str.is_empty(),
        "PrimaryBolt should have a Debug impl"
    );
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
