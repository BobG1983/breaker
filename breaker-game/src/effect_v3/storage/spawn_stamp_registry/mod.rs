//! `SpawnStampRegistry` resource and its `Added<T>` watcher systems.

mod resource;
mod watchers;

pub use resource::SpawnStampRegistry;
pub(crate) use watchers::{
    stamp_spawned_bolts, stamp_spawned_breakers, stamp_spawned_cells, stamp_spawned_walls,
};
