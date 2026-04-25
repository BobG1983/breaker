//! Burnout protocol — `BurnoutConfig` resource + `activate` setup.

use bevy::prelude::*;

use crate::mutators::protocols::definition::ProtocolTuning;

// ── BurnoutConfig ───────────────────────────────────────────────────────────

/// Per-run Burnout tuning extracted from `ProtocolTuning::Burnout` at
/// activation time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct BurnoutConfig {
    /// Seconds of continuous movement required to fill heat from 0.0 to 1.0.
    pub(crate) fill_duration:               f32,
    /// Seconds of continuous stillness required to drain heat from 1.0 to 0.0.
    pub(crate) drain_duration:              f32,
    /// Seconds of stillness before the instant-drain + speed-boost fires.
    pub(crate) still_threshold:             f32,
    /// Multiplier applied to `BoltBaseDamage` on a mega-bump cell impact.
    pub(crate) full_heat_damage_multiplier: f32,
    /// Seconds the `BurnoutSpeedBoost` component lasts after still-threshold.
    pub(crate) speed_boost_duration:        f32,
}

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts `BurnoutConfig` from `ProtocolTuning::Burnout`. Warns and no-ops
/// on a non-Burnout tuning variant, leaving any existing `BurnoutConfig`
/// intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Burnout {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
    } = *tuning
    else {
        warn!("burnout::activate called with non-Burnout tuning");
        return;
    };
    commands.insert_resource(BurnoutConfig {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
    });
}
