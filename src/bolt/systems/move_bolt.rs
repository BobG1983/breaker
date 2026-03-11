//! System to move the bolt by its velocity each fixed tick.

use bevy::prelude::*;

use crate::bolt::components::{Bolt, BoltServing, BoltVelocity};
use crate::bolt::resources::BoltConfig;

type MoveBoltFilter = (With<Bolt>, Without<BoltServing>);

/// Moves the bolt by its velocity each fixed timestep.
///
/// Enforces speed clamping and minimum angle from horizontal.
pub fn move_bolt(
    config: Res<BoltConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &mut BoltVelocity), MoveBoltFilter>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut velocity) in &mut query {
        // Enforce speed bounds
        let speed = velocity.speed();
        if speed > f32::EPSILON {
            let clamped_speed = speed.clamp(config.min_speed, config.max_speed);
            if (clamped_speed - speed).abs() > f32::EPSILON {
                velocity.value = velocity.direction() * clamped_speed;
            }

            // Enforce minimum angle from horizontal
            velocity.enforce_min_angle(config.min_angle_from_horizontal);
        }

        // Apply velocity to position
        transform.translation.x = velocity.value.x.mul_add(dt, transform.translation.x);
        transform.translation.y = velocity.value.y.mul_add(dt, transform.translation.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.add_systems(FixedUpdate, move_bolt);
        // Prime time baseline
        app.update();
        app
    }

    /// Runs enough updates with sleeps to ensure `FixedUpdate` ticks at least once.
    fn tick_fixed(app: &mut App) {
        // FixedUpdate needs wall-clock time to accumulate past the timestep (~16ms)
        std::thread::sleep(std::time::Duration::from_millis(20));
        app.update();
    }

    #[test]
    fn move_bolt_translates_position() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            Transform::from_xyz(0.0, 0.0, 0.0),
            BoltVelocity::new(0.0, 400.0),
        ));

        tick_fixed(&mut app);

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");

        assert!(
            tf.translation.y > 0.0,
            "bolt should move upward, got y={}",
            tf.translation.y
        );
    }

    #[test]
    fn serving_bolt_is_not_moved() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                Transform::from_xyz(0.0, 0.0, 0.0),
                BoltVelocity::new(0.0, 400.0),
            ))
            .id();

        tick_fixed(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert!(
            tf.translation.y.abs() < f32::EPSILON,
            "serving bolt should not move, got y={}",
            tf.translation.y
        );
    }

    #[test]
    fn speed_below_min_is_clamped_up() {
        let mut app = test_app();
        let config = app.world().resource::<BoltConfig>().clone();

        app.world_mut().spawn((
            Bolt,
            Transform::from_xyz(0.0, 0.0, 0.0),
            BoltVelocity::new(0.0, 1.0), // far below min_speed
        ));

        tick_fixed(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(
            vel.speed() >= config.min_speed - f32::EPSILON,
            "speed {} should be at least min_speed {}",
            vel.speed(),
            config.min_speed
        );
    }
}
