//! Gravity well systems — tick force application, despawn expired.

use bevy::prelude::*;

use super::components::*;
use crate::prelude::*;

/// Applies gravitational pull to bolts within each well's radius.
pub fn tick_gravity_well(
    well_query: Query<
        (&Position2D, &GravityWellStrength, &GravityWellRadius),
        With<GravityWellSource>,
    >,
    mut bolt_query: Query<(&mut Velocity2D, &Position2D), With<Bolt>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (well_pos, strength, radius) in &well_query {
        for (mut velocity, bolt_pos) in &mut bolt_query {
            let to_well = well_pos.0 - bolt_pos.0;
            let dist = to_well.length();
            if dist > f32::EPSILON && dist <= radius.0 {
                let original_speed = velocity.0.length();
                if original_speed > f32::EPSILON {
                    let direction = to_well / dist;
                    velocity.0 += direction * strength.0 * dt;
                    velocity.0 = velocity.0.normalize_or(Vec2::ZERO) * original_speed;
                }
            }
        }
    }
}

/// Despawns gravity wells whose lifetime has expired.
pub fn despawn_expired_wells(
    mut query: Query<(Entity, &mut GravityWellLifetime), With<GravityWellSource>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;

    fn gravity_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, tick_gravity_well)
            .build()
    }

    fn spawn_well(app: &mut App, pos: Vec2, strength: f32, radius: f32) -> Entity {
        app.world_mut()
            .spawn((
                GravityWellSource,
                GravityWellStrength(strength),
                GravityWellRadius(radius),
                Position2D(pos),
            ))
            .id()
    }

    fn spawn_bolt(app: &mut App, pos: Vec2, vel: Vec2) -> Entity {
        app.world_mut()
            .spawn((Bolt, Position2D(pos), Velocity2D(vel)))
            .id()
    }

    // ── C6: GravityWell speed preservation ─────────────────────────────────

    #[test]
    fn gravity_well_bends_bolt_direction_without_changing_speed() {
        let mut app = gravity_test_app();

        spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 200.0);
        let bolt = spawn_bolt(&mut app, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0));

        let original_speed = 400.0_f32;

        tick(&mut app);

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        let new_speed = velocity.0.length();

        // Direction should have changed (bolt pulled toward well at origin).
        // The x-component should now be negative (pulled left toward well).
        assert!(
            velocity.0.x < 0.0,
            "bolt x-velocity should be negative (pulled toward well), got {}",
            velocity.0.x,
        );

        // Speed magnitude should be preserved.
        assert!(
            (new_speed - original_speed).abs() < 0.01,
            "bolt speed should remain {original_speed}, got {new_speed}. \
             Gravity well should bend direction without changing speed magnitude.",
        );
    }

    #[test]
    fn gravity_well_preserves_speed_for_bolt_moving_toward_well() {
        let mut app = gravity_test_app();

        spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 200.0);
        let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(-300.0, 0.0));

        let original_speed = 300.0_f32;

        tick(&mut app);

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        let new_speed = velocity.0.length();

        // Direction should still be primarily toward the well (negative x).
        assert!(
            velocity.0.x < 0.0,
            "bolt should still be moving toward well (negative x), got {}",
            velocity.0.x,
        );

        // Speed magnitude should be preserved.
        assert!(
            (new_speed - original_speed).abs() < 0.01,
            "bolt speed should remain {original_speed}, got {new_speed}. \
             Gravity well should preserve speed magnitude.",
        );
    }

    #[test]
    fn gravity_well_bolt_outside_radius_is_unaffected() {
        let mut app = gravity_test_app();

        spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 200.0);
        let bolt = spawn_bolt(
            &mut app,
            Vec2::new(300.0, 0.0), // outside radius of 200
            Vec2::new(0.0, 400.0),
        );

        tick(&mut app);

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (velocity.0.x).abs() < f32::EPSILON,
            "bolt outside radius should have unchanged x-velocity, got {}",
            velocity.0.x,
        );
        assert!(
            (velocity.0.y - 400.0).abs() < f32::EPSILON,
            "bolt outside radius should have unchanged y-velocity, got {}",
            velocity.0.y,
        );
    }

    #[test]
    fn gravity_well_zero_velocity_bolt_does_not_produce_nan() {
        let mut app = gravity_test_app();

        spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 200.0);
        let bolt = spawn_bolt(&mut app, Vec2::new(50.0, 0.0), Vec2::ZERO);

        tick(&mut app);

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            !velocity.0.x.is_nan() && !velocity.0.y.is_nan(),
            "bolt velocity should not contain NaN, got {:?}",
            velocity.0,
        );

        // After fix: zero-velocity bolt should remain zero because we normalize
        // and multiply by original speed (0). Current code adds to velocity,
        // making it nonzero — but speed preservation would keep it at zero.
        assert!(
            velocity.0.length() < f32::EPSILON,
            "zero-velocity bolt should remain at zero speed after gravity well, got {:?}",
            velocity.0,
        );
    }

    #[test]
    fn gravity_well_bolt_at_center_is_not_modified() {
        let mut app = gravity_test_app();

        spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 200.0);
        let bolt = spawn_bolt(&mut app, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));

        tick(&mut app);

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            (velocity.0.x).abs() < f32::EPSILON,
            "bolt at well center should have unchanged x-velocity, got {}",
            velocity.0.x,
        );
        assert!(
            (velocity.0.y - 400.0).abs() < f32::EPSILON,
            "bolt at well center should have unchanged y-velocity, got {}",
            velocity.0.y,
        );
    }
}
