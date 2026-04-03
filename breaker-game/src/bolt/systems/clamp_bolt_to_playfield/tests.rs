use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::*;
use crate::{
    bolt::components::{Bolt, BoltServing},
    shared::{NodeScalingFactor, PlayfieldConfig},
};

/// Local alias for the CCD epsilon used in test expected values.
///
/// Matches the production `BOUNDARY_INSET` constant and the physics
/// crate's `CCD_EPSILON` -- both are 0.01.
const CCD_EPSILON: f32 = 0.01;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .add_systems(FixedUpdate, clamp_bolt_to_playfield);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Default playfield: width=800, height=600 -> left=-400, right=400, top=300, bottom=-300
const RADIUS: f32 = 6.0;
const TOLERANCE: f32 = 0.001;

/// Spawns a bolt with permissive spatial params for clamping tests.
/// Uses MinSpeed(0.0) and MaxSpeed(MAX) so the velocity formula is a no-op.
fn spawn_test_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(Vec2::new(x, y))
            .with_speed(500.0, 0.0, f32::MAX)
            .with_angle(0.0, 0.0)
            .with_radius(RADIUS)
            .with_velocity(Velocity2D(Vec2::new(vx, vy)))
            .primary()
            .headless()
            .spawn(&mut commands)
    };
    queue.apply(world);
    entity
}

#[test]
fn bolt_inside_bounds_position2d_unchanged() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 100.0, 50.0, 300.0, 400.0);
    tick(&mut app);

    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!((pos.0.x - 100.0).abs() < TOLERANCE);
    assert!((pos.0.y - 50.0).abs() < TOLERANCE);
    assert!((vel.0.x - 300.0).abs() < TOLERANCE);
    assert!((vel.0.y - 400.0).abs() < TOLERANCE);
}

#[test]
fn bolt_past_right_wall_position2d_clamped_vx_flipped() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 500.0, 0.0, 300.0, 400.0);
    tick(&mut app);

    let expected_x = 400.0 - RADIUS - CCD_EPSILON; // 393.99
    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - expected_x).abs() < TOLERANCE,
        "x should be clamped to {expected_x}, got {}",
        pos.0.x
    );
    assert!(
        (vel.0.x - (-300.0)).abs() < TOLERANCE,
        "vx should be flipped to -300, got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < TOLERANCE,
        "vy should be unchanged"
    );
}

#[test]
fn bolt_past_left_wall_position2d_clamped_vx_flipped() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, -500.0, 0.0, -300.0, 400.0);
    tick(&mut app);

    let expected_x = -400.0 + RADIUS + CCD_EPSILON; // -393.99
    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - expected_x).abs() < TOLERANCE,
        "x should be clamped to {expected_x}, got {}",
        pos.0.x
    );
    assert!(
        (vel.0.x - 300.0).abs() < TOLERANCE,
        "vx should be flipped to 300, got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < TOLERANCE,
        "vy should be unchanged"
    );
}

#[test]
fn bolt_past_ceiling_position2d_clamped_vy_flipped() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 0.0, 400.0, 300.0, 400.0);
    tick(&mut app);

    let expected_y = 300.0 - RADIUS - CCD_EPSILON; // 293.99
    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.y - expected_y).abs() < TOLERANCE,
        "y should be clamped to {expected_y}, got {}",
        pos.0.y
    );
    assert!(
        (vel.0.y - (-400.0)).abs() < TOLERANCE,
        "vy should be flipped to -400, got {}",
        vel.0.y
    );
    assert!(
        (vel.0.x - 300.0).abs() < TOLERANCE,
        "vx should be unchanged"
    );
}

#[test]
fn bolt_below_floor_position2d_not_clamped() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 0.0, -500.0, 300.0, -400.0);
    tick(&mut app);

    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.y - (-500.0)).abs() < TOLERANCE,
        "y should NOT be clamped, got {}",
        pos.0.y
    );
    assert!(
        (vel.0.y - (-400.0)).abs() < TOLERANCE,
        "vy should NOT be flipped, got {}",
        vel.0.y
    );
}

#[test]
fn velocity_already_inward_not_flipped_right_wall() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 500.0, 0.0, -300.0, 400.0);
    tick(&mut app);

    let expected_x = 400.0 - RADIUS - CCD_EPSILON;
    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - expected_x).abs() < TOLERANCE,
        "x should be clamped"
    );
    assert!(
        (vel.0.x - (-300.0)).abs() < TOLERANCE,
        "vx already pointing inward should NOT be flipped, got {}",
        vel.0.x
    );
}

#[test]
fn velocity_already_inward_not_flipped_ceiling() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 0.0, 400.0, 300.0, -400.0);
    tick(&mut app);

    let expected_y = 300.0 - RADIUS - CCD_EPSILON;
    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.y - expected_y).abs() < TOLERANCE,
        "y should be clamped"
    );
    assert!(
        (vel.0.y - (-400.0)).abs() < TOLERANCE,
        "vy already pointing inward should NOT be flipped, got {}",
        vel.0.y
    );
}

#[test]
fn corner_escape_both_axes_position2d_clamped() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 500.0, 400.0, 300.0, 400.0);
    tick(&mut app);

    let expected_x = 400.0 - RADIUS - CCD_EPSILON;
    let expected_y = 300.0 - RADIUS - CCD_EPSILON;
    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - expected_x).abs() < TOLERANCE,
        "x should be clamped to {expected_x}, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - expected_y).abs() < TOLERANCE,
        "y should be clamped to {expected_y}, got {}",
        pos.0.y
    );
    assert!(
        (vel.0.x - (-300.0)).abs() < TOLERANCE,
        "vx should be flipped to -300, got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - (-400.0)).abs() < TOLERANCE,
        "vy should be flipped to -400, got {}",
        vel.0.y
    );
}

#[test]
fn serving_bolt_excluded() {
    let mut app = test_app();
    let _entity = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::new(500.0, 0.0))
                .with_speed(500.0, 0.0, f32::MAX)
                .with_angle(0.0, 0.0)
                .with_radius(RADIUS)
                .serving()
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, (With<Bolt>, With<BoltServing>)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - 500.0).abs() < TOLERANCE,
        "serving bolt should NOT be clamped, got {}",
        pos.0.x
    );
}

// --- NodeScalingFactor clamping tests ---

#[test]
fn scaled_bolt_uses_effective_radius_for_playfield_clamping() {
    let mut app = test_app();
    let entity = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::new(500.0, 0.0))
                .with_speed(500.0, 0.0, f32::MAX)
                .with_angle(0.0, 0.0)
                .with_radius(8.0)
                .with_velocity(Velocity2D(Vec2::new(300.0, 400.0)))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    app.world_mut()
        .entity_mut(entity)
        .insert(NodeScalingFactor(0.5));
    tick(&mut app);

    let expected_x_scaled = 400.0 - 4.0 - CCD_EPSILON; // ~395.99
    let expected_x_unscaled = 400.0 - 8.0 - CCD_EPSILON; // ~391.99
    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - expected_x_scaled).abs() < TOLERANCE,
        "scaled bolt should clamp to {expected_x_scaled:.2} (not {expected_x_unscaled:.2}), got {:.2}",
        pos.0.x
    );
}

#[test]
fn bolt_without_entity_scale_in_clamping_is_backward_compatible() {
    let mut app = test_app();
    spawn_test_bolt(&mut app, 500.0, 0.0, 300.0, 400.0);
    tick(&mut app);

    let expected_x = 400.0 - RADIUS - CCD_EPSILON;
    let (pos, vel) = app
        .world_mut()
        .query::<(&Position2D, &Velocity2D)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - expected_x).abs() < TOLERANCE,
        "bolt without NodeScalingFactor should clamp to {expected_x:.2}, got {:.2}",
        pos.0.x
    );
    assert!(
        (vel.0.x - (-300.0)).abs() < TOLERANCE,
        "vx should be flipped to -300, got {:.1}",
        vel.0.x
    );
}
