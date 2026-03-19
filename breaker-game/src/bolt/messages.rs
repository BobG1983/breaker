//! Bolt domain messages.

use bevy::prelude::*;

/// Sent by the breaker behavior system to spawn an additional bolt.
///
/// Consumed by `spawn_additional_bolt` in the bolt domain.
#[derive(Message, Clone, Debug)]
pub struct SpawnAdditionalBolt;

/// Sent by `spawn_bolt` after the bolt entity is spawned.
///
/// Consumed by the spawn coordinator in the node subdomain.
#[derive(Message, Clone, Debug)]
pub struct BoltSpawned;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_debug_format() {
        let msg = SpawnAdditionalBolt;
        assert!(format!("{msg:?}").contains("SpawnAdditionalBolt"));

        let msg = BoltSpawned;
        assert!(format!("{msg:?}").contains("BoltSpawned"));
    }
}
