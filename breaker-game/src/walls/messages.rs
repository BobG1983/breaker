//! Messages sent by the wall domain.

use bevy::prelude::*;

/// Sent by `spawn_walls` after all wall entities are spawned.
///
/// Consumed by the spawn coordinator in the node subdomain.
#[derive(Message, Clone, Debug)]
pub(crate) struct WallsSpawned;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walls_spawned_debug_format() {
        let msg = WallsSpawned;
        assert!(format!("{msg:?}").contains("WallsSpawned"));
    }
}
