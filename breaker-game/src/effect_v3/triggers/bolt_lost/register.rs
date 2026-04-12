//! Registration for bolt lost trigger bridge.

use bevy::prelude::*;

use super::bridges;
use crate::effect_v3::EffectV3Systems;

/// Registers the bolt lost trigger bridge system in `EffectV3Systems::Bridge`.
pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        bridges::on_bolt_lost_occurred.in_set(EffectV3Systems::Bridge),
    );
}
