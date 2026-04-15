//! Second-wind systems — despawn on first reflection.

use bevy::prelude::*;

use super::super::components::SecondWindWall;
use crate::bolt::messages::BoltImpactWall;

/// Despawns a `SecondWindWall` entity on the first `BoltImpactWall` targeting it.
///
/// Reads `BoltImpactWall` messages and despawns any message target that matches
/// `With<SecondWindWall>`. Second messages targeting an already-despawned wall
/// (same frame or subsequent frames) are silent no-ops.
pub(crate) fn despawn_on_first_reflection(
    mut reader: MessageReader<BoltImpactWall>,
    query: Query<Entity, With<SecondWindWall>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if query.get(msg.wall).is_ok() {
            commands.entity(msg.wall).despawn();
        }
    }
}
