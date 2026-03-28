//! Bridge system for the `bumped` trigger.
use bevy::prelude::*;

use crate::effect::core::*;

fn bridge_bumped(
    mut _query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut _commands: Commands,
) {
    // Placeholder — message reading wired in Wave 8
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, bridge_bumped);
}
