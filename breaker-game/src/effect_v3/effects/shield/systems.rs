//! Shield systems — tick duration countdown.

use bevy::prelude::*;

use super::components::{ShieldDuration, ShieldWall};

/// Decrements shield duration each frame and despawns expired shields.
pub fn tick_shield_duration(
    mut query: Query<(Entity, &mut ShieldDuration), With<ShieldWall>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut duration) in &mut query {
        duration.0 -= dt;
        if duration.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
