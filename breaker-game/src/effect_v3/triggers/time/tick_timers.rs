//! Effect timer tick system.
//!
//! Decrements all active effect timers each frame and sends
//! [`EffectTimerExpired`] when a timer reaches zero.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{components::EffectTimers, messages::EffectTimerExpired};

/// Ticks all [`EffectTimers`] components, decrementing remaining time.
///
/// When an entry reaches zero, sends [`EffectTimerExpired`] with the entity
/// and original duration, then removes the entry. If all entries are removed,
/// removes the [`EffectTimers`] component from the entity.
pub fn tick_effect_timers(
    mut query: Query<(Entity, &mut EffectTimers)>,
    time: Res<Time>,
    mut writer: MessageWriter<EffectTimerExpired>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut timers) in &mut query {
        let mut expired = Vec::new();

        for (i, (remaining, original)) in timers.timers.iter_mut().enumerate() {
            *remaining = OrderedFloat(remaining.0 - dt);
            if remaining.0 <= 0.0 {
                expired.push((i, *original));
            }
        }

        // Remove expired entries in reverse order to preserve indices
        for &(i, original) in expired.iter().rev() {
            timers.timers.swap_remove(i);
            writer.write(EffectTimerExpired {
                entity,
                original_duration: original,
            });
        }

        // Clean up component if no timers remain
        if timers.timers.is_empty() {
            commands.entity(entity).remove::<EffectTimers>();
        }
    }
}
