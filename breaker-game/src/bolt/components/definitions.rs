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
