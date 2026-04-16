use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::helpers::{cells_plugin_app, sequence_plugin_app_loading, tick_cells};
use crate::{
    cells::behaviors::magnetic::components::{MagneticCell, MagneticField},
    prelude::*,
};

/// Behavior 32: `CellsPlugin` registers `apply_magnetic_fields` in
/// `FixedUpdate` with `run_if(NodeState::Playing)`.
///
/// Given: Magnetic cell at origin, bolt at (50, 0) with velocity (0, 400).
/// When: one tick in `NodeState::Playing`.
/// Then: bolt velocity x becomes negative (pulled toward magnet).
#[test]
fn cells_plugin_registers_apply_magnetic_fields_in_playing() {
    let mut app = cells_plugin_app();

    // Spawn magnetic cell at origin
    app.world_mut().spawn((
        Cell,
        MagneticCell,
        MagneticField {
            radius:   200.0,
            strength: 1000.0,
        },
        Position2D(Vec2::ZERO),
        Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
        Hp::new(20.0),
        KilledBy::default(),
    ));

    // Spawn bolt at (50, 0) with velocity (0, 400)
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(50.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
        ))
        .id();

    tick_cells(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.x < 0.0,
        "CellsPlugin should register apply_magnetic_fields; bolt should be pulled toward magnet, got vx={}",
        vel.0.x
    );
}

/// Behavior 32 edge (control): same setup but in `NodeState::Loading` --
/// velocity should remain unchanged, proving `run_if` gate works.
#[test]
fn cells_plugin_magnetic_does_not_run_in_loading_state() {
    let mut app = sequence_plugin_app_loading();

    // Spawn magnetic cell at origin
    app.world_mut().spawn((
        Cell,
        MagneticCell,
        MagneticField {
            radius:   200.0,
            strength: 1000.0,
        },
        Position2D(Vec2::ZERO),
        Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
        Hp::new(20.0),
        KilledBy::default(),
    ));

    // Spawn bolt
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(Vec2::new(50.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
        ))
        .id();

    // Do NOT advance to playing -- tick in Loading state
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON,
        "magnetic should NOT run in Loading state, got vx={}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "magnetic should NOT run in Loading state, got vy={}",
        vel.0.y
    );
}
