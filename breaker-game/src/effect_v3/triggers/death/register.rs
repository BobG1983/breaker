//! Registration for death trigger bridges.

use bevy::prelude::*;

use super::bridges;
use crate::effect_v3::EffectV3Systems;

/// Registers all death trigger bridge systems in `EffectV3Systems::Bridge`.
pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridges::on_cell_destroyed,
            bridges::on_bolt_destroyed,
            bridges::on_wall_destroyed,
            bridges::on_breaker_destroyed,
        )
            .in_set(EffectV3Systems::Bridge),
    );
}
