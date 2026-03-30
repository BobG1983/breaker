use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use super::super::helpers::*;
use crate::{
    cells::messages::DamageCell,
    shared::{BOLT_LAYER, CELL_LAYER, GameRng, WALL_LAYER},
};

// ── Behavior 7: fire() only targets cells on CELL_LAYER ──

#[test]
fn fire_only_targets_cells_on_cell_layer() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    // Cell on CELL_LAYER
    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    // Wall entity on WALL_LAYER (not a cell)
    let wall_pos = Vec2::new(5.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    // Bolt entity on BOLT_LAYER (not a cell)
    let bolt_pos = Vec2::new(8.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(bolt_pos),
        GlobalPosition2D(bolt_pos),
        Spatial2D,
    ));

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "only CELL_LAYER entity should be targeted"
    );
    assert_eq!(
        written[0].cell, cell,
        "DamageCell should target the CELL_LAYER entity"
    );
}

#[test]
fn fire_targets_entity_with_combined_cell_layer_membership() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    // Entity with CELL_LAYER | BOLT_LAYER
    let pos = Vec2::new(10.0, 0.0);
    let combined = app
        .world_mut()
        .spawn((
            crate::cells::components::Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER | BOLT_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "combined CELL_LAYER entity should be targeted"
    );
    assert_eq!(written[0].cell, combined);
}

// ── Behavior 8: fire() reads position from Position2D (no Transform fallback) ──

#[test]
fn fire_defaults_position_to_zero_with_no_position_or_transform() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn_empty().id();

    // Cell at (10, 0) — within range 50 from origin
    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "cell at (10,0) should be in range from default (0,0)"
    );
    assert_eq!(written[0].cell, cell);
}

#[test]
fn fire_prefers_position2d_over_transform() {
    let mut app = chain_lightning_test_app();

    // Entity with Position2D(50, 50) and Transform at (100, 100)
    // Production code reads Position2D only (no Transform fallback),
    // so Transform is ignored.
    let entity = app
        .world_mut()
        .spawn((
            rantzsoft_spatial2d::components::Position2D(Vec2::new(50.0, 50.0)),
            Transform::from_xyz(100.0, 100.0, 0.0),
        ))
        .id();

    // Cell at (60, 50) — 10 units from Position2D, out of range from Transform origin
    let cell = spawn_test_cell(&mut app, 60.0, 50.0);

    tick(&mut app);

    fire(entity, 1, 15.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();

    assert_eq!(
        written.len(),
        1,
        "should use Position2D (50,50), cell at (60,50) is 10 units away, within range 15"
    );
    assert_eq!(written[0].cell, cell);
}

// ── Behavior 10: fire() uses GameRng deterministically ──

#[test]
fn fire_uses_game_rng_deterministically() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 0.0, 10.0);
    let _cell_c = spawn_test_cell(&mut app, -10.0, 0.0);

    tick(&mut app);

    // First call with seed 42
    app.world_mut().insert_resource(GameRng::from_seed(42));
    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    let first_written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(first_written.len(), 1, "first fire should damage one cell");
    let first_target = first_written[0].cell;

    // Reset by creating a new app with same setup for clean message state
    let mut app2 = chain_lightning_test_app();

    let entity2 = app2.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a2 = spawn_test_cell(&mut app2, 10.0, 0.0);
    let _cell_b2 = spawn_test_cell(&mut app2, 0.0, 10.0);
    let _cell_c2 = spawn_test_cell(&mut app2, -10.0, 0.0);

    tick(&mut app2);

    // Same seed
    app2.world_mut().insert_resource(GameRng::from_seed(42));
    fire(entity2, 1, 50.0, 1.0, 200.0, "", app2.world_mut());

    let messages2 = app2.world().resource::<Messages<DamageCell>>();
    let second_written: Vec<&DamageCell> = messages2.iter_current_update_messages().collect();
    assert_eq!(
        second_written.len(),
        1,
        "second fire should damage one cell"
    );
    let second_target = second_written[0].cell;

    // Entity IDs may differ across apps, but with same entity spawn order they should be the same
    assert_eq!(
        first_target, second_target,
        "same RNG seed should produce same target selection"
    );
}
