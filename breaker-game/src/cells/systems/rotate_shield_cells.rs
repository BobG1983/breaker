//! System to rotate orbit cells around their shield parent.

use bevy::prelude::*;

use crate::cells::components::{OrbitAngle, OrbitCell, OrbitConfig};

/// Increments the [`OrbitAngle`] of each orbit cell by `speed * dt`.
///
/// Runs each fixed timestep. Angle wraps naturally via `f32` arithmetic
/// (no explicit modulo needed since `cos`/`sin` handle large angles).
pub(crate) fn rotate_shield_cells(
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut OrbitAngle, &OrbitConfig), With<OrbitCell>>,
) {
    let dt = time.delta_secs();
    for (mut angle, config) in &mut query {
        angle.0 = config.speed.mul_add(dt, angle.0);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        f32::consts::{FRAC_PI_2, PI},
        time::Duration,
    };

    use super::*;
    use crate::cells::components::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, rotate_shield_cells);
        app
    }

    fn tick_with_dt(app: &mut App, dt: Duration) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(dt);
        app.update();
    }

    fn spawn_orbit_cell(app: &mut App, angle: f32, radius: f32, speed: f32) -> Entity {
        app.world_mut()
            .spawn((OrbitCell, OrbitAngle(angle), OrbitConfig { radius, speed }))
            .id()
    }

    // ── Behavior 6: rotate_shield_cells increments OrbitAngle by speed * dt ──

    #[test]
    fn orbit_angle_incremented_by_speed_times_dt() {
        // Given: orbit cell at angle 0.0, speed PI/2 rad/s
        // When: tick with dt = 1.0s
        // Then: angle = 0.0 + (PI/2) * 1.0 = PI/2
        let mut app = test_app();
        let entity = spawn_orbit_cell(&mut app, 0.0, 60.0, FRAC_PI_2);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let angle = app.world().get::<OrbitAngle>(entity).unwrap();
        assert!(
            (angle.0 - FRAC_PI_2).abs() < 1e-5,
            "orbit angle should be PI/2 ({FRAC_PI_2}) after 1s at speed PI/2, got {}",
            angle.0
        );
    }

    #[test]
    fn orbit_angle_accumulates_over_multiple_ticks() {
        // Given: orbit cell at angle 0.0, speed PI/2 rad/s
        // When: 2 ticks at dt = 1.0s each
        // Then: angle = PI/2 * 2 = PI
        let mut app = test_app();
        let entity = spawn_orbit_cell(&mut app, 0.0, 60.0, FRAC_PI_2);

        tick_with_dt(&mut app, Duration::from_secs(1));
        tick_with_dt(&mut app, Duration::from_secs(1));

        let angle = app.world().get::<OrbitAngle>(entity).unwrap();
        assert!(
            (angle.0 - PI).abs() < 1e-4,
            "orbit angle should be PI ({PI}) after 2s at speed PI/2, got {}",
            angle.0
        );
    }

    #[test]
    fn orbit_angle_unchanged_at_zero_speed() {
        // Given: orbit cell at angle 1.0, speed 0.0
        // When: tick with dt = 1.0s
        // Then: angle remains 1.0
        let mut app = test_app();
        let entity = spawn_orbit_cell(&mut app, 1.0, 60.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let angle = app.world().get::<OrbitAngle>(entity).unwrap();
        assert!(
            (angle.0 - 1.0).abs() < f32::EPSILON,
            "orbit angle should remain 1.0 with speed 0.0, got {}",
            angle.0
        );
    }

    #[test]
    fn orbit_angle_preserves_initial_offset() {
        // Given: orbit cell starts at angle 2*PI/3, speed PI/2
        // When: tick with dt = 1.0s
        // Then: angle = 2*PI/3 + PI/2
        let initial = 2.0 * PI / 3.0;
        let expected = initial + FRAC_PI_2;
        let mut app = test_app();
        let entity = spawn_orbit_cell(&mut app, initial, 60.0, FRAC_PI_2);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let angle = app.world().get::<OrbitAngle>(entity).unwrap();
        assert!(
            (angle.0 - expected).abs() < 1e-5,
            "orbit angle should be {expected} (initial {initial} + PI/2), got {}",
            angle.0
        );
    }

    #[test]
    fn non_orbit_entity_not_affected() {
        // Given: an entity with OrbitAngle and OrbitConfig but WITHOUT OrbitCell marker
        // When: tick
        // Then: should not panic or crash (entity not matched by query)
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                OrbitAngle(0.0),
                OrbitConfig {
                    radius: 60.0,
                    speed: FRAC_PI_2,
                },
            ))
            .id();

        tick_with_dt(&mut app, Duration::from_secs(1));

        // Entity without OrbitCell marker is not processed — angle unchanged.
        let angle = app.world().get::<OrbitAngle>(entity).unwrap();
        assert!(
            (angle.0 - 0.0).abs() < f32::EPSILON,
            "entity without OrbitCell marker should not be rotated, got {}",
            angle.0
        );
    }
}
