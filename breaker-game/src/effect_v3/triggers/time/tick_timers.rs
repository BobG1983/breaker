//! Effect timer tick system.
//!
//! Decrements all active effect timers each frame and sends
//! [`EffectTimerExpired`] when a timer reaches zero.

use bevy::prelude::*;

use super::{components::EffectTimers, messages::EffectTimerExpired};

/// Ticks all [`EffectTimers`] components, decrementing remaining time.
///
/// When an entry reaches zero, sends [`EffectTimerExpired`] with the entity
/// and original duration, then removes the entry. If all entries are removed,
/// removes the [`EffectTimers`] component from the entity.
pub fn tick_effect_timers() {
    todo!()
}
