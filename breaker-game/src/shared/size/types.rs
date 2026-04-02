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

/// Computes effective bolt radius from base radius, boost multiplier,
/// node scale, and optional min/max clamping.
///
/// Formula: `base_radius * size_boost_multiplier * node_scaling_factor`, then clamped.
#[must_use]
pub(crate) fn effective_radius(
    base_radius: f32,
    size_boost_multiplier: f32,
    node_scaling_factor: f32,
    radius_range: ClampRange,
) -> f32 {
    radius_range.apply(base_radius * size_boost_multiplier * node_scaling_factor)
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
