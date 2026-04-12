//! Registration for impact trigger bridges.

use bevy::prelude::*;

use super::bridges;
use crate::effect_v3::EffectV3Systems;

/// Registers all impact trigger bridge systems in `EffectV3Systems::Bridge`.
pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (bridges::on_impacted, bridges::on_impact_occurred).in_set(EffectV3Systems::Bridge),
    );
}
