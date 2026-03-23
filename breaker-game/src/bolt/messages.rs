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

/// Sent when the bolt collides with the breaker.
///
/// Consumed by breaker (`grade_bump`).
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltHitBreaker {
    /// The bolt entity that hit the breaker.
    pub bolt: Entity,
}

/// Sent when the bolt collides with a cell.
///
/// Consumed by chips, cells, and audio.
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltHitCell {
    /// The cell entity that was hit.
    pub cell: Entity,
    /// The bolt entity that caused the hit.
    pub bolt: Entity,
}

/// Sent when the bolt falls below the breaker.
///
/// Consumed by the breaker plugin (applies penalty per breaker type).
#[derive(Message, Clone, Debug)]
pub struct BoltLost;

/// Sent when the bolt reflects off a wall.
///
/// Consumed by bolt/behaviors (overclock wall impact bridge).
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltHitWall {
    /// The bolt entity that hit the wall.
    pub bolt: Entity,
}

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

    #[test]
    fn bolt_hit_cell_carries_both_cell_and_bolt_fields() {
        let cell_entity = Entity::PLACEHOLDER;
        let bolt_entity = Entity::PLACEHOLDER;
        let msg = BoltHitCell {
            cell: cell_entity,
            bolt: bolt_entity,
        };
        assert_eq!(
            msg.cell, cell_entity,
            "BoltHitCell.cell should be accessible and match the entity passed in"
        );
        assert_eq!(
            msg.bolt, bolt_entity,
            "BoltHitCell.bolt should be accessible and match the entity passed in"
        );
    }

    #[test]
    fn collision_messages_debug_format() {
        let a = BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        };
        let a_fmt = format!("{a:?}");
        assert!(a_fmt.contains("BoltHitBreaker"));
        assert!(
            a_fmt.contains("bolt"),
            "BoltHitBreaker debug format should include 'bolt' field name"
        );

        let b = BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt: Entity::PLACEHOLDER,
        };
        let b_fmt = format!("{b:?}");
        assert!(b_fmt.contains("BoltHitCell"));
        assert!(
            b_fmt.contains("bolt"),
            "debug format should include 'bolt' field name"
        );

        let c = BoltLost;
        assert!(format!("{c:?}").contains("BoltLost"));

        let d = BoltHitWall {
            bolt: Entity::PLACEHOLDER,
        };
        let d_fmt = format!("{d:?}");
        assert!(d_fmt.contains("BoltHitWall"));
        assert!(
            d_fmt.contains("bolt"),
            "BoltHitWall debug format should include 'bolt' field name"
        );
    }
}
