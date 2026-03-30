//! Tests for collision layer filtering in `tick_tether_beam`.

use super::super::*;

#[test]
fn tick_tether_beam_skips_entities_outside_cell_layer() {
    let mut app = damage_test_app();

    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Spawn a bolt-layer entity at (50, 0) — should NOT be damaged
    let pos = Vec2::new(50.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(pos),
        GlobalPosition2D(pos),
        Spatial2D,
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "non-CELL_LAYER entities should not receive DamageCell"
    );
}

#[test]
fn tick_tether_beam_damages_entity_with_cell_layer_in_combined_layers() {
    let mut app = damage_test_app();

    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Entity with CELL_LAYER | BOLT_LAYER — IS on CELL_LAYER, so should be damaged
    let pos = Vec2::new(50.0, 0.0);
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            CollisionLayers::new(CELL_LAYER | BOLT_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "entity with CELL_LAYER in combined mask should be damaged"
    );
    assert_eq!(collector.0[0].cell, cell);
}
