//! Components for the time trigger category.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::prelude::SourceId;

/// Collection of active effect duration timers on an entity.
///
/// Each entry is a `(remaining_time, original_duration, source)` tuple. Added
/// by the tree walker when installing an effect with a `TimeExpires` condition.
/// Ticked by [`tick_effect_timers`] each frame. The `source` is the
/// [`SourceId`] of the Until entry that armed this timer; used by the bridge to
/// filter which `BoundEffects`/`StagedEffects` entries to walk on expiry.
#[derive(Component, Debug, Clone)]
pub struct EffectTimers {
    /// Active timers: `(remaining_seconds, original_duration_seconds, source)`.
    pub timers: Vec<(OrderedFloat<f32>, OrderedFloat<f32>, SourceId)>,
}
