use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::bolt::components::Bolt;

#[test]
fn bolt_reflects_off_wall() {
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();

    // Place a wall to the right
    spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

    // Bolt moving right toward the wall
    let start_x = 200.0 - 50.0 - bc.radius - 5.0;
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(400.0, 0.1)),
        Position2D(Vec2::new(start_x, 0.0)),
    ));

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.x < 0.0,
        "bolt should reflect off wall, got vx={}",
        vel.0.x
    );
}

#[test]
fn wall_hit_does_not_emit_cell_message() {
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

    spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

    let start_x = 200.0 - 50.0 - bc.radius - 5.0;
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(400.0, 0.1)),
        Position2D(Vec2::new(start_x, 0.0)),
    ));

    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert!(
        hits.0.is_empty(),
        "wall hit should not emit BoltImpactCell message"
    );
}

#[test]
fn cell_hit_preferred_over_farther_wall() {
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

    // Cell closer than wall
    let cell_y = 50.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);
    spawn_wall(&mut app, 0.0, 200.0, 400.0, 50.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(0.0, 400.0)),
        Position2D(Vec2::new(0.0, start_y)),
    ));

    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(hits.0.len(), 1);
    assert_eq!(hits.0[0], cell_entity, "should hit cell, not wall");
}
