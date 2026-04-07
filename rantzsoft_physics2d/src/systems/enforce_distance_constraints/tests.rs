use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::system::*;
use crate::constraint::DistanceConstraint;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(FixedUpdate, enforce_distance_constraints);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// -- Behavior 3: Slack constraint -- positions/velocities unchanged --

#[test]
fn slack_constraint_leaves_positions_unchanged() {
    // Given: A at (0,0), B at (100,0), max_distance=200
    // Distance = 100, well within slack
    // When: enforce_distance_constraints runs
    // Then: positions and velocities remain unchanged
    let mut app = test_app();

    let a = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(100.0, 200.0)),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(100.0, 0.0)),
            Velocity2D(Vec2::new(-50.0, 300.0)),
        ))
        .id();
    app.world_mut().spawn(DistanceConstraint {
        entity_a: a,
        entity_b: b,
        max_distance: 200.0,
    });

    tick(&mut app);

    let pos_a = app.world().get::<Position2D>(a).unwrap();
    let pos_b = app.world().get::<Position2D>(b).unwrap();
    let vel_a = app.world().get::<Velocity2D>(a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(b).unwrap();

    assert!(
        (pos_a.0 - Vec2::new(0.0, 0.0)).length() < f32::EPSILON,
        "A position should be unchanged when slack, got {:?}",
        pos_a.0,
    );
    assert!(
        (pos_b.0 - Vec2::new(100.0, 0.0)).length() < f32::EPSILON,
        "B position should be unchanged when slack, got {:?}",
        pos_b.0,
    );
    assert!(
        (vel_a.0 - Vec2::new(100.0, 200.0)).length() < f32::EPSILON,
        "A velocity should be unchanged when slack, got {:?}",
        vel_a.0,
    );
    assert!(
        (vel_b.0 - Vec2::new(-50.0, 300.0)).length() < f32::EPSILON,
        "B velocity should be unchanged when slack, got {:?}",
        vel_b.0,
    );
}

// -- Behavior 4: Taut constraint -- position correction --

#[test]
fn taut_constraint_corrects_positions_symmetrically() {
    // Given: A at (0,0), B at (300,0), max_distance=200
    // Distance = 300, overshoot = 100
    // When: enforce_distance_constraints runs
    // Then: A=(50,0), B=(250,0) -- each moved 50 units inward
    let mut app = test_app();

    let a = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(300.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 0.0)),
        ))
        .id();
    app.world_mut().spawn(DistanceConstraint {
        entity_a: a,
        entity_b: b,
        max_distance: 200.0,
    });

    tick(&mut app);

    let pos_a = app.world().get::<Position2D>(a).unwrap();
    let pos_b = app.world().get::<Position2D>(b).unwrap();

    assert!(
        (pos_a.0.x - 50.0).abs() < 0.01,
        "A.x should be 50.0 (moved 50 toward B), got {:.2}",
        pos_a.0.x,
    );
    assert!(
        pos_a.0.y.abs() < 0.01,
        "A.y should be 0.0, got {:.2}",
        pos_a.0.y,
    );
    assert!(
        (pos_b.0.x - 250.0).abs() < 0.01,
        "B.x should be 250.0 (moved 50 toward A), got {:.2}",
        pos_b.0.x,
    );
    assert!(
        pos_b.0.y.abs() < 0.01,
        "B.y should be 0.0, got {:.2}",
        pos_b.0.y,
    );
}

// -- Behavior 5: Taut -- velocity redistribution along chain axis --

#[test]
fn taut_constraint_redistributes_velocity_along_axis() {
    // Given: A at (0,0), B at (300,0), max_distance=200
    // A.vel = (400,0), B.vel = (0,0)
    // Chain axis = (1,0) -- purely horizontal
    // When: enforce_distance_constraints runs
    // Then: average axial velocity = 200 for each (momentum conservation)
    let mut app = test_app();

    let a = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(400.0, 0.0)),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(300.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 0.0)),
        ))
        .id();
    app.world_mut().spawn(DistanceConstraint {
        entity_a: a,
        entity_b: b,
        max_distance: 200.0,
    });

    tick(&mut app);

    let vel_a = app.world().get::<Velocity2D>(a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(b).unwrap();

    // Momentum along axis should be conserved: avg = (400+0)/2 = 200 each
    assert!(
        (vel_a.0.x - 200.0).abs() < 1.0,
        "A.vx should be ~200 (redistributed), got {:.1}",
        vel_a.0.x,
    );
    assert!(
        (vel_b.0.x - 200.0).abs() < 1.0,
        "B.vx should be ~200 (redistributed), got {:.1}",
        vel_b.0.x,
    );
}

// -- Behavior 6: Taut -- perpendicular velocity preserved --

#[test]
fn taut_constraint_preserves_perpendicular_velocity() {
    // Given: A at (0,0), B at (300,0), max_distance=200
    // A.vel = (400,100), B.vel = (0,300)
    // Chain axis = (1,0), perpendicular = (0,1)
    // When: enforce_distance_constraints runs
    // Then: A.vy = 100 (unchanged), B.vy = 300 (unchanged)
    let mut app = test_app();

    let a = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(400.0, 100.0)),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(300.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 300.0)),
        ))
        .id();
    app.world_mut().spawn(DistanceConstraint {
        entity_a: a,
        entity_b: b,
        max_distance: 200.0,
    });

    tick(&mut app);

    let vel_a = app.world().get::<Velocity2D>(a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(b).unwrap();

    assert!(
        (vel_a.0.y - 100.0).abs() < 0.01,
        "A.vy should be preserved at 100, got {:.2}",
        vel_a.0.y,
    );
    assert!(
        (vel_b.0.y - 300.0).abs() < 0.01,
        "B.vy should be preserved at 300, got {:.2}",
        vel_b.0.y,
    );
}

// -- Behavior 7: No redistribution when converging --

#[test]
fn taut_constraint_no_velocity_redistribution_when_converging() {
    // Given: A at (0,0), B at (300,0), max_distance=200
    // A.vel = (400,0), B.vel = (-100,0)
    // A at 0 moving right (+400) toward B at 300.
    // B at 300 moving left (-100) toward A.
    // Both converging -- no velocity redistribution, only position correction.
    let mut app = test_app();

    let a = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(400.0, 0.0)),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(300.0, 0.0)),
            Velocity2D(Vec2::new(-100.0, 0.0)),
        ))
        .id();
    app.world_mut().spawn(DistanceConstraint {
        entity_a: a,
        entity_b: b,
        max_distance: 200.0,
    });

    tick(&mut app);

    let vel_a = app.world().get::<Velocity2D>(a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(b).unwrap();

    // Positions should still be corrected, but velocities should NOT be redistributed
    // because the entities are converging along the axis.
    assert!(
        (vel_a.0.x - 400.0).abs() < 0.01,
        "A.vx should be unchanged (converging), got {:.2}",
        vel_a.0.x,
    );
    assert!(
        (vel_b.0.x - (-100.0)).abs() < 0.01,
        "B.vx should be unchanged (converging), got {:.2}",
        vel_b.0.x,
    );
}

// -- Behavior 8: Missing entity -- skip gracefully --

#[test]
fn missing_entity_skipped_gracefully() {
    // Given: constraint referencing a despawned entity
    // When: enforce_distance_constraints runs
    // Then: no panic, surviving entity unchanged
    let mut app = test_app();

    let a = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(100.0, 200.0)),
        ))
        .id();
    let stale_entity = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(stale_entity);
    app.world_mut().spawn(DistanceConstraint {
        entity_a: a,
        entity_b: stale_entity,
        max_distance: 200.0,
    });

    tick(&mut app);

    let pos_a = app.world().get::<Position2D>(a).unwrap();
    assert!(
        (pos_a.0 - Vec2::new(0.0, 0.0)).length() < f32::EPSILON,
        "surviving entity should be unaffected when partner is missing",
    );
}

// -- Behavior 9: Zero distance (same position) -- skip correction --

#[test]
fn zero_distance_same_position_skips_correction() {
    // Given: A and B at the same position (0,0), max_distance=200
    // When: enforce_distance_constraints runs
    // Then: no correction applied, no division by zero, all values finite
    let mut app = test_app();

    let a = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(100.0, 200.0)),
        ))
        .id();
    let b = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(-50.0, 300.0)),
        ))
        .id();
    app.world_mut().spawn(DistanceConstraint {
        entity_a: a,
        entity_b: b,
        max_distance: 200.0,
    });

    tick(&mut app);

    let pos_a = app.world().get::<Position2D>(a).unwrap();
    let pos_b = app.world().get::<Position2D>(b).unwrap();
    let vel_a = app.world().get::<Velocity2D>(a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(b).unwrap();

    assert!(
        pos_a.0.x.is_finite() && pos_a.0.y.is_finite(),
        "A position should be finite, got {:?}",
        pos_a.0,
    );
    assert!(
        pos_b.0.x.is_finite() && pos_b.0.y.is_finite(),
        "B position should be finite, got {:?}",
        pos_b.0,
    );
    assert!(
        vel_a.0.x.is_finite() && vel_a.0.y.is_finite(),
        "A velocity should be finite, got {:?}",
        vel_a.0,
    );
    assert!(
        vel_b.0.x.is_finite() && vel_b.0.y.is_finite(),
        "B velocity should be finite, got {:?}",
        vel_b.0,
    );
}
