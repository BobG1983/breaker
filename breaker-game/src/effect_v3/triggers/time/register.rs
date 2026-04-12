//! Registration for time trigger bridge and tick system.

use bevy::prelude::*;

use super::{bridges, messages::EffectTimerExpired, tick_timers};
use crate::effect_v3::EffectV3Systems;

/// Registers the time trigger bridge, `tick_timers` system, and related types.
pub fn register(app: &mut App) {
    app.add_message::<EffectTimerExpired>();

    app.add_systems(
        FixedUpdate,
        (
            tick_timers::tick_effect_timers.in_set(EffectV3Systems::Tick),
            bridges::on_time_expires.in_set(EffectV3Systems::Bridge),
        ),
    );
}
