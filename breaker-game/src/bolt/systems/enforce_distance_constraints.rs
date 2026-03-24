//! Distance constraint solver for tethered chain bolts.

use bevy::prelude::*;
use rantzsoft_physics2d::constraint::DistanceConstraint;
use rantzsoft_spatial2d::components::Position2D;

use crate::bolt::components::BoltVelocity;

/// Enforces distance constraints between tethered bolt pairs.
///
/// When the distance between two bolts exceeds the constraint's `max_distance`,
/// both bolts are pulled back symmetrically along the chain axis, and their
/// velocity components along the axis are redistributed (momentum conservation)
/// while perpendicular velocity is preserved.
///
/// Runs after `clamp_bolt_to_playfield`, before `bolt_lost`.
pub(crate) fn enforce_distance_constraints(
    constraint_query: Query<&DistanceConstraint>,
    mut bolt_query: Query<(&mut Position2D, &mut BoltVelocity)>,
) {
    for constraint in &constraint_query {
        // Look up both entities; skip if either is missing
        let Ok([mut a, mut b]) =
            bolt_query.get_many_mut([constraint.entity_a, constraint.entity_b])
        else {
            continue;
        };

        let delta = b.0 .0 - a.0 .0;
        let distance = delta.length();

        // Skip if same position (can't normalize zero vector) or within slack
        if distance < f32::EPSILON || distance <= constraint.max_distance {
            continue;
        }

        // Taut — apply position correction
        let axis = delta / distance;
        let half_correction = (distance - constraint.max_distance) / 2.0;
        a.0 .0 += axis * half_correction;
        b.0 .0 -= axis * half_correction;

        // Velocity redistribution — only when NOT both actively converging.
        // "Both converging" = A moving toward B (positive along axis) AND
        // B moving toward A (negative along axis). In that case both bolts
        // will naturally close the gap and no velocity adjustment is needed.
        let vel_a_along = a.1.value.dot(axis);
        let vel_b_along = b.1.value.dot(axis);
        let both_converging = vel_a_along > 0.0 && vel_b_along < 0.0;

        if !both_converging {
            let avg = (vel_a_along + vel_b_along) / 2.0;
            a.1.value += (avg - vel_a_along) * axis;
            b.1.value += (avg - vel_b_along) * axis;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use rantzsoft_physics2d::constraint::DistanceConstraint;
    use rantzsoft_spatial2d::components::Position2D;

    use crate::bolt::components::{Bolt, BoltVelocity};

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

    // ── Behavior 1: Slack constraint — positions/velocities unchanged ──

    #[test]
    fn slack_constraint_leaves_positions_unchanged() {
        // Given: A at (0,0), B at (100,0), max_distance=200
        // When: enforce_distance_constraints runs
        // Then: positions and velocities remain unchanged
        let mut app = test_app();

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(100.0, 200.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, 0.0)),
                BoltVelocity::new(-50.0, 300.0),
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
        let vel_a = app.world().get::<BoltVelocity>(a).unwrap();
        let vel_b = app.world().get::<BoltVelocity>(b).unwrap();

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
            (vel_a.value - Vec2::new(100.0, 200.0)).length() < f32::EPSILON,
            "A velocity should be unchanged when slack, got {:?}",
            vel_a.value,
        );
        assert!(
            (vel_b.value - Vec2::new(-50.0, 300.0)).length() < f32::EPSILON,
            "B velocity should be unchanged when slack, got {:?}",
            vel_b.value,
        );
    }

    // ── Behavior 2: Taut constraint — position correction ──

    #[test]
    fn taut_constraint_corrects_positions_symmetrically() {
        // Given: A at (0,0), B at (300,0), max_distance=200
        // Distance = 300, overshoot = 100
        // When: enforce_distance_constraints runs
        // Then: A=(50,0), B=(250,0) — each moved 50 units inward
        let mut app = test_app();

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(0.0, 0.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(300.0, 0.0)),
                BoltVelocity::new(0.0, 0.0),
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

    // ── Behavior 3: Taut — velocity redistribution along chain axis ──

    #[test]
    fn taut_constraint_redistributes_velocity_along_axis() {
        // Given: A at (0,0), B at (300,0), max_distance=200
        // A.vel = (400,0), B.vel = (0,0)
        // Chain axis = (1,0) — purely horizontal
        // When: enforce_distance_constraints runs
        // Then: average axial velocity = 200 for each (momentum conservation)
        let mut app = test_app();

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(400.0, 0.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(300.0, 0.0)),
                BoltVelocity::new(0.0, 0.0),
            ))
            .id();
        app.world_mut().spawn(DistanceConstraint {
            entity_a: a,
            entity_b: b,
            max_distance: 200.0,
        });

        tick(&mut app);

        let vel_a = app.world().get::<BoltVelocity>(a).unwrap();
        let vel_b = app.world().get::<BoltVelocity>(b).unwrap();

        // Momentum along axis should be conserved: avg = (400+0)/2 = 200 each
        assert!(
            (vel_a.value.x - 200.0).abs() < 1.0,
            "A.vx should be ~200 (redistributed), got {:.1}",
            vel_a.value.x,
        );
        assert!(
            (vel_b.value.x - 200.0).abs() < 1.0,
            "B.vx should be ~200 (redistributed), got {:.1}",
            vel_b.value.x,
        );
    }

    // ── Behavior 4: Taut — perpendicular velocity preserved ──

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
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(400.0, 100.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(300.0, 0.0)),
                BoltVelocity::new(0.0, 300.0),
            ))
            .id();
        app.world_mut().spawn(DistanceConstraint {
            entity_a: a,
            entity_b: b,
            max_distance: 200.0,
        });

        tick(&mut app);

        let vel_a = app.world().get::<BoltVelocity>(a).unwrap();
        let vel_b = app.world().get::<BoltVelocity>(b).unwrap();

        assert!(
            (vel_a.value.y - 100.0).abs() < 0.01,
            "A.vy should be preserved at 100, got {:.2}",
            vel_a.value.y,
        );
        assert!(
            (vel_b.value.y - 300.0).abs() < 0.01,
            "B.vy should be preserved at 300, got {:.2}",
            vel_b.value.y,
        );
    }

    // ── Behavior 5: Only redistributes when separating along axis ──

    #[test]
    fn taut_constraint_no_velocity_redistribution_when_converging() {
        // Given: A at (0,0), B at (300,0), max_distance=200
        // A.vel = (400,0), B.vel = (-100,0) — converging (both moving toward center)
        // Relative axial velocity = 400 - (-100) = 500 (positive = separating? No!)
        // Actually the spec says: only redistribute if bolts are separating.
        // A moving right (+400) and B moving left (-100) means they are
        // converging (A moves toward B, B moves toward A), so NO redistribution.
        // Wait: A at 0 moving right toward B at 300, B at 300 moving left toward A.
        // That's converging. No velocity redistribution.
        let mut app = test_app();

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(400.0, 0.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(300.0, 0.0)),
                BoltVelocity::new(-100.0, 0.0),
            ))
            .id();
        app.world_mut().spawn(DistanceConstraint {
            entity_a: a,
            entity_b: b,
            max_distance: 200.0,
        });

        tick(&mut app);

        let vel_a = app.world().get::<BoltVelocity>(a).unwrap();
        let vel_b = app.world().get::<BoltVelocity>(b).unwrap();

        // Positions should still be corrected, but velocities should NOT be redistributed
        // because the bolts are converging along the axis.
        assert!(
            (vel_a.value.x - 400.0).abs() < 0.01,
            "A.vx should be unchanged (converging), got {:.2}",
            vel_a.value.x,
        );
        assert!(
            (vel_b.value.x - (-100.0)).abs() < 0.01,
            "B.vx should be unchanged (converging), got {:.2}",
            vel_b.value.x,
        );
    }

    // ── Behavior 6: Diagonal axis works ──

    #[test]
    fn taut_constraint_works_on_diagonal_axis() {
        // Given: A at (0,0), B at (300,300), max_distance=200
        // Distance = ~424.26, axis = (1/sqrt2, 1/sqrt2)
        // Overshoot = 424.26 - 200 = 224.26, half = 112.13
        // Each moves 112.13 along the diagonal axis
        // A moves toward B: (0,0) + 112.13 * (0.707,0.707) ~ (79.3, 79.3)
        // B moves toward A: (300,300) - 112.13 * (0.707,0.707) ~ (220.7, 220.7)
        let mut app = test_app();

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(0.0, 0.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(300.0, 300.0)),
                BoltVelocity::new(0.0, 0.0),
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

        let distance = (pos_b.0 - pos_a.0).length();
        assert!(
            (distance - 200.0).abs() < 1.0,
            "distance after correction should be ~200, got {distance:.1}",
        );

        // Both should have moved symmetrically
        let a_moved = pos_a.0.length();
        let b_moved = (pos_b.0 - Vec2::new(300.0, 300.0)).length();
        assert!(
            (a_moved - b_moved).abs() < 1.0,
            "both bolts should move equal distances: A={a_moved:.1}, B={b_moved:.1}",
        );
    }

    // ── Behavior 7: Missing entity — skip gracefully ──

    #[test]
    fn missing_entity_skipped_gracefully() {
        // Given: constraint referencing a despawned entity
        // When: enforce_distance_constraints runs
        // Then: no panic, surviving bolt unchanged
        let mut app = test_app();

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(100.0, 200.0),
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
            "surviving bolt should be unaffected when partner is missing",
        );
    }

    // ── Behavior 8: Zero distance (same position) — skip correction ──

    #[test]
    fn zero_distance_same_position_skips_correction() {
        // Given: A and B at the same position (0,0), max_distance=200
        // When: enforce_distance_constraints runs
        // Then: no correction applied, no division by zero
        let mut app = test_app();

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(100.0, 200.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(-50.0, 300.0),
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
        let vel_a = app.world().get::<BoltVelocity>(a).unwrap();
        let vel_b = app.world().get::<BoltVelocity>(b).unwrap();

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
            vel_a.value.x.is_finite() && vel_a.value.y.is_finite(),
            "A velocity should be finite, got {:?}",
            vel_a.value,
        );
        assert!(
            vel_b.value.x.is_finite() && vel_b.value.y.is_finite(),
            "B velocity should be finite, got {:?}",
            vel_b.value,
        );
    }
}
