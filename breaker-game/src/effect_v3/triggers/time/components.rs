//! Components for the time trigger category.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

/// Collection of active effect duration timers on an entity.
///
/// Each entry is a `(remaining_time, original_duration)` pair. Added by the
/// tree walker when installing an effect with a `TimeExpires` condition.
/// Ticked by [`tick_effect_timers`] each frame.
#[derive(Component, Debug, Clone)]
pub struct EffectTimers {
    /// Active timers: `(remaining_seconds, original_duration_seconds)`.
    pub timers: Vec<(OrderedFloat<f32>, OrderedFloat<f32>)>,
}
