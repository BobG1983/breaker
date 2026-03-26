//! Applies `Velocity2D` to `Position2D` each fixed timestep.
//! Only affects entities that also carry the [`ApplyVelocity`] marker component.

use bevy::prelude::*;

use crate::components::{ApplyVelocity, Position2D, Velocity2D};

/// Moves `Position2D` by `Velocity2D` * delta time each fixed step.
///
/// Only affects entities that also have the [`ApplyVelocity`] marker component.
pub fn apply_velocity(
    mut query: Query<(&mut Position2D, &Velocity2D), With<ApplyVelocity>>,
    time: Res<Time<Fixed>>,
) {
    for (mut pos, vel) in &mut query {
        pos.0 += vel.0 * time.delta_secs();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Override fixed timestep to 1/64s for predictable math.
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(Duration::from_secs_f64(1.0 / 64.0));
        app.add_systems(FixedUpdate, apply_velocity);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 32: Moves Position2D by Velocity2D * dt ──

    #[test]
    fn applies_velocity_to_position() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
                ApplyVelocity,
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        // dt = 1/64 = 0.015625, displacement = 400 * 0.015625 = 6.25
        assert!(
            (pos.0.x).abs() < f32::EPSILON,
            "x should remain 0.0, got {}",
            pos.0.x
        );
        assert!(
            (pos.0.y - 6.25).abs() < 1e-3,
            "y should be 6.25, got {}",
            pos.0.y
        );
    }

    // ── Behavior 33: No effect without Velocity2D ──

    #[test]
    fn no_effect_without_velocity() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn(Position2D(Vec2::new(10.0, 20.0)))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(
            pos.0,
            Vec2::new(10.0, 20.0),
            "Position2D should be unchanged without Velocity2D"
        );
    }

    // ── Behavior 34: Handles negative velocity ──

    #[test]
    fn handles_negative_velocity() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(100.0, 100.0)),
                Velocity2D(Vec2::new(-200.0, -300.0)),
                ApplyVelocity,
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        // dt = 1/64, x displacement = -200 * (1/64) = -3.125, y = -300 * (1/64) = -4.6875
        // expected: (100 - 3.125, 100 - 4.6875) = (96.875, 95.3125)
        assert!(
            (pos.0.x - 96.875).abs() < 1e-3,
            "x should be 96.875, got {}",
            pos.0.x
        );
        assert!(
            (pos.0.y - 95.3125).abs() < 1e-3,
            "y should be 95.3125, got {}",
            pos.0.y
        );
    }

    // ── Behavior: No effect without ApplyVelocity marker ──

    #[test]
    fn no_effect_without_apply_velocity_marker() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(10.0, 20.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(
            pos.0,
            Vec2::new(10.0, 20.0),
            "Position2D should be unchanged without ApplyVelocity marker"
        );
    }
}
