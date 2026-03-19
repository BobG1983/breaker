//! Safety clamp — catches bolts that escape through wall corner overlaps.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{BoltRadius, BoltVelocity},
        filters::ActiveFilter,
    },
    shared::{PlayfieldConfig, math::CCD_EPSILON},
};

/// Clamps bolt position to within the playfield walls and reflects the
/// offending velocity component.
///
/// Runs after all CCD collision systems. Only triggers when a bolt has
/// already escaped past a wall — a belt-and-suspenders fix for the rare
/// case where CCD misses due to overlapping expanded AABBs at corners.
///
/// The bottom edge is intentionally open — bolts that fall below the
/// playfield are handled by [`bolt_lost`].
pub(crate) fn clamp_bolt_to_playfield(
    playfield: Res<PlayfieldConfig>,
    mut bolt_query: Query<(&mut Transform, &mut BoltVelocity, &BoltRadius), ActiveFilter>,
) {
    for (mut tf, mut vel, radius) in &mut bolt_query {
        let r = radius.0;
        // Read immutably first to avoid triggering Bevy change detection
        // when no clamping is needed (the common case).
        let pos = tf.translation;

        let x_min = playfield.left() + r + CCD_EPSILON;
        let x_max = playfield.right() - r - CCD_EPSILON;
        let y_max = playfield.top() - r - CCD_EPSILON;

        let mut new_pos = pos;
        let mut new_vel = vel.value;
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
            tf.translation = new_pos;
            vel.value = new_vel;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltServing};

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

    /// Default playfield: width=800, height=600 → left=-400, right=400, top=300, bottom=-300
    const RADIUS: f32 = 6.0;
    const TOLERANCE: f32 = 0.001;

    #[test]
    fn bolt_inside_bounds_unchanged() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(300.0, 400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(100.0, 50.0, 0.0),
        ));
        tick(&mut app);

        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!((tf.translation.x - 100.0).abs() < TOLERANCE);
        assert!((tf.translation.y - 50.0).abs() < TOLERANCE);
        assert!((vel.value.x - 300.0).abs() < TOLERANCE);
        assert!((vel.value.y - 400.0).abs() < TOLERANCE);
    }

    #[test]
    fn bolt_past_right_wall_clamped_vx_flipped() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(300.0, 400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(500.0, 0.0, 0.0),
        ));
        tick(&mut app);

        let expected_x = 400.0 - RADIUS - CCD_EPSILON; // 393.99
        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.x - expected_x).abs() < TOLERANCE,
            "x should be clamped to {expected_x}, got {}",
            tf.translation.x
        );
        assert!(
            (vel.value.x - (-300.0)).abs() < TOLERANCE,
            "vx should be flipped to -300, got {}",
            vel.value.x
        );
        assert!(
            (vel.value.y - 400.0).abs() < TOLERANCE,
            "vy should be unchanged"
        );
    }

    #[test]
    fn bolt_past_left_wall_clamped_vx_flipped() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(-300.0, 400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(-500.0, 0.0, 0.0),
        ));
        tick(&mut app);

        let expected_x = -400.0 + RADIUS + CCD_EPSILON; // -393.99
        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.x - expected_x).abs() < TOLERANCE,
            "x should be clamped to {expected_x}, got {}",
            tf.translation.x
        );
        assert!(
            (vel.value.x - 300.0).abs() < TOLERANCE,
            "vx should be flipped to 300, got {}",
            vel.value.x
        );
        assert!(
            (vel.value.y - 400.0).abs() < TOLERANCE,
            "vy should be unchanged"
        );
    }

    #[test]
    fn bolt_past_ceiling_clamped_vy_flipped() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(300.0, 400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(0.0, 400.0, 0.0),
        ));
        tick(&mut app);

        let expected_y = 300.0 - RADIUS - CCD_EPSILON; // 293.99
        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.y - expected_y).abs() < TOLERANCE,
            "y should be clamped to {expected_y}, got {}",
            tf.translation.y
        );
        assert!(
            (vel.value.y - (-400.0)).abs() < TOLERANCE,
            "vy should be flipped to -400, got {}",
            vel.value.y
        );
        assert!(
            (vel.value.x - 300.0).abs() < TOLERANCE,
            "vx should be unchanged"
        );
    }

    #[test]
    fn bolt_below_floor_not_clamped() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(300.0, -400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(0.0, -500.0, 0.0),
        ));
        tick(&mut app);

        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.y - (-500.0)).abs() < TOLERANCE,
            "y should NOT be clamped, got {}",
            tf.translation.y
        );
        assert!(
            (vel.value.y - (-400.0)).abs() < TOLERANCE,
            "vy should NOT be flipped, got {}",
            vel.value.y
        );
    }

    #[test]
    fn velocity_already_inward_not_flipped_right_wall() {
        let mut app = test_app();
        // Bolt past right wall but velocity already pointing left
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(-300.0, 400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(500.0, 0.0, 0.0),
        ));
        tick(&mut app);

        let expected_x = 400.0 - RADIUS - CCD_EPSILON;
        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.x - expected_x).abs() < TOLERANCE,
            "x should be clamped"
        );
        assert!(
            (vel.value.x - (-300.0)).abs() < TOLERANCE,
            "vx already pointing inward should NOT be flipped, got {}",
            vel.value.x
        );
    }

    #[test]
    fn velocity_already_inward_not_flipped_ceiling() {
        let mut app = test_app();
        // Bolt past ceiling but velocity already pointing down
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(300.0, -400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(0.0, 400.0, 0.0),
        ));
        tick(&mut app);

        let expected_y = 300.0 - RADIUS - CCD_EPSILON;
        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.y - expected_y).abs() < TOLERANCE,
            "y should be clamped"
        );
        assert!(
            (vel.value.y - (-400.0)).abs() < TOLERANCE,
            "vy already pointing inward should NOT be flipped, got {}",
            vel.value.y
        );
    }

    #[test]
    fn corner_escape_both_axes_clamped() {
        let mut app = test_app();
        // Bolt past both right wall and ceiling simultaneously
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(300.0, 400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(500.0, 400.0, 0.0),
        ));
        tick(&mut app);

        let expected_x = 400.0 - RADIUS - CCD_EPSILON;
        let expected_y = 300.0 - RADIUS - CCD_EPSILON;
        let (tf, vel) = app
            .world_mut()
            .query::<(&Transform, &BoltVelocity)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.x - expected_x).abs() < TOLERANCE,
            "x should be clamped to {expected_x}, got {}",
            tf.translation.x
        );
        assert!(
            (tf.translation.y - expected_y).abs() < TOLERANCE,
            "y should be clamped to {expected_y}, got {}",
            tf.translation.y
        );
        assert!(
            (vel.value.x - (-300.0)).abs() < TOLERANCE,
            "vx should be flipped to -300, got {}",
            vel.value.x
        );
        assert!(
            (vel.value.y - (-400.0)).abs() < TOLERANCE,
            "vy should be flipped to -400, got {}",
            vel.value.y
        );
    }

    #[test]
    fn serving_bolt_excluded() {
        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltServing,
            BoltVelocity::new(300.0, 400.0),
            BoltRadius(RADIUS),
            Transform::from_xyz(500.0, 0.0, 0.0),
        ));
        tick(&mut app);

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, (With<Bolt>, With<BoltServing>)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.x - 500.0).abs() < TOLERANCE,
            "serving bolt should NOT be clamped, got {}",
            tf.translation.x
        );
    }
}
