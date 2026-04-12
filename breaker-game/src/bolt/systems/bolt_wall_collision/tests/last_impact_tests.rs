use bevy::prelude::*;

use super::helpers::*;
use crate::bolt::components::{ImpactSide, LastImpact};

// ── Behavior 7: left wall rebound stamps LastImpact with ImpactSide::Left ──

#[test]
fn left_wall_rebound_stamps_last_impact_with_left_side() {
    // Given: Bolt at (-2.0, 200.0) with velocity (-400.0, 0.0) and radius 8.0,
    //        left wall at (-5.0, 200.0) with half_extents (5.0, 400.0).
    //        Bolt center is inside the wall's expanded AABB.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Left
    let mut app = test_app();

    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after left wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Left,
        "left wall rebound should stamp ImpactSide::Left, got {:?}",
        last_impact.side
    );
}

#[test]
fn left_wall_tangent_bolt_does_not_stamp_last_impact() {
    // Edge case: Bolt tangent to wall boundary (exactly on expanded edge).
    // No collision, no LastImpact.
    let mut app = test_app();

    // Wall spans x=[-10, 0]. Expanded by radius 8: x=[-18, 8].
    // Bolt center at x=8 is exactly on the edge — not inside.
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, 8.0, 200.0, -400.0, 0.0);

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "bolt tangent to wall should NOT get LastImpact, got {last_impact:?}"
    );
}

// ── Behavior 8: right wall rebound stamps LastImpact with ImpactSide::Right ──

#[test]
fn right_wall_rebound_stamps_last_impact_with_right_side() {
    // Given: Bolt at (2.0, 200.0) with velocity (400.0, 0.0) and radius 8.0,
    //        right wall at (5.0, 200.0) with half_extents (5.0, 400.0).
    //        Bolt center is inside the wall's expanded AABB.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Right
    let mut app = test_app();

    spawn_wall(&mut app, 5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, 2.0, 200.0, 400.0, 0.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after right wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Right,
        "right wall rebound should stamp ImpactSide::Right, got {:?}",
        last_impact.side
    );
}

// ── Behavior 9: ceiling wall rebound stamps LastImpact with ImpactSide::Top ──

#[test]
fn ceiling_wall_rebound_stamps_last_impact_with_top_side() {
    // Given: Bolt at (100.0, 298.0) with velocity (100.0, 300.0) and radius 8.0,
    //        ceiling wall at (0.0, 305.0) with half_extents (400.0, 5.0).
    //        Wall spans y=[300, 310]. Expanded by radius 8: y=[292, 318].
    //        Bolt at y=298 is inside.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Top
    let mut app = test_app();

    spawn_wall(&mut app, 0.0, 305.0, 400.0, 5.0);
    let bolt_entity = spawn_bolt(&mut app, 100.0, 298.0, 100.0, 300.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after ceiling wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "ceiling wall rebound should stamp ImpactSide::Top, got {:?}",
        last_impact.side
    );
}

#[test]
fn ceiling_wall_diagonal_bolt_side_determined_by_nearest_face() {
    // Edge case: Bolt moving diagonally (100.0, 300.0) — side is determined by
    // nearest face, not velocity direction.
    let mut app = test_app();

    spawn_wall(&mut app, 0.0, 305.0, 400.0, 5.0);
    let bolt_entity = spawn_bolt(&mut app, 100.0, 298.0, 100.0, 300.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after ceiling wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "ceiling wall side should be ImpactSide::Top regardless of diagonal velocity, got {:?}",
        last_impact.side
    );
}

// ── Behavior 13: each new rebound overwrites the previous LastImpact ──

#[test]
fn wall_rebound_overwrites_previous_last_impact() {
    // Given: Bolt with pre-existing LastImpact { position: (50.0, 300.0), side: Top }.
    //        Bolt at (-2.0, 200.0) with velocity (-400.0, 0.0), overlapping left wall.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: LastImpact is overwritten with new wall collision data.
    //       side is ImpactSide::Left, old values are gone.
    let mut app = test_app();

    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);

    // Insert a pre-existing LastImpact
    app.world_mut().entity_mut(bolt_entity).insert(LastImpact {
        position: Vec2::new(50.0, 300.0),
        side:     ImpactSide::Top,
    });

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Left,
        "wall rebound should overwrite side from Top to Left, got {:?}",
        last_impact.side
    );
    assert!(
        (last_impact.position.y - 200.0).abs() < 10.0,
        "wall rebound should overwrite position from (50, 300) — new position.y should be near 200.0, got {:?}",
        last_impact.position
    );
    assert!(
        (last_impact.position.x - 50.0).abs() > 0.01,
        "wall rebound should overwrite position.x from old value 50.0, got {}",
        last_impact.position.x
    );
}

// ── Behavior 14: bolt with no collision does not gain LastImpact ──

#[test]
fn bolt_with_no_collision_does_not_gain_last_impact() {
    // Given: Bolt at (200.0, 200.0) with velocity (-300.0, 100.0), no walls nearby.
    //        No pre-existing LastImpact on the bolt.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt entity does NOT have a LastImpact component
    let mut app = test_app();

    // Walls far away
    spawn_wall(&mut app, -300.0, 0.0, 5.0, 400.0);
    spawn_wall(&mut app, 300.0, 0.0, 5.0, 400.0);

    let bolt_entity = spawn_bolt(&mut app, 200.0, 200.0, -300.0, 100.0);

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "bolt with no wall collision should NOT get LastImpact, got {last_impact:?}"
    );
}

#[test]
fn bolt_with_zero_velocity_does_not_gain_last_impact() {
    // Edge case: Bolt with zero velocity — no collision possible, no LastImpact.
    let mut app = test_app();

    // Wall nearby but bolt has zero velocity
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, 200.0, 200.0, 0.0, 0.0);

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "bolt with zero velocity should NOT get LastImpact, got {last_impact:?}"
    );
}
