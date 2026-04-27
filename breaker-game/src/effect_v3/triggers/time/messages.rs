//! Messages for the time trigger category.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::prelude::SourceId;

/// Sent by [`tick_effect_timers`] when an entry in [`EffectTimers`] reaches zero.
///
/// Read by the `on_time_expires` bridge which dispatches
/// `Trigger::TimeExpires(original_duration)` on the entity (Self scope), but
/// ONLY against the bound/staged entries whose name matches `source` —
/// disambiguating concurrent same-duration Untils on the same entity.
#[derive(Message, Clone, Debug)]
pub struct EffectTimerExpired {
    /// The entity whose timer expired.
    pub entity:            Entity,
    /// The timer's original duration, used to construct the correct trigger variant.
    pub original_duration: OrderedFloat<f32>,
    /// The source that armed this timer entry.
    pub source:            SourceId,
}
