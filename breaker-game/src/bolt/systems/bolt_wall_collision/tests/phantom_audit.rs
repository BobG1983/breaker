use bevy::prelude::Vec2;

use super::helpers::*;
use crate::{
    bolt::components::{ImpactSide, LastImpact, PhantomBolt},
    prelude::*,
};

// ── T8 — Phantom bolt behaves identically to normal bolt at walls ──────────

// Sub-case A: Left wall
// Wall at center (-5, 200) half-extents (5, 400) → world-x [-10, 0].
// Expanded AABB (radius 8): x [-18, 8]. Bolt at (-2, 200) with vx=-400 overlaps.
// Nearest face: right face at x=8 (distance 10). Push-out normal Vec2::X.
// Push-out position: wall_cx + wall_half_x + bolt_radius = -5 + 5 + 8 = 8. → x=8.0
#[test]
fn phantom_bolt_reflects_off_left_wall_identically_to_normal_bolt() {
    let mut app_a = test_app();
    let wall_a = spawn_wall(&mut app_a, -5.0, 200.0, 5.0, 400.0);
    let bolt_a = spawn_bolt(&mut app_a, -2.0, 200.0, -400.0, 0.0);

    let mut app_b = test_app();
    let wall_b = spawn_wall(&mut app_b, -5.0, 200.0, 5.0, 400.0);
    let bolt_b = spawn_bolt(&mut app_b, -2.0, 200.0, -400.0, 0.0);
    app_b.world_mut().entity_mut(bolt_b).insert(PhantomBolt);

    tick(&mut app_a);
    tick(&mut app_b);

    // Both emit exactly one BoltImpactWall with correct entity refs
    let msgs_a = app_a.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs_a.0.len(),
        1,
        "app_a: expected 1 BoltImpactWall, got {}",
        msgs_a.0.len()
    );
    assert_eq!(msgs_a.0[0].bolt, bolt_a, "app_a: msg.bolt should be bolt_a");
    assert_eq!(msgs_a.0[0].wall, wall_a, "app_a: msg.wall should be wall_a");

    let msgs_b = app_b.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs_b.0.len(),
        1,
        "app_b: expected 1 BoltImpactWall, got {}",
        msgs_b.0.len()
    );
    assert_eq!(msgs_b.0[0].bolt, bolt_b, "app_b: msg.bolt should be bolt_b");
    assert_eq!(msgs_b.0[0].wall, wall_b, "app_b: msg.wall should be wall_b");

    // Both reflect rightward (positive x)
    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;
    assert!(
        vel_a.x > 0.0,
        "normal bolt vx should be positive after left wall reflect, got {}",
        vel_a.x
    );
    assert!(
        vel_b.x > 0.0,
        "phantom bolt vx should be positive after left wall reflect, got {}",
        vel_b.x
    );

    // Velocities are equivalent between phantom and normal bolt
    assert!(
        (vel_a.x - vel_b.x).abs() < 1e-3,
        "vx must be equal between normal and phantom bolt: normal={}, phantom={}",
        vel_a.x,
        vel_b.x
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must be equal between normal and phantom bolt: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );

    // Push-out position is identical and load-bearing
    let pos_a = app_a.world().get::<Position2D>(bolt_a).unwrap().0;
    let pos_b = app_b.world().get::<Position2D>(bolt_b).unwrap().0;
    assert!(
        (pos_a.x - 8.0).abs() < 1e-3,
        "normal bolt x should be pushed out to 8.0, got {}",
        pos_a.x
    );
    assert!(
        (pos_b.x - 8.0).abs() < 1e-3,
        "phantom bolt x should be pushed out to 8.0, got {}",
        pos_b.x
    );
    assert!(
        (pos_a.y - 200.0).abs() < 1e-3,
        "normal bolt y should remain 200.0, got {}",
        pos_a.y
    );
    assert!(
        (pos_b.y - 200.0).abs() < 1e-3,
        "phantom bolt y should remain 200.0, got {}",
        pos_b.y
    );
}

// Sub-case B: Right wall
// Wall at center (5, 200) half-extents (5, 400) → world-x [0, 10].
// Expanded AABB: x [-8, 18]. Bolt at (2, 200) with vx=+400 overlaps.
// Nearest face: left face at x=-8 (distance 10). Push-out normal Vec2::NEG_X.
// Push-out position: wall_cx - wall_half_x - bolt_radius = 5 - 5 - 8 = -8. → x=-8.0
#[test]
fn phantom_bolt_reflects_off_right_wall_identically_to_normal_bolt() {
    let mut app_a = test_app();
    let wall_a = spawn_wall(&mut app_a, 5.0, 200.0, 5.0, 400.0);
    let bolt_a = spawn_bolt(&mut app_a, 2.0, 200.0, 400.0, 0.0);

    let mut app_b = test_app();
    let wall_b = spawn_wall(&mut app_b, 5.0, 200.0, 5.0, 400.0);
    let bolt_b = spawn_bolt(&mut app_b, 2.0, 200.0, 400.0, 0.0);
    app_b.world_mut().entity_mut(bolt_b).insert(PhantomBolt);

    tick(&mut app_a);
    tick(&mut app_b);

    let msgs_a = app_a.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs_a.0.len(),
        1,
        "app_a: expected 1 BoltImpactWall, got {}",
        msgs_a.0.len()
    );
    assert_eq!(msgs_a.0[0].bolt, bolt_a);
    assert_eq!(msgs_a.0[0].wall, wall_a);

    let msgs_b = app_b.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs_b.0.len(),
        1,
        "app_b: expected 1 BoltImpactWall, got {}",
        msgs_b.0.len()
    );
    assert_eq!(msgs_b.0[0].bolt, bolt_b);
    assert_eq!(msgs_b.0[0].wall, wall_b);

    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;
    assert!(
        vel_a.x < 0.0,
        "normal bolt vx should be negative after right wall reflect, got {}",
        vel_a.x
    );
    assert!(
        vel_b.x < 0.0,
        "phantom bolt vx should be negative after right wall reflect, got {}",
        vel_b.x
    );
    assert!(
        (vel_a.x - vel_b.x).abs() < 1e-3,
        "vx must be equal between normal and phantom bolt: normal={}, phantom={}",
        vel_a.x,
        vel_b.x
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must be equal: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );

    let pos_a = app_a.world().get::<Position2D>(bolt_a).unwrap().0;
    let pos_b = app_b.world().get::<Position2D>(bolt_b).unwrap().0;
    assert!(
        (pos_a.x - (-8.0)).abs() < 1e-3,
        "normal bolt x should be pushed to -8.0, got {}",
        pos_a.x
    );
    assert!(
        (pos_b.x - (-8.0)).abs() < 1e-3,
        "phantom bolt x should be pushed to -8.0, got {}",
        pos_b.x
    );
}

// Sub-case C: Top wall
// Wall at center (0, 305) half-extents (400, 5) → world-y [300, 310].
// Expanded AABB: y [292, 318]. Bolt at (0, 298) with vy=+400 overlaps.
// Nearest face: bottom face at y=292 (distance |298-292|=6). Push-out normal Vec2::NEG_Y.
// Push-out position: wall_cy - wall_half_y - bolt_radius = 305 - 5 - 8 = 292. → y=292.0
#[test]
fn phantom_bolt_reflects_off_top_wall_identically_to_normal_bolt() {
    let mut app_a = test_app();
    let wall_a = spawn_wall(&mut app_a, 0.0, 305.0, 400.0, 5.0);
    let bolt_a = spawn_bolt(&mut app_a, 0.0, 298.0, 0.0, 400.0);

    let mut app_b = test_app();
    let wall_b = spawn_wall(&mut app_b, 0.0, 305.0, 400.0, 5.0);
    let bolt_b = spawn_bolt(&mut app_b, 0.0, 298.0, 0.0, 400.0);
    app_b.world_mut().entity_mut(bolt_b).insert(PhantomBolt);

    tick(&mut app_a);
    tick(&mut app_b);

    let msgs_a = app_a.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs_a.0.len(),
        1,
        "app_a: expected 1 BoltImpactWall, got {}",
        msgs_a.0.len()
    );
    assert_eq!(msgs_a.0[0].bolt, bolt_a);
    assert_eq!(msgs_a.0[0].wall, wall_a);

    let msgs_b = app_b.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs_b.0.len(),
        1,
        "app_b: expected 1 BoltImpactWall, got {}",
        msgs_b.0.len()
    );
    assert_eq!(msgs_b.0[0].bolt, bolt_b);
    assert_eq!(msgs_b.0[0].wall, wall_b);

    let vel_a = app_a.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vel_b = app_b.world().get::<Velocity2D>(bolt_b).unwrap().0;
    assert!(
        vel_a.y < 0.0,
        "normal bolt vy should be negative after top wall reflect, got {}",
        vel_a.y
    );
    assert!(
        vel_b.y < 0.0,
        "phantom bolt vy should be negative after top wall reflect, got {}",
        vel_b.y
    );
    assert!(
        (vel_a.x - vel_b.x).abs() < 1e-3,
        "vx must be equal: normal={}, phantom={}",
        vel_a.x,
        vel_b.x
    );
    assert!(
        (vel_a.y - vel_b.y).abs() < 1e-3,
        "vy must be equal: normal={}, phantom={}",
        vel_a.y,
        vel_b.y
    );

    let pos_a = app_a.world().get::<Position2D>(bolt_a).unwrap().0;
    let pos_b = app_b.world().get::<Position2D>(bolt_b).unwrap().0;
    assert!(
        (pos_a.y - 292.0).abs() < 1e-3,
        "normal bolt y should be pushed to 292.0, got {}",
        pos_a.y
    );
    assert!(
        (pos_b.y - 292.0).abs() < 1e-3,
        "phantom bolt y should be pushed to 292.0, got {}",
        pos_b.y
    );
}

// Sub-case D: Pre-existing LastImpact is updated the same way for phantom and normal bolt
// Pins against a future regression where LastImpact stamping is gated on Without<PhantomBolt>.
#[test]
fn phantom_bolt_last_impact_updated_identically_to_normal_bolt_on_left_wall() {
    let mut app_a = test_app();
    spawn_wall(&mut app_a, -5.0, 200.0, 5.0, 400.0);
    let bolt_a = spawn_bolt(&mut app_a, -2.0, 200.0, -400.0, 0.0);
    app_a.world_mut().entity_mut(bolt_a).insert(LastImpact {
        position: Vec2::new(999.0, 999.0),
        side:     ImpactSide::Top,
    });

    let mut app_b = test_app();
    spawn_wall(&mut app_b, -5.0, 200.0, 5.0, 400.0);
    let bolt_b = spawn_bolt(&mut app_b, -2.0, 200.0, -400.0, 0.0);
    app_b.world_mut().entity_mut(bolt_b).insert(PhantomBolt);
    app_b.world_mut().entity_mut(bolt_b).insert(LastImpact {
        position: Vec2::new(999.0, 999.0),
        side:     ImpactSide::Top,
    });

    tick(&mut app_a);
    tick(&mut app_b);

    // Both bolts have LastImpact updated to the real impact values
    let li_a = app_a
        .world()
        .get::<LastImpact>(bolt_a)
        .expect("normal bolt should have LastImpact after wall hit");
    let li_b = app_b
        .world()
        .get::<LastImpact>(bolt_b)
        .expect("phantom bolt should have LastImpact after wall hit");

    // Push-out normal Vec2::X from right face of left wall → ImpactSide::Left
    assert_eq!(
        li_a.side,
        ImpactSide::Left,
        "normal bolt LastImpact.side should be Left, got {:?}",
        li_a.side
    );
    assert_eq!(
        li_b.side,
        ImpactSide::Left,
        "phantom bolt LastImpact.side should be Left, got {:?}",
        li_b.side
    );

    // Position is the push-out position: x=8.0, y=200.0
    assert!(
        (li_a.position.x - 8.0).abs() < 1e-3 && (li_a.position.y - 200.0).abs() < 1e-3,
        "normal bolt LastImpact.position should be (8.0, 200.0), got {:?}",
        li_a.position
    );
    assert!(
        (li_b.position.x - 8.0).abs() < 1e-3 && (li_b.position.y - 200.0).abs() < 1e-3,
        "phantom bolt LastImpact.position should be (8.0, 200.0), got {:?}",
        li_b.position
    );
}
