//! Bridge system for the `no_bump` trigger.
use bevy::prelude::*;

use crate::effect::core::*;

const fn bridge_no_bump(
    _query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    _commands: Commands,
) {
    // Placeholder — message reading wired in Wave 8
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, bridge_no_bump);
}
