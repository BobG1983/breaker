//! Shared helpers for effect fire/reverse functions.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

/// Returns the entity's [`Position2D`] value, or [`Vec2::ZERO`] if absent.
pub(crate) fn entity_position(world: &World, entity: Entity) -> Vec2 {
    world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0)
}

/// Computes the effective range for an area-of-effect based on stacks.
///
/// Formula: `base_range + u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX) as f32 * range_per_level`
///
/// For `stacks >= 1`, the extra range is `(stacks - 1) * range_per_level` (capped at `u16::MAX` levels).
/// For `stacks == 0`, `saturating_sub(1)` wraps to `u32::MAX` which saturates to `u16::MAX`.
pub(crate) fn effective_range(base_range: f32, range_per_level: f32, stacks: u32) -> f32 {
    let extra = u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX);
    base_range + f32::from(extra) * range_per_level
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- A4: effective_range with stacks=0 returns base_range ──────────────

    #[test]
    fn effective_range_stacks_zero_returns_base_range() {
        let result = effective_range(50.0, 10.0, 0);
        assert!(
            (result - 50.0).abs() < f32::EPSILON,
            "stacks=0: expected 50.0, got {result}"
        );
    }

    // -- A5: effective_range with stacks=1 returns base ────────────────────

    #[test]
    fn effective_range_stacks_one_returns_base() {
        let result = effective_range(100.0, 20.0, 1);
        assert!(
            (result - 100.0).abs() < f32::EPSILON,
            "stacks=1: expected 100.0, got {result}"
        );
    }

    // -- A6: effective_range with stacks=3 linear scaling ──────────────────

    #[test]
    fn effective_range_stacks_three_linear_scaling() {
        let result = effective_range(100.0, 20.0, 3);
        assert!(
            (result - 140.0).abs() < f32::EPSILON,
            "stacks=3: expected 140.0, got {result}"
        );
    }

    #[test]
    fn effective_range_stacks_two_linear_scaling() {
        let result = effective_range(100.0, 20.0, 2);
        assert!(
            (result - 120.0).abs() < f32::EPSILON,
            "stacks=2: expected 120.0, got {result}"
        );
    }

    // -- A7: effective_range with stacks=u32::MAX caps extra at u16::MAX ──

    #[test]
    fn effective_range_stacks_u32_max_caps_at_u16_max() {
        let result = effective_range(100.0, 1.0, u32::MAX);
        assert!(
            (result - 65635.0).abs() < f32::EPSILON,
            "stacks=u32::MAX: expected 65635.0, got {result}"
        );
    }

    #[test]
    fn effective_range_stacks_u16_max_plus_two_caps_at_u16_max() {
        let stacks = u32::from(u16::MAX) + 2; // 65537
        let result = effective_range(100.0, 1.0, stacks);
        assert!(
            (result - 65635.0).abs() < f32::EPSILON,
            "stacks=65537: expected 65635.0 (cap at u16::MAX), got {result}"
        );
    }
}
