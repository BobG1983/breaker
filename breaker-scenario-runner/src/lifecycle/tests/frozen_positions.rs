use super::helpers::*;

// -------------------------------------------------------------------------
// enforce_frozen_positions — resets entity to frozen target each tick
// -------------------------------------------------------------------------

/// Each fixed-update tick, `enforce_frozen_positions` must set the entity's
/// `Position2D` exactly to `ScenarioPhysicsFrozen.target`, regardless
/// of where physics moved it.
///
/// Given target = `(0.0, -500.0)` and current position `(100.0, 200.0)`,
/// after one tick the position must be exactly `(0.0, -500.0)`.
#[test]
fn enforce_frozen_positions_resets_entity_to_frozen_target_each_tick() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(FixedUpdate, enforce_frozen_positions);

    let entity = app
        .world_mut()
        .spawn((
            ScenarioPhysicsFrozen {
                target: Vec2::new(0.0, -500.0),
            },
            Position2D(Vec2::new(100.0, 200.0)),
        ))
        .id();

    tick(&mut app);

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");

    assert_eq!(
        position.0,
        Vec2::new(0.0, -500.0),
        "expected position to be reset to frozen target (0.0, -500.0), got {:?}",
        position.0
    );
}
