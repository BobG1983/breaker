//! Unified size computation — pure functions for entity dimensions.
//!
//! `effective_size` is the single source of truth
//! for computing entity dimensions from base values, boost multipliers,
//! node scaling, and optional min/max clamping.

use bevy::prelude::*;

/// Minimum effective width after boosts and scale.
#[derive(Component, Debug, Clone, Copy)]
pub struct MinWidth(pub f32);

/// Maximum effective width after boosts and scale.
#[derive(Component, Debug, Clone, Copy)]
pub struct MaxWidth(pub f32);

/// Minimum effective height after boosts and scale.
#[derive(Component, Debug, Clone, Copy)]
pub struct MinHeight(pub f32);

/// Maximum effective height after boosts and scale.
#[derive(Component, Debug, Clone, Copy)]
pub struct MaxHeight(pub f32);

/// Base radius of a circular entity in world units.
#[derive(Component, Debug, Clone, Copy)]
pub struct BaseRadius(pub f32);

/// Minimum effective radius after boosts and scale.
#[derive(Component, Debug, Clone, Copy)]
pub struct MinRadius(pub f32);

/// Maximum effective radius after boosts and scale.
#[derive(Component, Debug, Clone, Copy)]
pub struct MaxRadius(pub f32);

/// Optional min/max clamp range for a single dimension.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClampRange {
    /// Minimum value. `None` = no lower bound.
    pub min: Option<f32>,
    /// Maximum value. `None` = no upper bound.
    pub max: Option<f32>,
}

impl ClampRange {
    /// No clamping.
    pub const NONE: Self = Self {
        min: None,
        max: None,
    };

    /// Apply the clamp range to a value.
    const fn apply(self, value: f32) -> f32 {
        let v = match self.min {
            Some(min) if min > value => min,
            _ => value,
        };
        match self.max {
            Some(max) if max < v => max,
            _ => v,
        }
    }
}

/// Computes effective entity dimensions from base values, boost multiplier,
/// node scale, and optional min/max clamping per dimension.
///
/// Formula: `base * size_boost_multiplier * node_scaling_factor`, then clamped.
/// When clamp range min/max are `None`, clamping is skipped for that dimension.
#[must_use]
pub(crate) fn effective_size(
    base_width: f32,
    base_height: f32,
    size_boost_multiplier: f32,
    node_scaling_factor: f32,
    width_range: ClampRange,
    height_range: ClampRange,
) -> Vec2 {
    let w = width_range.apply(base_width * size_boost_multiplier * node_scaling_factor);
    let h = height_range.apply(base_height * size_boost_multiplier * node_scaling_factor);
    Vec2::new(w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Part A: effective_size pure function tests ───────────────────

    // Behavior 1: No boosts, no node scale, no constraints returns base dimensions

    #[test]
    fn effective_size_no_boosts_no_scale_returns_base_dimensions() {
        let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
            "expected (120.0, 20.0), got ({}, {})",
            result.x,
            result.y,
        );
    }

    // Behavior 2: Size boost multiplier scales both width and height

    #[test]
    fn effective_size_boost_scales_both_width_and_height() {
        let result = effective_size(
            120.0,
            20.0,
            4.0_f32 / 3.0,
            1.0,
            ClampRange::NONE,
            ClampRange::NONE,
        );
        assert!(
            (result.x - 160.0).abs() < 1e-3,
            "expected width 160.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 20.0 * 4.0 / 3.0).abs() < 1e-3,
            "expected height ~26.666, got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_identity_boost_returns_base() {
        let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
            "identity boost should return base, got ({}, {})",
            result.x,
            result.y,
        );
    }

    // Behavior 3: Node scaling factor scales both width and height

    #[test]
    fn effective_size_node_scale_scales_both_dimensions() {
        let result = effective_size(120.0, 20.0, 1.0, 0.7, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 84.0).abs() < 1e-3,
            "expected width 84.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 14.0).abs() < 1e-3,
            "expected height 14.0, got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_identity_node_scale_returns_base() {
        let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
            "identity node scale should return base, got ({}, {})",
            result.x,
            result.y,
        );
    }

    // Behavior 4: Both boost and node scale multiply together

    #[test]
    fn effective_size_boost_and_node_scale_multiply() {
        let result = effective_size(
            120.0,
            20.0,
            4.0_f32 / 3.0,
            0.7,
            ClampRange::NONE,
            ClampRange::NONE,
        );
        // 120.0 * 4/3 * 0.7 = 112.0, 20.0 * 4/3 * 0.7 = 18.666...
        assert!(
            (result.x - 112.0).abs() < 1e-3,
            "expected width 112.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 18.666_666).abs() < 1e-2,
            "expected height ~18.666, got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_large_boost_with_half_scale() {
        let result = effective_size(120.0, 20.0, 3.0, 0.5, ClampRange::NONE, ClampRange::NONE);
        // 120.0 * 3.0 * 0.5 = 180.0, 20.0 * 3.0 * 0.5 = 30.0
        assert!(
            (result.x - 180.0).abs() < 1e-3,
            "expected width 180.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 30.0).abs() < 1e-3,
            "expected height 30.0, got {}",
            result.y,
        );
    }

    // Behavior 5: Clamps width to max when boost would exceed it

    #[test]
    fn effective_size_clamps_width_to_max() {
        // 120.0 * 3.0 = 360.0, clamped to max 200.0
        // 20.0 * 3.0 = 60.0, within bounds [10.0, 100.0]
        let result = effective_size(
            120.0,
            20.0,
            3.0,
            1.0,
            ClampRange {
                min: Some(60.0),
                max: Some(200.0),
            },
            ClampRange {
                min: Some(10.0),
                max: Some(100.0),
            },
        );
        assert!(
            (result.x - 200.0).abs() < 1e-3,
            "expected width clamped to 200.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 60.0).abs() < 1e-3,
            "expected height 60.0, got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_clamps_both_to_max() {
        // 120.0 * 10.0 = 1200.0 -> 200.0, 20.0 * 10.0 = 200.0 -> 100.0
        let result = effective_size(
            120.0,
            20.0,
            10.0,
            1.0,
            ClampRange {
                min: Some(60.0),
                max: Some(200.0),
            },
            ClampRange {
                min: Some(10.0),
                max: Some(100.0),
            },
        );
        assert!(
            (result.x - 200.0).abs() < 1e-3,
            "expected width clamped to 200.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 100.0).abs() < 1e-3,
            "expected height clamped to 100.0, got {}",
            result.y,
        );
    }

    // Behavior 6: Clamps to min when scale would shrink below it

    #[test]
    fn effective_size_clamps_to_min_when_scaled_small() {
        // 120.0 * 0.1 = 12.0, clamped to min 60.0
        // 20.0 * 0.1 = 2.0, clamped to min 10.0
        let result = effective_size(
            120.0,
            20.0,
            1.0,
            0.1,
            ClampRange {
                min: Some(60.0),
                max: Some(600.0),
            },
            ClampRange {
                min: Some(10.0),
                max: Some(100.0),
            },
        );
        assert!(
            (result.x - 60.0).abs() < 1e-3,
            "expected width clamped to min 60.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 10.0).abs() < 1e-3,
            "expected height clamped to min 10.0, got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_exactly_at_min_boundary() {
        // 120.0 * 0.5 = 60.0 == min, 20.0 * 0.5 = 10.0 == min
        let result = effective_size(
            120.0,
            20.0,
            1.0,
            0.5,
            ClampRange {
                min: Some(60.0),
                max: Some(600.0),
            },
            ClampRange {
                min: Some(10.0),
                max: Some(100.0),
            },
        );
        assert!(
            (result.x - 60.0).abs() < 1e-3,
            "expected width at min 60.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 10.0).abs() < 1e-3,
            "expected height at min 10.0, got {}",
            result.y,
        );
    }

    // Behavior 7: At exactly max boundary stays at max

    #[test]
    fn effective_size_exactly_at_max_boundary() {
        // 120.0 * 5.0 = 600.0 == max, 20.0 * 5.0 = 100.0 == max
        let result = effective_size(
            120.0,
            20.0,
            5.0,
            1.0,
            ClampRange {
                min: Some(60.0),
                max: Some(600.0),
            },
            ClampRange {
                min: Some(10.0),
                max: Some(100.0),
            },
        );
        assert!(
            (result.x - 600.0).abs() < 1e-3,
            "expected width at max 600.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 100.0).abs() < 1e-3,
            "expected height at max 100.0, got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_slightly_below_max_not_clamped() {
        // 120.0 * 4.99 = 598.8, 20.0 * 4.99 = 99.8 -- both under max
        let result = effective_size(
            120.0,
            20.0,
            4.99,
            1.0,
            ClampRange {
                min: Some(60.0),
                max: Some(600.0),
            },
            ClampRange {
                min: Some(10.0),
                max: Some(100.0),
            },
        );
        assert!(
            (result.x - 598.8).abs() < 1e-1,
            "expected width 598.8 (not clamped), got {}",
            result.x,
        );
        assert!(
            (result.y - 99.8).abs() < 1e-1,
            "expected height 99.8 (not clamped), got {}",
            result.y,
        );
    }

    // Behavior 8: None min/max skips clamping entirely

    #[test]
    fn effective_size_none_constraints_no_clamping() {
        // 120.0 * 10.0 = 1200.0, 20.0 * 10.0 = 200.0 -- unclamped
        let result = effective_size(120.0, 20.0, 10.0, 1.0, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 1200.0).abs() < 1e-3,
            "expected width 1200.0 (unclamped), got {}",
            result.x,
        );
        assert!(
            (result.y - 200.0).abs() < 1e-3,
            "expected height 200.0 (unclamped), got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_none_constraints_very_small_scale() {
        // 120.0 * 0.01 = 1.2, 20.0 * 0.01 = 0.2 -- unclamped
        let result = effective_size(120.0, 20.0, 1.0, 0.01, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 1.2).abs() < 1e-3,
            "expected width 1.2 (unclamped), got {}",
            result.x,
        );
        assert!(
            (result.y - 0.2).abs() < 1e-3,
            "expected height 0.2 (unclamped), got {}",
            result.y,
        );
    }

    // Behavior 9: Partial None constraints clamp only specified dimension

    #[test]
    fn effective_size_partial_constraints_clamps_only_specified() {
        // max_width = Some(200.0), rest None
        // width: 120.0 * 10.0 = 1200.0, clamped to 200.0
        // height: 20.0 * 10.0 = 200.0, unclamped (None)
        let result = effective_size(
            120.0,
            20.0,
            10.0,
            1.0,
            ClampRange {
                min: None,
                max: Some(200.0),
            },
            ClampRange::NONE,
        );
        assert!(
            (result.x - 200.0).abs() < 1e-3,
            "expected width clamped to 200.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 200.0).abs() < 1e-3,
            "expected height 200.0 (unclamped), got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_only_min_width_specified() {
        // min_width = Some(200.0), rest None, node_scaling_factor = 0.01
        // width: 120.0 * 0.01 = 1.2, clamped to 200.0
        // height: 20.0 * 0.01 = 0.2, unclamped
        let result = effective_size(
            120.0,
            20.0,
            1.0,
            0.01,
            ClampRange {
                min: Some(200.0),
                max: None,
            },
            ClampRange::NONE,
        );
        assert!(
            (result.x - 200.0).abs() < 1e-3,
            "expected width clamped to min 200.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 0.2).abs() < 1e-3,
            "expected height 0.2 (unclamped), got {}",
            result.y,
        );
    }

    // Behavior 10: Multiple stacked boosts (pre-computed multiplier)

    #[test]
    fn effective_size_stacked_boosts_precomputed() {
        // multiplier = 3.0 (product of [1.5, 2.0])
        // 120.0 * 3.0 = 360.0, 20.0 * 3.0 = 60.0
        let result = effective_size(120.0, 20.0, 3.0, 1.0, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 360.0).abs() < 1e-3,
            "expected width 360.0, got {}",
            result.x,
        );
        assert!(
            (result.y - 60.0).abs() < 1e-3,
            "expected height 60.0, got {}",
            result.y,
        );
    }

    #[test]
    fn effective_size_empty_boosts_identity_multiplier() {
        // empty boosts => multiplier = 1.0
        let result = effective_size(120.0, 20.0, 1.0, 1.0, ClampRange::NONE, ClampRange::NONE);
        assert!(
            (result.x - 120.0).abs() < f32::EPSILON && (result.y - 20.0).abs() < f32::EPSILON,
            "empty boosts (multiplier 1.0) should return base, got ({}, {})",
            result.x,
            result.y,
        );
    }
}
