//! System to release locked cells when all adjacent cells are destroyed.

use bevy::{ecs::entity::Entities, prelude::*};

use crate::{
    cells::{behaviors::locked::components::Unlocked, components::Cell},
    prelude::*,
};

type LockedCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Locks), (With<LockCell>, With<Locked>, Without<Unlocked>)>;

/// Removes [`Locked`] marker when all adjacent cells are destroyed.
///
/// Listens for [`Destroyed<Cell>`] messages and checks each locked cell's
/// [`Locks`] list. An adjacent is considered "gone" if it has been marked
/// [`Dead`] (same tick as the `Destroyed<Cell>` emission) or has been
/// despawned in a prior tick. When every adjacent is gone, [`Locked`] is
/// removed and [`Unlocked`] is inserted, allowing the cell to take damage.
pub(crate) fn check_lock_release(
    mut reader: MessageReader<Destroyed<Cell>>,
    query: LockedCellQuery,
    dead_query: Query<(), With<Dead>>,
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
            // Only scan when something was actually destroyed this tick.
            let all_gone = locks
                .0
                .iter()
                .all(|adj| !all_entities.contains(*adj) || dead_query.contains(*adj));
            if all_gone {
                commands.entity(entity).remove::<Locked>().insert(Unlocked);
            }
        }
    }
}
