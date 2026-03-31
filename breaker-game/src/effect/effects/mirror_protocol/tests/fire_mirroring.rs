//! Tests for `fire()` side-dependent mirror position and velocity computation.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt, ImpactSide, LastImpact},
        resources::BoltConfig,
    },
    shared::rng::GameRng,
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

// -- Behavior 1: fire() spawns mirrored bolts with top impact --

#[test]
fn fire_spawns_mirrored_bolts_at_reflected_position_top_impact() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(bolt_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(
        bolts.len(),
        1,
        "expected 1 mirrored bolt spawned, got {}",
        bolts.len()
    );

    let bolt = bolts[0];
    let pos = world
        .get::<Position2D>(bolt)
        .expect("mirrored bolt should have Position2D");
    // Top impact flips X: 2*50 - 60 = 40, Y unchanged: 250
    assert_eq!(
        pos.0,
        Vec2::new(40.0, 250.0),
        "mirrored bolt position should be (40, 250) for top impact, got {:?}",
        pos.0
    );

    let vel = world
        .get::<Velocity2D>(bolt)
        .expect("mirrored bolt should have Velocity2D");
    // Top impact negates X: -100, Y unchanged: 400
    assert_eq!(
        vel.0,
        Vec2::new(-100.0, 400.0),
        "mirrored bolt velocity should be (-100, 400) for top impact, got {:?}",
        vel.0
    );
}

// -- Behavior 2: Top impact flips X position and negates X velocity --

#[test]
fn top_impact_mirrors_x_position_and_negates_x_velocity() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(70.0, 300.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query =
        world.query_filtered::<(&Position2D, &Velocity2D), (With<Bolt>, With<ExtraBolt>)>();
    let (pos, vel) = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");
    // Formula: (2*50 - 70, 300) = (30, 300)
    assert_eq!(pos.0, Vec2::new(30.0, 300.0), "top impact mirror position");
    // X negated, Y unchanged
    assert_eq!(
        vel.0,
        Vec2::new(-100.0, 400.0),
        "top impact mirror velocity"
    );
}

// -- Behavior 3: Bottom impact flips X position and negates X velocity --

#[test]
fn bottom_impact_mirrors_x_position_and_negates_x_velocity() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(70.0, 300.0)),
            Velocity2D(Vec2::new(100.0, -400.0)),
            LastImpact {
                position: Vec2::new(50.0, 350.0),
                side: ImpactSide::Bottom,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query =
        world.query_filtered::<(&Position2D, &Velocity2D), (With<Bolt>, With<ExtraBolt>)>();
    let (pos, vel) = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");
    // Formula: (2*50 - 70, 300) = (30, 300)
    assert_eq!(
        pos.0,
        Vec2::new(30.0, 300.0),
        "bottom impact mirror position"
    );
    // X negated, Y unchanged
    assert_eq!(
        vel.0,
        Vec2::new(-100.0, -400.0),
        "bottom impact mirror velocity"
    );
}

// -- Behavior 4: Left impact flips Y position and negates Y velocity --

#[test]
fn left_impact_mirrors_y_position_and_negates_y_velocity() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(300.0, 70.0)),
            Velocity2D(Vec2::new(-400.0, 100.0)),
            LastImpact {
                position: Vec2::new(250.0, 50.0),
                side: ImpactSide::Left,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query =
        world.query_filtered::<(&Position2D, &Velocity2D), (With<Bolt>, With<ExtraBolt>)>();
    let (pos, vel) = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");
    // Formula: (300, 2*50 - 70) = (300, 30)
    assert_eq!(pos.0, Vec2::new(300.0, 30.0), "left impact mirror position");
    // X unchanged, Y negated
    assert_eq!(
        vel.0,
        Vec2::new(-400.0, -100.0),
        "left impact mirror velocity"
    );
}

// -- Behavior 5: Right impact flips Y position and negates Y velocity --

#[test]
fn right_impact_mirrors_y_position_and_negates_y_velocity() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(300.0, 70.0)),
            Velocity2D(Vec2::new(400.0, 100.0)),
            LastImpact {
                position: Vec2::new(350.0, 50.0),
                side: ImpactSide::Right,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query =
        world.query_filtered::<(&Position2D, &Velocity2D), (With<Bolt>, With<ExtraBolt>)>();
    let (pos, vel) = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");
    // Formula: (300, 2*50 - 70) = (300, 30)
    assert_eq!(
        pos.0,
        Vec2::new(300.0, 30.0),
        "right impact mirror position"
    );
    // X unchanged, Y negated
    assert_eq!(
        vel.0,
        Vec2::new(400.0, -100.0),
        "right impact mirror velocity"
    );
}

// -- Behavior 13: Spawned bolt has deterministic velocity (not random) --

#[test]
fn spawned_bolt_has_deterministic_mirror_velocity_not_random() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let vel = query
        .iter(&world)
        .next()
        .expect("should have 1 spawned bolt");

    assert_eq!(
        vel.0,
        Vec2::new(-100.0, 400.0),
        "velocity should be deterministic mirror (-100, 400), not random from GameRng"
    );
}

// -- Behavior 15: Straight up velocity with top impact (X=0) --

#[test]
fn straight_up_velocity_with_top_impact_mirrors_to_same_position() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(100.0, 200.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            LastImpact {
                position: Vec2::new(100.0, 80.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query =
        world.query_filtered::<(&Position2D, &Velocity2D), (With<Bolt>, With<ExtraBolt>)>();
    let (pos, vel) = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");
    // Mirror position: (2*100 - 100, 200) = (100, 200)
    assert_eq!(
        pos.0,
        Vec2::new(100.0, 200.0),
        "mirror position when X is centered on impact"
    );
    // Mirror velocity: (-0.0, 400.0) -- negating 0.0 yields 0.0 (or -0.0)
    assert!(
        (vel.0.x).abs() < f32::EPSILON && (vel.0.y - 400.0).abs() < f32::EPSILON,
        "mirror velocity should be (0.0, 400.0), got {:?}",
        vel.0
    );
}

// -- Behavior 16: Straight right velocity with left impact (Y=0) --

#[test]
fn straight_right_velocity_with_left_impact_mirrors_to_same_position() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(200.0, 100.0)),
            Velocity2D(Vec2::new(400.0, 0.0)),
            LastImpact {
                position: Vec2::new(80.0, 100.0),
                side: ImpactSide::Left,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query =
        world.query_filtered::<(&Position2D, &Velocity2D), (With<Bolt>, With<ExtraBolt>)>();
    let (pos, vel) = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");
    // Mirror position: (200, 2*100 - 100) = (200, 100)
    assert_eq!(
        pos.0,
        Vec2::new(200.0, 100.0),
        "mirror position when Y is centered on impact"
    );
    // Mirror velocity: (400.0, -0.0) -- negating 0.0 yields 0.0 (or -0.0)
    assert!(
        (vel.0.x - 400.0).abs() < f32::EPSILON && (vel.0.y).abs() < f32::EPSILON,
        "mirror velocity should be (400.0, 0.0), got {:?}",
        vel.0
    );
}
