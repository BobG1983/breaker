//! Burnout — System 4: decrement `BurnoutSpeedBoost.remaining` and remove
//! the component on expiry.

use bevy::prelude::*;

use super::{config::BurnoutConfig, update_heat::BurnoutSpeedBoost};

// ── System 4 — burnout_tick_speed_boost ─────────────────────────────────────

/// Decrements `BurnoutSpeedBoost.remaining` by `delta_secs` each tick, and
/// removes the component when `remaining` reaches `0.0`.
///
/// Harness-safe: early-returns when `BurnoutConfig` is absent.
pub(crate) fn burnout_tick_speed_boost(
    time: Res<Time<Fixed>>,
    config: Option<Res<BurnoutConfig>>,
    mut boosts: Query<(Entity, &mut BurnoutSpeedBoost)>,
    mut commands: Commands,
) {
    if config.is_none() {
        return;
    }
    let dt = time.delta_secs();
    for (entity, mut boost) in &mut boosts {
        boost.remaining -= dt;
        if boost.remaining <= 0.0
            && let Ok(mut e) = commands.get_entity(entity)
        {
            e.remove::<BurnoutSpeedBoost>();
        }
    }
}
