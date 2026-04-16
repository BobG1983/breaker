//! Inverse-square attraction force calculation.

use bevy::prelude::*;

/// Computes an attraction force vector pointing from `target` toward `source`
/// with magnitude `strength / max(distance_squared, min_distance_squared)`.
///
/// Returns `Vec2::ZERO` when `source == target` (avoids NaN from zero-length
/// normalization).
///
/// # Arguments
///
/// * `source` - Position of the attractor (the magnetic cell center).
/// * `target` - Position of the attracted object (the bolt).
/// * `strength` - Force strength coefficient.
/// * `min_distance` - Minimum distance clamp to prevent infinite force at very
///   close range. The actual clamping uses `min_distance * min_distance`.
pub(crate) fn inverse_square_attraction(
    source: Vec2,
    target: Vec2,
    strength: f32,
    min_distance: f32,
) -> Vec2 {
    let delta = source - target;
    let direction = delta.normalize_or_zero();
    let d_squared = delta.length_squared();
    let min_d_squared = min_distance * min_distance;
    let magnitude = strength / d_squared.max(min_d_squared);
    direction * magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: Force direction points from target toward source ──

    #[test]
    fn force_direction_points_from_target_toward_source_along_x() {
        let force =
            inverse_square_attraction(Vec2::new(100.0, 0.0), Vec2::new(0.0, 0.0), 100.0, 5.0);
        assert!(
            force.x > 0.0,
            "force x should be positive (toward source at positive x), got {}",
            force.x
        );
        assert!(
            force.y.abs() < 1e-6,
            "force y should be ~0.0, got {}",
            force.y
        );
    }

    #[test]
    fn force_direction_points_from_target_toward_source_along_y() {
        let force =
            inverse_square_attraction(Vec2::new(0.0, 100.0), Vec2::new(0.0, 0.0), 100.0, 5.0);
        assert!(
            force.y > 0.0,
            "force y should be positive (toward source at positive y), got {}",
            force.y
        );
        assert!(
            force.x.abs() < 1e-6,
            "force x should be ~0.0, got {}",
            force.x
        );
    }

    // ── Behavior 2: Magnitude follows inverse-square at distance > min_distance ──

    #[test]
    fn magnitude_follows_inverse_square_at_distance_100() {
        let force =
            inverse_square_attraction(Vec2::new(100.0, 0.0), Vec2::new(0.0, 0.0), 100.0, 5.0);
        // distance = 100.0, distance_squared = 10000.0, magnitude = 100.0 / 10000.0 = 0.01
        let magnitude = force.length();
        assert!(
            (magnitude - 0.01).abs() < 1e-6,
            "magnitude should be 0.01, got {magnitude}"
        );
    }

    #[test]
    fn magnitude_follows_inverse_square_at_distance_10() {
        let force =
            inverse_square_attraction(Vec2::new(10.0, 0.0), Vec2::new(0.0, 0.0), 100.0, 5.0);
        // distance = 10.0, distance_squared = 100.0, magnitude = 100.0 / 100.0 = 1.0
        let magnitude = force.length();
        assert!(
            (magnitude - 1.0).abs() < 1e-6,
            "magnitude should be 1.0, got {magnitude}"
        );
    }

    // ── Behavior 3: Magnitude clamps to min_distance_squared when distance < min_distance ──

    #[test]
    fn magnitude_clamps_to_min_distance_squared_when_distance_below_min() {
        let force = inverse_square_attraction(Vec2::new(2.0, 0.0), Vec2::new(0.0, 0.0), 100.0, 5.0);
        // actual distance = 2.0 < min_distance = 5.0
        // uses min_distance_squared = 25.0
        // magnitude = 100.0 / 25.0 = 4.0
        // direction = (1.0, 0.0), force = (4.0, 0.0)
        assert!(
            (force.x - 4.0).abs() < 1e-6,
            "force x should be 4.0, got {}",
            force.x
        );
        assert!(
            force.y.abs() < 1e-6,
            "force y should be ~0.0, got {}",
            force.y
        );
    }

    #[test]
    fn magnitude_at_exactly_min_distance_uses_min_distance_squared() {
        let force = inverse_square_attraction(Vec2::new(5.0, 0.0), Vec2::new(0.0, 0.0), 100.0, 5.0);
        // distance = 5.0 = min_distance, d_sq = 25.0 = min_distance_sq
        // magnitude = 100.0 / 25.0 = 4.0
        assert!(
            (force.x - 4.0).abs() < 1e-6,
            "force x should be 4.0 at exactly min_distance, got {}",
            force.x
        );
    }

    // ── Behavior 4: Source equals target returns Vec2::ZERO ──

    #[test]
    fn source_equals_target_returns_zero() {
        let force =
            inverse_square_attraction(Vec2::new(50.0, 30.0), Vec2::new(50.0, 30.0), 1000.0, 5.0);
        assert_eq!(
            force,
            Vec2::ZERO,
            "coincident positions should return Vec2::ZERO, got {force:?}"
        );
        assert!(
            !force.x.is_nan() && !force.y.is_nan(),
            "result should not be NaN"
        );
    }

    #[test]
    fn both_at_origin_returns_zero() {
        let force = inverse_square_attraction(Vec2::ZERO, Vec2::ZERO, 1000.0, 5.0);
        assert_eq!(
            force,
            Vec2::ZERO,
            "both at origin should return Vec2::ZERO, got {force:?}"
        );
    }

    // ── Behavior 5: Diagonal distance uses Euclidean length ──

    #[test]
    fn diagonal_distance_uses_euclidean_length() {
        let force = inverse_square_attraction(Vec2::new(3.0, 4.0), Vec2::new(0.0, 0.0), 250.0, 1.0);
        // distance = 5.0, distance_squared = 25.0, magnitude = 250.0 / 25.0 = 10.0
        // direction = (3.0/5.0, 4.0/5.0) = (0.6, 0.8)
        // force = (6.0, 8.0), length = 10.0
        assert!(
            (force.x - 6.0).abs() < 1e-4,
            "force x should be 6.0, got {}",
            force.x
        );
        assert!(
            (force.y - 8.0).abs() < 1e-4,
            "force y should be 8.0, got {}",
            force.y
        );
        let magnitude = force.length();
        assert!(
            (magnitude - 10.0).abs() < 1e-4,
            "force magnitude should be 10.0, got {magnitude}"
        );
    }

    // ── Behavior 6: Very large distance produces very small force ──

    #[test]
    fn very_large_distance_produces_very_small_force() {
        let force =
            inverse_square_attraction(Vec2::new(1000.0, 0.0), Vec2::new(0.0, 0.0), 100.0, 5.0);
        // distance_squared = 1_000_000.0, magnitude = 100.0 / 1_000_000.0 = 0.0001
        let magnitude = force.length();
        assert!(
            (magnitude - 0.0001).abs() < 1e-8,
            "magnitude should be 0.0001 at large distance, got {magnitude}"
        );
        assert!(
            magnitude.is_finite() && magnitude > 0.0,
            "force should be finite and non-zero, got {magnitude}"
        );
    }
}
