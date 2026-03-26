//! System to release locked cells when all adjacent cells are destroyed.

use bevy::{ecs::entity::Entities, prelude::*};

use crate::cells::{
    components::{LockAdjacents, Locked},
    messages::CellDestroyedAt,
};

/// Removes [`Locked`] marker when all adjacent cells are destroyed.
///
/// Listens for [`CellDestroyedAt`] messages and checks each locked cell's
/// [`LockAdjacents`] list. If every entity in the adjacents list has been
/// destroyed (no longer exists in the world), the [`Locked`] component is
/// removed, allowing the cell to take damage.
pub(crate) fn check_lock_release(
    mut reader: MessageReader<CellDestroyedAt>,
    query: Query<(Entity, &LockAdjacents), With<Locked>>,
    mut commands: Commands,
    all_entities: &Entities,
) {
    // Drain destroyed messages and count them so they don't accumulate.
    let destroyed_count = reader.read().count();

    for (entity, adjacents) in &query {
        if adjacents.0.is_empty() {
            // Empty adjacents list means the cell should always unlock.
            commands.entity(entity).remove::<Locked>();
        } else if destroyed_count > 0 {
            // Only scan entity existence when something was actually destroyed.
            let all_gone = adjacents.0.iter().all(|adj| !all_entities.contains(*adj));
            if all_gone {
                commands.entity(entity).remove::<Locked>();
            }
        }
    }
}
