use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use super::{super::effect::*, helpers::*};
use crate::{bolt::components::Bolt, shared::playing_state::PlayingState};

#[test]
fn apply_gravity_pull_steers_bolt_toward_well_within_radius() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Gravity well at origin with large radius
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 200.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::ZERO),
    ));

    // Bolt at (100, 0) with zero velocity — should be pulled toward (0,0)
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::new(100.0, 0.0)),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // Bolt should have been pulled in the -x direction (toward the well)
    assert!(
        velocity.x < 0.0,
        "bolt velocity x should be negative (pulled toward well), got {}",
        velocity.x
    );
}

// ── apply_gravity_pull uses Position2D ──────────────────────

#[test]
fn apply_gravity_pull_uses_position2d_for_bolt_distance() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Gravity well at Position2D origin with deliberately different Transform.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 200.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::new(0.0, 0.0)),
        Transform::from_xyz(999.0, 999.0, 0.0),
    ));

    // Bolt at Position2D (100, 0) with deliberately different Transform.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::new(100.0, 0.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // Bolt should be pulled toward (0,0) via Position2D, not toward Transform positions.
    // Direction from bolt (100,0) to well (0,0) = (-1, 0), so velocity.x should be negative.
    assert!(
        velocity.x < 0.0,
        "bolt should be pulled in -x direction toward well at Position2D (0,0), got velocity.x = {}",
        velocity.x
    );
}

#[test]
fn apply_gravity_pull_uses_position2d_for_well_position_not_transform() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Well at Position2D (0,0) but Transform at (500, 500). If the system reads
    // Transform, the bolt at (100, 0) would be pulled toward (500, 500) instead of (0,0).
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 200.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::new(0.0, 0.0)),
        Transform::from_xyz(500.0, 500.0, 0.0),
    ));

    // Bolt at Position2D (100, 0).
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::new(100.0, 0.0)),
            Transform::from_xyz(100.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // If using Position2D: delta = (0,0) - (100,0) = (-100, 0) -> bolt pulled in -x.
    assert!(
        velocity.x < 0.0,
        "bolt should be pulled toward well at Position2D (0,0), not Transform (500,500). Got velocity.x = {}",
        velocity.x
    );
}

#[test]
fn apply_gravity_pull_uses_position2d_radius_check_not_transform() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Well at Position2D (200, 0) with small radius 50. Transform at (30, 0).
    // By Position2D, bolt at (0,0) is 200 units away (outside radius 50).
    // By Transform, bolt at (0,0) would appear 30 units away (inside radius 50).
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 50.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::new(200.0, 0.0)),
        Transform::from_xyz(30.0, 0.0, 0.0),
    ));

    // Bolt at Position2D origin.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::ZERO),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // Bolt is outside radius by Position2D distance (200 > 50), should NOT be pulled.
    assert!(
        velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
        "bolt should NOT be pulled when outside radius by Position2D distance. Got velocity = ({}, {})",
        velocity.x,
        velocity.y
    );
}

#[test]
fn apply_gravity_pull_does_not_pull_bolt_outside_radius_by_position2d() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Well at Position2D origin, radius 50. Transform at (0,0) matches.
    // Bolt at Position2D (100, 0) = distance 100 > radius 50.
    // Transform at (10, 0) would be within radius if the system reads Transform.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 50.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::ZERO),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::new(100.0, 0.0)),
            Transform::from_xyz(10.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // By Position2D: distance 100 > radius 50 -> no pull.
    // By Transform: distance 10 < radius 50 -> would incorrectly pull.
    assert!(
        velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
        "bolt at Position2D distance 100 should NOT be pulled (radius 50). Got velocity = ({}, {})",
        velocity.x,
        velocity.y
    );
}

#[test]
fn apply_gravity_pull_pulls_bolt_at_exact_radius_boundary() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Well at Position2D origin, radius 50.
    // Bolt at Position2D (50, 0) = distance exactly 50 (inside).
    // Transform at (999, 0) — if system reads Transform, distance 999 > 50, would NOT pull.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 50.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::ZERO),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::new(50.0, 0.0)),
            Transform::from_xyz(999.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // By Position2D: distance 50.0 <= radius 50.0 -> should pull in -x direction.
    // By Transform: distance 999.0 > radius 50.0 -> would not pull.
    assert!(
        velocity.x < 0.0,
        "bolt at exact radius boundary by Position2D (50.0) should be pulled toward well. Got velocity.x = {}",
        velocity.x
    );
}

#[test]
fn apply_gravity_pull_no_pull_when_bolt_at_same_position2d_as_well() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Well and bolt both at Position2D origin — distance = 0, no pull.
    // Transform values differ to ensure the system reads Position2D.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 200.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::ZERO),
        Transform::from_xyz(100.0, 0.0, 0.0),
    ));

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::ZERO),
            Transform::from_xyz(50.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // By Position2D: distance = 0, guard prevents pull.
    // By Transform: distance = 50, would incorrectly pull.
    assert!(
        velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
        "bolt at same Position2D as well (distance 0) should not be pulled. Got velocity = ({}, {})",
        velocity.x,
        velocity.y
    );
}

#[test]
fn apply_gravity_pull_skips_well_without_position2d() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Well with NO Position2D — only Transform. The query should not match this well.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 200.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::new(10.0, 0.0)),
            Transform::from_xyz(10.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // Well lacks Position2D, so it should not appear in the query.
    assert!(
        velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
        "well without Position2D should not affect bolt. Got velocity = ({}, {})",
        velocity.x,
        velocity.y
    );
}

#[test]
fn apply_gravity_pull_only_well_with_position2d_affects_bolt() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Well A: has Position2D at (0, 0), radius 200 — should affect bolt.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 200.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::ZERO),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Well B: NO Position2D, only Transform — should be skipped.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 500.0,
            radius: 200.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Bolt at Position2D (100, 0) — within radius of well A.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::ZERO),
            Position2D(Vec2::new(100.0, 0.0)),
            Transform::from_xyz(100.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // Only well A (with Position2D) should pull the bolt.
    // The bolt should be pulled in -x direction toward well A at (0,0).
    assert!(
        velocity.x < 0.0,
        "bolt should be pulled by well with Position2D. Got velocity.x = {}",
        velocity.x
    );
}

// ── Regression: apply_gravity_pull must run before speed clamp ───

/// Regression: `apply_gravity_pull` adds velocity after speed clamp, allowing
/// bolt speed to exceed `BoltMaxSpeed`.
///
/// Given: Bolt at max speed (600.0) heading upward, gravity well at (0, 200)
///        pulling bolt upward (same direction), strength 5000.0.
/// When: Both `apply_gravity_pull` and `prepare_bolt_velocity` run in the same
///        `FixedUpdate` tick.
/// Then: Final bolt speed is at most `BoltMaxSpeed` (600.0).
///
/// This test FAILS if `apply_gravity_pull` runs after the speed clamp (the bug).
/// The fix: add `.before(BoltSystems::PrepareVelocity)` to `register()` so
/// the speed clamp always catches velocity added by gravity pull.
///
/// Scheduling note: uses production `register()` for gravity well systems.
/// `prepare_bolt_velocity` is registered FIRST so that without an explicit
/// `.before()` constraint, Bevy's topological sort may place gravity pull
/// after the clamp, reproducing the bug.
#[test]
fn apply_gravity_pull_is_ordered_before_prepare_velocity() {
    use crate::{
        bolt::{
            BoltSystems,
            components::{Bolt, BoltMaxSpeed, BoltMinSpeed},
            systems::prepare_bolt_velocity,
        },
        breaker::components::{Breaker, MinAngleFromHorizontal},
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<crate::shared::game_state::GameState>();
    app.add_sub_state::<PlayingState>();

    // Register bolt speed clamping FIRST.
    app.add_systems(
        FixedUpdate,
        prepare_bolt_velocity
            .in_set(BoltSystems::PrepareVelocity)
            .run_if(in_state(PlayingState::Active)),
    );

    // Register gravity well systems via production register() SECOND.
    // Without .before(BoltSystems::PrepareVelocity), apply_gravity_pull
    // may run after the speed clamp.
    register(&mut app);

    // Enter Playing state
    app.world_mut()
        .resource_mut::<NextState<crate::shared::game_state::GameState>>()
        .set(crate::shared::game_state::GameState::Playing);
    app.update();

    // Spawn breaker with MinAngleFromHorizontal (required by prepare_bolt_velocity)
    app.world_mut()
        .spawn((Breaker, MinAngleFromHorizontal(15.0_f32.to_radians())));

    let max_speed = 600.0_f32;

    // Gravity well above bolt, pulling bolt upward (same direction as velocity).
    // Very high strength to ensure measurable pull in one tick.
    app.world_mut().spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength: 5000.0,
            radius: 500.0,
            remaining: 10.0,
            owner: Entity::PLACEHOLDER,
        },
        Position2D(Vec2::new(0.0, 200.0)),
    ));

    // Bolt already at max speed heading upward, positioned at (0,0)
    // within the gravity well radius.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, max_speed)),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(max_speed),
            Position2D(Vec2::ZERO),
        ))
        .id();

    // Tick one fixed update
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    let final_speed = velocity.speed();
    assert!(
        final_speed <= max_speed + 1.0,
        "bolt speed ({final_speed:.1}) should not exceed BoltMaxSpeed ({max_speed:.1}) \
         after gravity pull + speed clamp — apply_gravity_pull must be ordered \
         before BoltSystems::PrepareVelocity"
    );
}
