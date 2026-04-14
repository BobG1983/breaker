//! `Added<T>` watcher systems that install `SpawnStampRegistry` entries on
//! newly spawned entities.

mod stamp_spawned_bolts;
mod stamp_spawned_breakers;
mod stamp_spawned_cells;
mod stamp_spawned_walls;

pub(crate) use stamp_spawned_bolts::stamp_spawned_bolts;
pub(crate) use stamp_spawned_breakers::stamp_spawned_breakers;
pub(crate) use stamp_spawned_cells::stamp_spawned_cells;
pub(crate) use stamp_spawned_walls::stamp_spawned_walls;

#[cfg(test)]
mod tests;
