//! Bolt domain components.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{InterpolateTransform2D, Spatial2D, Velocity2D};

/// Marker component identifying the bolt entity.
#[derive(Component, Debug, Default)]
#[require(Spatial2D, InterpolateTransform2D, BoltVelocity, Velocity2D)]
pub struct Bolt;

/// Marker component indicating the bolt is hovering above the breaker,
/// waiting for the player to launch it. Present only on the first node.
#[derive(Component, Debug)]
pub struct BoltServing;

/// The bolt's velocity in world units per second.
#[derive(Component, Debug, Clone, Default)]
pub struct BoltVelocity {
    /// Velocity vector (x, y).
    pub value: Vec2,
}

impl BoltVelocity {
    /// Creates a new bolt velocity.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            value: Vec2::new(x, y),
        }
    }

    /// Returns the current speed (magnitude of velocity).
    #[must_use]
    pub fn speed(&self) -> f32 {
        self.value.length()
    }

    /// Returns the normalized direction vector.
    #[must_use]
    pub fn direction(&self) -> Vec2 {
        self.value.normalize_or_zero()
    }

    /// Adjusts velocity so it never gets too close to horizontal.
    ///
    /// If the angle from horizontal is less than `min_angle`, rotates the
    /// vector to the minimum angle while preserving speed and Y sign.
    pub fn enforce_min_angle(&mut self, min_angle: f32) {
        let speed = self.value.length();
        if speed < f32::EPSILON {
            return;
        }

        let angle_from_horizontal = self.value.y.abs().atan2(self.value.x.abs());
        if angle_from_horizontal < min_angle {
            let sign_x = self.value.x.signum();
            let sign_y = if self.value.y.abs() < f32::EPSILON {
                1.0 // Default to upward if perfectly horizontal
            } else {
                self.value.y.signum()
            };
            self.value.x = sign_x * speed * min_angle.cos();
            self.value.y = sign_y * speed * min_angle.sin();
        }
    }
}

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
/// `BoltVelocity::enforce_min_angle`.
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

/// Marker for extra bolts spawned by archetype consequences (e.g. Prism).
///
/// Extra bolts are despawned on loss rather than respawned. Only the
/// baseline bolt (without this marker) respawns.
#[derive(Component, Debug)]
pub struct ExtraBolt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bolt_velocity_speed() {
        let vel = BoltVelocity::new(3.0, 4.0);
        assert!((vel.speed() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bolt_velocity_direction_normalized() {
        let vel = BoltVelocity::new(3.0, 4.0);
        let dir = vel.direction();
        assert!((dir.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bolt_velocity_zero_direction() {
        let vel = BoltVelocity::new(0.0, 0.0);
        assert_eq!(vel.direction(), Vec2::ZERO);
    }

    #[test]
    fn enforce_min_angle_leaves_steep_unchanged() {
        use crate::breaker::resources::BreakerConfig;
        let mut vel = BoltVelocity::new(1.0, 5.0);
        let original = vel.value;
        vel.enforce_min_angle(
            BreakerConfig::default()
                .min_angle_from_horizontal
                .to_radians(),
        );
        assert!((vel.value.x - original.x).abs() < 1e-6);
        assert!((vel.value.y - original.y).abs() < 1e-6);
    }

    #[test]
    fn enforce_min_angle_corrects_shallow() {
        use std::f32::consts::FRAC_PI_4;
        let mut vel = BoltVelocity::new(10.0, 0.01);
        let speed_before = vel.speed();
        vel.enforce_min_angle(FRAC_PI_4);
        let speed_after = vel.speed();
        assert!((speed_before - speed_after).abs() < 1e-4);
        let angle = vel.value.y.abs().atan2(vel.value.x.abs());
        assert!(angle >= FRAC_PI_4 - 1e-4);
    }

    #[test]
    fn enforce_min_angle_preserves_signs() {
        use std::f32::consts::FRAC_PI_4;
        let mut vel = BoltVelocity::new(-10.0, -0.01);
        vel.enforce_min_angle(FRAC_PI_4);
        assert!(vel.value.x < 0.0);
        assert!(vel.value.y < 0.0);
    }

    #[test]
    fn enforce_min_angle_horizontal_defaults_upward() {
        use std::f32::consts::FRAC_PI_4;
        let mut vel = BoltVelocity::new(10.0, 0.0);
        let speed_before = vel.speed();
        vel.enforce_min_angle(FRAC_PI_4);
        let speed_after = vel.speed();
        assert!(
            (speed_before - speed_after).abs() < 1e-4,
            "speed should be preserved"
        );
        assert!(
            vel.value.y > 0.0,
            "horizontal velocity should default to upward"
        );
        let angle = vel.value.y.abs().atan2(vel.value.x.abs());
        assert!(
            angle >= FRAC_PI_4 - 1e-4,
            "angle should be at least min_angle"
        );
    }

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
            .get::<BoltVelocity>(entity)
            .expect("Bolt should auto-insert BoltVelocity via #[require]");
        assert_eq!(
            velocity.value,
            Vec2::ZERO,
            "default BoltVelocity should have value Vec2::ZERO"
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
                BoltVelocity::new(0.0, 400.0),
                Position2D(Vec2::new(10.0, 20.0)),
            ))
            .id();
        app.update();
        let velocity = app
            .world()
            .get::<BoltVelocity>(entity)
            .expect("BoltVelocity should be present");
        assert!(
            (velocity.value.x - 0.0).abs() < f32::EPSILON
                && (velocity.value.y - 400.0).abs() < f32::EPSILON,
            "explicit BoltVelocity(0.0, 400.0) should override the default, got {:?}",
            velocity.value
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
mod proptests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// enforce_min_angle never changes the speed magnitude.
        #[test]
        fn enforce_min_angle_preserves_speed(
            vx in -500.0_f32..500.0,
            vy in -500.0_f32..500.0,
            min_deg in 5.0_f32..45.0,
        ) {
            let mut vel = BoltVelocity::new(vx, vy);
            let speed_before = vel.speed();
            if speed_before < f32::EPSILON {
                return Ok(());
            }
            vel.enforce_min_angle(min_deg.to_radians());
            let speed_after = vel.speed();
            prop_assert!(
                (speed_before - speed_after).abs() < 0.1,
                "speed should be preserved: {speed_before} vs {speed_after}"
            );
        }

        /// enforce_min_angle never produces NaN or infinity.
        #[test]
        fn enforce_min_angle_never_nan(
            vx in -1000.0_f32..1000.0,
            vy in -1000.0_f32..1000.0,
            min_deg in 1.0_f32..89.0,
        ) {
            let mut vel = BoltVelocity::new(vx, vy);
            vel.enforce_min_angle(min_deg.to_radians());
            prop_assert!(vel.value.x.is_finite(), "x should be finite: {}", vel.value.x);
            prop_assert!(vel.value.y.is_finite(), "y should be finite: {}", vel.value.y);
        }

        /// After enforce_min_angle, the angle from horizontal is >= min_angle.
        #[test]
        fn enforce_min_angle_result_meets_minimum(
            vx in -500.0_f32..500.0,
            vy in -500.0_f32..500.0,
            min_deg in 5.0_f32..45.0,
        ) {
            let mut vel = BoltVelocity::new(vx, vy);
            if vel.speed() < f32::EPSILON {
                return Ok(());
            }
            let min_rad = min_deg.to_radians();
            vel.enforce_min_angle(min_rad);
            let angle = vel.value.y.abs().atan2(vel.value.x.abs());
            prop_assert!(
                angle >= min_rad - 1e-4,
                "angle {angle:.4} should be >= min {min_rad:.4}, vel=({}, {})",
                vel.value.x, vel.value.y
            );
        }
    }
}
