use bevy::prelude::*;

use super::*;

#[test]
fn cell_width_half_width() {
    let w = CellWidth::new(70.0);
    assert!((w.half_width() - 35.0).abs() < f32::EPSILON);
}

#[test]
fn cell_height_half_height() {
    let h = CellHeight::new(24.0);
    assert!((h.half_height() - 12.0).abs() < f32::EPSILON);
}

// -- Cell #[require] tests ------------------------------------------------

#[test]
fn cell_require_inserts_spatial2d() {
    use rantzsoft_spatial2d::components::Spatial2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Cell).id();
    app.update();
    assert!(
        app.world().get::<Spatial2D>(entity).is_some(),
        "Cell should auto-insert Spatial2D via #[require]"
    );
}

#[test]
fn cell_require_inserts_cleanup_on_exit_node_state() {
    use rantzsoft_stateflow::CleanupOnExit;

    use crate::state::types::NodeState;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Cell).id();
    app.update();
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(entity)
            .is_some(),
        "Cell should auto-insert CleanupOnExit<NodeState> via #[require]"
    );
}

#[test]
fn cell_require_does_not_insert_interpolate_transform2d() {
    use rantzsoft_spatial2d::components::InterpolateTransform2D;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Cell).id();
    app.update();
    assert!(
        app.world().get::<InterpolateTransform2D>(entity).is_none(),
        "Cell #[require] should NOT auto-insert InterpolateTransform2D (cells are static)"
    );
}

#[test]
fn cell_explicit_values_override_defaults() {
    use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((
            Cell,
            Spatial2D,
            Position2D(Vec2::new(50.0, 100.0)),
            Scale2D { x: 70.0, y: 24.0 },
        ))
        .id();
    app.update();
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("Position2D should be present");
    assert_eq!(
        position.0,
        Vec2::new(50.0, 100.0),
        "explicit Position2D(50.0, 100.0) should be preserved"
    );
    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("Scale2D should be present");
    assert!(
        (scale.x - 70.0).abs() < f32::EPSILON && (scale.y - 24.0).abs() < f32::EPSILON,
        "explicit Scale2D {{ x: 70.0, y: 24.0 }} should be preserved, got {{ x: {}, y: {} }}",
        scale.x,
        scale.y
    );
}

// -- CollisionLayers tests ------------------------------------------------

#[test]
fn cell_collision_layers_have_correct_values() {
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::{BOLT_LAYER, CELL_LAYER};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app
        .world_mut()
        .spawn((Cell, CollisionLayers::new(CELL_LAYER, BOLT_LAYER)))
        .id();
    app.update();
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("Cell should have CollisionLayers");
    assert_eq!(
        layers.membership, CELL_LAYER,
        "Cell membership should be CELL_LAYER (0x{CELL_LAYER:02X}), got 0x{:02X}",
        layers.membership
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "Cell mask should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
        layers.mask
    );
}
