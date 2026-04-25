//! Burnout — System 1: per-tick breaker heat-gauge update.

use bevy::prelude::*;

use super::config::BurnoutConfig;
use crate::prelude::*;

// ── BurnoutHeat ─────────────────────────────────────────────────────────────

/// Per-breaker heat gauge. Fills while the breaker moves, drains while still,
/// and tracks how long the breaker has been still. When heat reaches 1.0 the
/// breaker arms a mega-bump (`mega_bump_charged = true`) — the next
/// `BumpPerformed` consumes the charge.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct BurnoutHeat {
    /// Current heat — `[0.0, 1.0]`, clamped by the update system.
    pub heat:              f32,
    /// Seconds accumulated while stationary — reset to 0.0 on any moving
    /// tick. When this crosses `config.still_threshold`, the instant-drain +
    /// speed-boost fires.
    pub still_timer:       f32,
    /// `true` once `heat == 1.0`, consumed on the next bump.
    pub mega_bump_charged: bool,
}

// ── BurnoutSpeedBoost ───────────────────────────────────────────────────────

/// Per-breaker component installed by the still-threshold drain. Decrements
/// each tick; removed when `remaining` reaches 0.0.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct BurnoutSpeedBoost {
    /// Seconds remaining on this boost — counted down by
    /// `burnout_tick_speed_boost`.
    pub(crate) remaining: f32,
}

// ── System 1 — burnout_update_heat ──────────────────────────────────────────

/// Per-tick breaker heat-gauge update. Fills while moving, drains while still,
/// clamps to `[0.0, 1.0]`, arms `mega_bump_charged` on full, tracks the
/// still-timer, and on `still_threshold` crossing fires an instant drain +
/// installs `BurnoutSpeedBoost` + dispatches a shockwave.
///
/// Harness-safe: early-returns when `BurnoutConfig` is absent.
///
/// Lazy-insert: breakers without `BurnoutHeat` get `BurnoutHeat::default()`
/// inserted via `Commands` on their first encounter; the component becomes
/// visible on the next `FixedUpdate` tick.
pub(crate) fn burnout_update_heat(
    time: Res<Time<Fixed>>,
    config: Option<Res<BurnoutConfig>>,
    mut breakers: Query<(Entity, &Velocity2D, Option<&mut BurnoutHeat>), With<Breaker>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        return;
    };
    let dt = time.delta_secs();
    for (entity, velocity, heat_opt) in &mut breakers {
        let Some(mut heat) = heat_opt else {
            commands.entity(entity).insert(BurnoutHeat::default());
            continue;
        };
        let speed_sq = velocity.0.length_squared();
        if speed_sq > f32::EPSILON {
            // Moving branch — fill heat, reset still timer.
            if config.fill_duration > f32::EPSILON {
                heat.heat = (heat.heat + dt / config.fill_duration).min(1.0);
            }
            heat.still_timer = 0.0;
            if heat.heat >= 1.0 - f32::EPSILON {
                heat.mega_bump_charged = true;
            }
        } else {
            // Stationary branch — drain heat, advance still timer, check threshold.
            if config.drain_duration > f32::EPSILON {
                heat.heat = (heat.heat - dt / config.drain_duration).max(0.0);
            }
            heat.still_timer += dt;
            // Still-threshold fire requires heat > 0: draining an already-empty
            // gauge is not a "burnout" event — there is nothing to convert into
            // a speed boost.
            if heat.heat > 0.0 && heat.still_timer >= config.still_threshold {
                heat.heat = 0.0;
                heat.still_timer = 0.0;
                heat.mega_bump_charged = false;
                commands.entity(entity).insert(BurnoutSpeedBoost {
                    remaining: config.speed_boost_duration,
                });
            }
        }
    }
}
