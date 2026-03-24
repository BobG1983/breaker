//! Safety clamp — catches bolts that escape through wall corner overlaps.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::{components::BoltRadius, filters::ActiveFilter},
    shared::{EntityScale, PlayfieldConfig, math::CCD_EPSILON},
};

/// Clamps bolt position to within the playfield walls and reflects the
/// offending velocity component.
///
/// Runs after all CCD collision systems. Only triggers when a bolt has
/// already escaped past a wall -- a belt-and-suspenders fix for the rare
/// case where CCD misses due to overlapping expanded AABBs at corners.
///
/// The bottom edge is intentionally open -- bolts that fall below the
/// playfield are handled by [`bolt_lost`].
pub(crate) fn clamp_bolt_to_playfield(
    playfield: Res<PlayfieldConfig>,
    mut bolt_query: Query<
        (
            &mut Position2D,
            &mut Velocity2D,
            &BoltRadius,
            Option<&EntityScale>,
        ),
        ActiveFilter,
    >,
) {
    for (mut position, mut vel, radius, bolt_entity_scale) in &mut bolt_query {
        let r = radius.0 * bolt_entity_scale.map_or(1.0, |s| s.0);
        let pos = position.0;

        let x_min = playfield.left() + r + CCD_EPSILON;
        let x_max = playfield.right() - r - CCD_EPSILON;
        let y_max = playfield.top() - r - CCD_EPSILON;

        let mut new_pos = pos;
        let mut new_vel = vel.0;
        let mut clamped = false;

        if pos.x < x_min {
            new_pos.x = x_min;
            if new_vel.x < 0.0 {
                new_vel.x = -new_vel.x;
            }
            clamped = true;
        } else if pos.x > x_max {
            new_pos.x = x_max;
            if new_vel.x > 0.0 {
                new_vel.x = -new_vel.x;
            }
            clamped = true;
        }

        if pos.y > y_max {
            new_pos.y = y_max;
            if new_vel.y > 0.0 {
                new_vel.y = -new_vel.y;
            }
            clamped = true;
        }
        // No bottom clamp — intentionally open for bolt-lost

        if clamped {
            position.0 = new_pos;
            vel.0 = new_vel;
        }
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{
        bolt::components::{Bolt, BoltServing},
        shared::math::CCD_EPSILON,
    };

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

    #[test]
    fn bolt_inside_bounds_position2d_unchanged() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, 400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(100.0, 50.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, 400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(500.0, 0.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(-300.0, 400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(-500.0, 0.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, 400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(0.0, 400.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, -400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(0.0, -500.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(-300.0, 400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(500.0, 0.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, -400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(0.0, 400.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, 400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(500.0, 400.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            BoltServing,
            Velocity2D(Vec2::new(300.0, 400.0)),
            BoltRadius(RADIUS),
            Position2D(Vec2::new(500.0, 0.0)),
        ));
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

    // --- EntityScale clamping tests ---

    #[test]
    fn scaled_bolt_uses_effective_radius_for_playfield_clamping() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, 400.0)),
            BoltRadius(8.0),
            EntityScale(0.5),
            Position2D(Vec2::new(500.0, 0.0)),
        ));
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
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, 400.0)),
            BoltRadius(RADIUS),
            // No EntityScale
            Position2D(Vec2::new(500.0, 0.0)),
        ));
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
            "bolt without EntityScale should clamp to {expected_x:.2}, got {:.2}",
            pos.0.x
        );
        assert!(
            (vel.0.x - (-300.0)).abs() < TOLERANCE,
            "vx should be flipped to -300, got {:.1}",
            vel.0.x
        );
    }
}
