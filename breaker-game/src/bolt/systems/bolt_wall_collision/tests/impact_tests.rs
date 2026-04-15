use super::helpers::*;
use crate::prelude::*;

// ── Behavior 2: bolt_wall_collision detects wall overlap and reflects velocity ──

#[test]
fn bolt_overlapping_left_wall_emits_impact_and_reflects_velocity() {
    // Spec behavior 2:
    // Given: Bolt at (-2.0, 200.0) with velocity (-400.0, 0.0) and radius 8.0,
    //        left wall at (-5.0, 200.0) with half_extents (5.0, 400.0).
    //        Bolt center is inside the wall's expanded AABB (expanded by bolt radius).
    // When: bolt_wall_collision runs
    // Then: BoltImpactWall emitted, velocity.x becomes positive (reflected off left wall)
    let mut app = test_app();

    // Wall at x=-5 with half_width=5 means wall spans x=[-10, 0].
    // Bolt at x=-2 with radius 8 means expanded AABB spans x=[-18, 8].
    // Bolt center at -2 is inside [-18, 8] => overlap.
    let wall_entity = spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt overlapping wall should emit exactly one BoltImpactWall, got {}",
        msgs.0.len()
    );
    assert_eq!(
        msgs.0[0].bolt, bolt_entity,
        "BoltImpactWall.bolt should match the overlapping bolt entity"
    );
    assert_eq!(
        msgs.0[0].wall, wall_entity,
        "BoltImpactWall.wall should match the overlapped wall entity"
    );

    let vel = app
        .world()
        .get::<Velocity2D>(bolt_entity)
        .expect("bolt should still exist");
    assert!(
        vel.0.x > 0.0,
        "bolt velocity.x should be reflected positive off left wall, got vx={}",
        vel.0.x
    );
}

// ── Behavior 4: bolt_wall_collision is no-op when bolt is not near any wall ──

#[test]
fn bolt_far_from_walls_emits_no_impact_and_preserves_state() {
    // Spec behavior 4:
    // Given: Bolt at (200.0, 200.0) center of playfield, no wall within bolt radius
    // When: bolt_wall_collision runs
    // Then: no BoltImpactWall emitted, position and velocity unchanged
    let mut app = test_app();

    // Walls at playfield edges, far from bolt
    spawn_wall(&mut app, -300.0, 0.0, 5.0, 400.0); // left wall
    spawn_wall(&mut app, 300.0, 0.0, 5.0, 400.0); // right wall
    spawn_wall(&mut app, 0.0, 300.0, 400.0, 5.0); // ceiling

    let bolt_entity = spawn_bolt(&mut app, 200.0, 200.0, -300.0, 100.0);

    let vel_before = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;
    let pos_before = app.world().get::<Position2D>(bolt_entity).unwrap().0;

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "bolt far from walls should emit no BoltImpactWall, got {} messages",
        msgs.0.len()
    );

    let vel_after = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;
    let pos_after = app.world().get::<Position2D>(bolt_entity).unwrap().0;
    assert_eq!(
        vel_before, vel_after,
        "velocity should be unchanged when no wall overlap: before={vel_before}, after={vel_after}"
    );
    assert_eq!(
        pos_before, pos_after,
        "position should be unchanged when no wall overlap: before={pos_before}, after={pos_after}"
    );
}

// ── Behavior 5 (simplified): bolt overlapping ceiling wall after cell bounce ──

#[test]
fn bolt_overlapping_ceiling_wall_reflects_velocity_downward() {
    // Simplified behavior 5: bolt ended up inside ceiling wall
    // (simulating post-cell-bounce overlap). bolt_wall_collision resolves.
    // Given: Bolt at (100.0, 298.0) with velocity (100.0, 300.0), radius 8.0.
    //        Ceiling wall at (0.0, 305.0) with half_extents (400.0, 5.0).
    //        Wall spans y=[300, 310]. Expanded by radius 8: y=[292, 318].
    //        Bolt center at y=298 is inside [292, 318] => overlap.
    // When: bolt_wall_collision runs
    // Then: BoltImpactWall emitted, velocity.y becomes negative (reflected off ceiling)
    let mut app = test_app();

    let wall_entity = spawn_wall(&mut app, 0.0, 305.0, 400.0, 5.0);
    let bolt_entity = spawn_bolt(&mut app, 100.0, 298.0, 100.0, 300.0);

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt overlapping ceiling should emit exactly one BoltImpactWall, got {}",
        msgs.0.len()
    );
    assert_eq!(msgs.0[0].bolt, bolt_entity);
    assert_eq!(msgs.0[0].wall, wall_entity);

    let vel = app
        .world()
        .get::<Velocity2D>(bolt_entity)
        .expect("bolt should still exist");
    assert!(
        vel.0.y < 0.0,
        "bolt velocity.y should be reflected negative off ceiling, got vy={}",
        vel.0.y
    );
}

// ── Edge case: bolt at exact wall boundary (tangent) should not trigger ──

#[test]
fn bolt_tangent_to_wall_does_not_trigger_overlap() {
    // Edge case for behavior 2:
    // Bolt radius = 8.0. Wall half_width = 5.0 at x = -50.0.
    // Wall right edge at x = -45.0. Expanded by bolt radius: x = -37.0.
    // Bolt at x = -37.0 is exactly at the boundary — not inside.
    // No BoltImpactWall should be emitted.
    let mut app = test_app();

    // Wall spans x=[-55, -45]. Expanded by radius 8: x=[-63, -37].
    // Bolt center at x=-37 is exactly on the edge.
    spawn_wall(&mut app, -50.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -37.0, 200.0, -400.0, 0.0);

    let vel_before = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "bolt tangent to wall boundary should not trigger overlap, got {} messages",
        msgs.0.len()
    );

    let vel_after = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;
    assert_eq!(
        vel_before, vel_after,
        "velocity should be unchanged when bolt is tangent to wall"
    );
}
