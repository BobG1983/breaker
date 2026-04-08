//! System to release locked cells when all adjacent cells are destroyed.

use bevy::{ecs::entity::Entities, prelude::*};

use crate::cells::{
    behaviors::locked::components::{LockCell, Locked, Locks, Unlocked},
    messages::CellDestroyedAt,
};

type LockedCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Locks), (With<LockCell>, With<Locked>, Without<Unlocked>)>;

/// Removes [`Locked`] marker when all adjacent cells are destroyed.
///
/// Listens for [`CellDestroyedAt`] messages and checks each locked cell's
/// [`Locks`] list. If every entity in the list has been destroyed (no longer
/// exists in the world), the [`Locked`] component is removed and [`Unlocked`]
/// is inserted, allowing the cell to take damage.
pub(crate) fn check_lock_release(
    mut reader: MessageReader<CellDestroyedAt>,
    query: LockedCellQuery,
    mut commands: Commands,
    all_entities: &Entities,
) {
    // Drain destroyed messages and count them so they don't accumulate.
    let destroyed_count = reader.read().count();

    for (entity, locks) in &query {
        if locks.0.is_empty() {
            // Empty locks list means the cell should always unlock.
            commands.entity(entity).remove::<Locked>().insert(Unlocked);
        } else if destroyed_count > 0 {
            // Only scan entity existence when something was actually destroyed.
            let all_gone = locks.0.iter().all(|adj| !all_entities.contains(*adj));
            if all_gone {
                commands.entity(entity).remove::<Locked>().insert(Unlocked);
            }
        }
    }
}
