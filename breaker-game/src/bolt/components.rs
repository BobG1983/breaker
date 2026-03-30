//! Bolt domain components.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{InterpolateTransform2D, Spatial2D, Velocity2D};

/// Marker component identifying the bolt entity.
#[derive(Component, Debug, Default)]
#[require(Spatial2D, InterpolateTransform2D, Velocity2D)]
pub struct Bolt;

/// Marker component indicating the bolt is hovering above the breaker,
/// waiting for the player to launch it. Present only on the first node.
#[derive(Component, Debug)]
pub struct BoltServing;

/// Base speed in world units per second.
#[derive(Component, Debug)]
pub struct BoltBaseSpeed(pub f32);

/// Minimum speed cap.
#[derive(Component, Debug)]
pub struct BoltMinSpeed(pub f32);

/// Maximum speed cap.
#[derive(Component, Debug)]
pub struct BoltMaxSpeed(pub f32);

/// Bolt radius in world units.
#[derive(Component, Debug)]
pub struct BoltRadius(pub f32);

/// Vertical offset above the breaker where the bolt spawns.
#[derive(Component, Debug)]
pub struct BoltSpawnOffsetY(pub f32);

/// Vertical offset above the breaker for bolt respawn after loss.
#[derive(Component, Debug)]
pub struct BoltRespawnOffsetY(pub f32);

/// Maximum respawn angle spread from vertical in radians.
#[derive(Component, Debug)]
pub struct BoltRespawnAngleSpread(pub f32);

/// Initial launch angle from vertical in radians.
#[derive(Component, Debug)]
pub struct BoltInitialAngle(pub f32);

/// Adjusts velocity so it never gets too close to horizontal (free-function variant).
///
/// If the angle from horizontal is less than `min_angle`, rotates the
/// vector to the minimum angle while preserving speed and Y sign.
/// Zero velocity is returned unchanged.
///
/// This is the `Velocity2D`-compatible replacement for
/// the old `BoltVelocity::enforce_min_angle`.
pub fn enforce_min_angle(velocity: &mut Vec2, min_angle: f32) {
    let speed = velocity.length();
    if speed < f32::EPSILON {
        return;
    }

    let angle_from_horizontal = velocity.y.abs().atan2(velocity.x.abs());
    if angle_from_horizontal < min_angle {
        let sign_x = velocity.x.signum();
        let sign_y = if velocity.y.abs() < f32::EPSILON {
            1.0 // Default to upward if perfectly horizontal
        } else {
            velocity.y.signum()
        };
        velocity.x = sign_x * speed * min_angle.cos();
        velocity.y = sign_y * speed * min_angle.sin();
    }
}

/// Marker for extra bolts spawned by breaker consequences (e.g. Prism).
///
/// Extra bolts are despawned on loss rather than respawned. Only the
/// baseline bolt (without this marker) respawns.
#[derive(Component, Debug)]
pub struct ExtraBolt;

/// Marks a bolt as having been spawned by an evolution chip.
///
/// Used for damage attribution — cell kills by this bolt count toward the
/// named evolution's cumulative damage for the `MostPowerfulEvolution` highlight.
#[derive(Component, Debug, Clone)]
pub struct SpawnedByEvolution(pub String);

/// Remaining pierces before exhaustion. Reset to [`EffectivePiercing`] on
/// wall/breaker contact.
///
/// This is bolt gameplay state — decremented by `bolt_cell_collision` on each
/// pierce-through, reset by `bolt_wall_collision` and `bolt_breaker_collision`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiercingRemaining(pub u32);

/// Countdown timer that despawns the bolt when it expires.
///
/// Used by phantom bolts and other temporary bolt-like entities
/// to auto-destroy after a configured duration.
#[derive(Component, Debug)]
pub struct BoltLifespan(pub Timer);

#[cfg(test)]
mod tests {
    use super::*;

    // ── Bolt #[require] tests ────────────────────────────────────

    #[test]
    fn bolt_require_inserts_spatial2d() {
        use rantzsoft_spatial2d::components::Spatial2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Bolt).id();
        app.update();
        assert!(
            app.world().get::<Spatial2D>(entity).is_some(),
            "Bolt should auto-insert Spatial2D via #[require]"
        );
    }

    #[test]
    fn bolt_require_inserts_interpolate_transform2d() {
        use rantzsoft_spatial2d::components::InterpolateTransform2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Bolt).id();
        app.update();
        assert!(
            app.world().get::<InterpolateTransform2D>(entity).is_some(),
            "Bolt should auto-insert InterpolateTransform2D via #[require]"
        );
    }

    #[test]
    fn bolt_require_inserts_bolt_velocity_default() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Bolt).id();
        app.update();
        let velocity = app
            .world()
            .get::<Velocity2D>(entity)
            .expect("Bolt should auto-insert Velocity2D via #[require]");
        assert_eq!(
            velocity.0,
            Vec2::ZERO,
            "default Velocity2D should have value Vec2::ZERO"
        );
    }

    #[test]
    fn bolt_explicit_values_override_require_defaults() {
        use rantzsoft_spatial2d::components::Position2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(10.0, 20.0)),
            ))
            .id();
        app.update();
        let velocity = app
            .world()
            .get::<Velocity2D>(entity)
            .expect("Velocity2D should be present");
        assert!(
            (velocity.0.x - 0.0).abs() < f32::EPSILON
                && (velocity.0.y - 400.0).abs() < f32::EPSILON,
            "explicit Velocity2D(0.0, 400.0) should override the default, got {:?}",
            velocity.0
        );
        let position = app
            .world()
            .get::<Position2D>(entity)
            .expect("Position2D should be present");
        assert_eq!(
            position.0,
            Vec2::new(10.0, 20.0),
            "explicit Position2D(10.0, 20.0) should override the default"
        );
    }

    #[test]
    fn bolt_require_does_not_insert_cleanup_on_run_end() {
        use crate::shared::CleanupOnRunEnd;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Bolt).id();
        app.update();
        assert!(
            app.world().get::<CleanupOnRunEnd>(entity).is_none(),
            "Bolt #[require] should NOT auto-insert CleanupOnRunEnd"
        );
    }

    #[test]
    fn bolt_require_does_not_insert_cleanup_on_node_exit() {
        use crate::shared::CleanupOnNodeExit;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Bolt).id();
        app.update();
        assert!(
            app.world().get::<CleanupOnNodeExit>(entity).is_none(),
            "Bolt #[require] should NOT auto-insert CleanupOnNodeExit"
        );
    }

    // ── Velocity2D migration tests ────────────────────────────────

    #[test]
    fn bolt_require_inserts_velocity2d_default() {
        use rantzsoft_spatial2d::components::Velocity2D;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(Bolt).id();
        app.update();
        let velocity = app
            .world()
            .get::<Velocity2D>(entity)
            .expect("Bolt should auto-insert Velocity2D via #[require]");
        assert_eq!(
            velocity.0,
            Vec2::ZERO,
            "default Velocity2D should be Vec2::ZERO"
        );
    }

    // ── enforce_min_angle free function tests ─────────────────────

    #[test]
    fn free_enforce_min_angle_corrects_shallow() {
        use std::f32::consts::FRAC_PI_4;
        let mut velocity = Vec2::new(10.0, 0.01);
        let speed_before = velocity.length();
        enforce_min_angle(&mut velocity, FRAC_PI_4);
        let speed_after = velocity.length();
        assert!(
            (speed_before - speed_after).abs() < 1e-4,
            "speed should be preserved: before={speed_before}, after={speed_after}"
        );
        let angle = velocity.y.abs().atan2(velocity.x.abs());
        assert!(
            angle >= FRAC_PI_4 - 1e-4,
            "angle {angle} should be >= PI/4 ({FRAC_PI_4})"
        );
    }

    #[test]
    fn free_enforce_min_angle_preserves_signs() {
        use std::f32::consts::FRAC_PI_4;
        let mut velocity = Vec2::new(-10.0, -0.01);
        enforce_min_angle(&mut velocity, FRAC_PI_4);
        assert!(
            velocity.x < 0.0,
            "x sign should be negative, got {}",
            velocity.x
        );
        assert!(
            velocity.y < 0.0,
            "y sign should be negative, got {}",
            velocity.y
        );
    }

    #[test]
    fn free_enforce_min_angle_leaves_steep_unchanged() {
        use crate::breaker::resources::BreakerConfig;
        let mut velocity = Vec2::new(1.0, 5.0);
        let original = velocity;
        enforce_min_angle(
            &mut velocity,
            BreakerConfig::default()
                .min_angle_from_horizontal
                .to_radians(),
        );
        assert!(
            (velocity.x - original.x).abs() < 1e-6,
            "steep velocity x should be unchanged"
        );
        assert!(
            (velocity.y - original.y).abs() < 1e-6,
            "steep velocity y should be unchanged"
        );
    }

    #[test]
    fn free_enforce_min_angle_zero_velocity_unchanged() {
        use std::f32::consts::FRAC_PI_4;
        let mut velocity = Vec2::ZERO;
        enforce_min_angle(&mut velocity, FRAC_PI_4);
        assert_eq!(velocity, Vec2::ZERO, "zero velocity should remain zero");
    }

    // ── CollisionLayers tests ──────────────────────────────────────

    #[test]
    fn bolt_collision_layers_have_correct_values() {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;

        use crate::shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            ))
            .id();
        app.update();
        let layers = app
            .world()
            .get::<CollisionLayers>(entity)
            .expect("Bolt should have CollisionLayers");
        assert_eq!(
            layers.membership, BOLT_LAYER,
            "Bolt membership should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
            layers.membership
        );
        assert_eq!(
            layers.mask,
            CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
            "Bolt mask should be CELL|WALL|BREAKER (0x{:02X}), got 0x{:02X}",
            CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
            layers.mask
        );
    }
}

#[cfg(test)]
mod enforce_min_angle_coverage {
    use super::*;

    /// Helper: assert speed is preserved within tolerance.
    fn assert_speed_preserved(vx: f32, vy: f32, min_deg: f32) {
        let mut velocity = Vec2::new(vx, vy);
        let speed_before = velocity.length();
        if speed_before < f32::EPSILON {
            return;
        }
        enforce_min_angle(&mut velocity, min_deg.to_radians());
        let speed_after = velocity.length();
        assert!(
            (speed_before - speed_after).abs() < 0.1,
            "speed should be preserved: {speed_before} vs {speed_after} (input: ({vx}, {vy}), min_deg: {min_deg})"
        );
    }

    #[test]
    fn preserves_speed_nearly_horizontal_positive() {
        assert_speed_preserved(400.0, 0.01, 15.0);
    }

    #[test]
    fn preserves_speed_nearly_horizontal_negative() {
        assert_speed_preserved(-300.0, -0.01, 30.0);
    }

    #[test]
    fn preserves_speed_diagonal_positive() {
        assert_speed_preserved(200.0, 200.0, 10.0);
    }

    #[test]
    fn preserves_speed_diagonal_negative() {
        assert_speed_preserved(-250.0, -250.0, 45.0);
    }

    #[test]
    fn preserves_speed_nearly_vertical() {
        assert_speed_preserved(0.01, 500.0, 5.0);
    }

    #[test]
    fn preserves_speed_at_boundary_angle() {
        assert_speed_preserved(100.0, 26.8, 15.0); // ~15 deg from horizontal
    }

    #[test]
    fn preserves_speed_large_magnitude() {
        assert_speed_preserved(-499.0, 499.0, 20.0);
    }

    /// Helper: assert result is finite (no NaN/infinity).
    fn assert_finite(vx: f32, vy: f32, min_deg: f32) {
        let mut velocity = Vec2::new(vx, vy);
        enforce_min_angle(&mut velocity, min_deg.to_radians());
        assert!(
            velocity.x.is_finite(),
            "x should be finite: {} (input: ({vx}, {vy}), min_deg: {min_deg})",
            velocity.x
        );
        assert!(
            velocity.y.is_finite(),
            "y should be finite: {} (input: ({vx}, {vy}), min_deg: {min_deg})",
            velocity.y
        );
    }

    #[test]
    fn never_nan_horizontal() {
        assert_finite(500.0, 0.0, 20.0);
    }

    #[test]
    fn never_nan_vertical() {
        assert_finite(0.0, 500.0, 45.0);
    }

    #[test]
    fn never_nan_zero_velocity() {
        assert_finite(0.0, 0.0, 10.0);
    }

    #[test]
    fn never_nan_near_zero() {
        assert_finite(0.001, -0.001, 1.0);
    }

    #[test]
    fn never_nan_large_values() {
        assert_finite(-999.0, 999.0, 89.0);
    }

    #[test]
    fn never_nan_small_min_angle() {
        assert_finite(100.0, -50.0, 1.0);
    }

    #[test]
    fn never_nan_negative_both() {
        assert_finite(-700.0, -700.0, 30.0);
    }

    /// Helper: assert angle from horizontal meets minimum.
    fn assert_meets_minimum(vx: f32, vy: f32, min_deg: f32) {
        let mut velocity = Vec2::new(vx, vy);
        if velocity.length() < f32::EPSILON {
            return;
        }
        let min_rad = min_deg.to_radians();
        enforce_min_angle(&mut velocity, min_rad);
        let angle = velocity.y.abs().atan2(velocity.x.abs());
        assert!(
            angle >= min_rad - 1e-4,
            "angle {angle:.4} should be >= min {min_rad:.4}, vel=({}, {}), input: ({vx}, {vy}), min_deg: {min_deg}",
            velocity.x,
            velocity.y
        );
    }

    #[test]
    fn meets_minimum_shallow_positive() {
        assert_meets_minimum(400.0, 1.0, 15.0);
    }

    #[test]
    fn meets_minimum_shallow_negative() {
        assert_meets_minimum(-400.0, -1.0, 30.0);
    }

    #[test]
    fn meets_minimum_already_steep() {
        assert_meets_minimum(100.0, 400.0, 10.0);
    }

    #[test]
    fn meets_minimum_exactly_at_boundary() {
        // 45 degrees exactly
        assert_meets_minimum(100.0, 100.0, 45.0);
    }

    #[test]
    fn meets_minimum_nearly_horizontal_large_min() {
        assert_meets_minimum(500.0, 0.01, 45.0);
    }

    #[test]
    fn meets_minimum_mixed_signs() {
        assert_meets_minimum(-300.0, 50.0, 20.0);
    }

    #[test]
    fn meets_minimum_small_angle_requirement() {
        assert_meets_minimum(200.0, 10.0, 5.0);
    }
}
