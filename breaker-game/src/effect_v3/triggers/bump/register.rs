//! Registration for bump trigger bridges.

use bevy::prelude::*;

use super::bridges;
use crate::effect_v3::EffectV3Systems;

/// Registers all bump trigger bridge systems in `EffectV3Systems::Bridge`.
pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridges::on_bumped,
            bridges::on_perfect_bumped,
            bridges::on_early_bumped,
            bridges::on_late_bumped,
            bridges::on_bump_occurred,
            bridges::on_perfect_bump_occurred,
            bridges::on_early_bump_occurred,
            bridges::on_late_bump_occurred,
            bridges::on_bump_whiff_occurred,
            bridges::on_no_bump_occurred,
        )
            .in_set(EffectV3Systems::Bridge),
    );
}
