//! Anchor systems — tick lock/unlock.

use bevy::prelude::*;

use super::components::{AnchorPlanted, AnchorTimer};

/// Decrements anchor timer and plants the anchor when it reaches zero.
pub fn tick_anchor(
    mut query: Query<(Entity, &mut AnchorTimer), Without<AnchorPlanted>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut timer) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            commands.entity(entity).insert(AnchorPlanted);
        }
    }
}
