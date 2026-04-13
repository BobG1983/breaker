//! Bolt domain messages.

use bevy::prelude::*;

/// Sent by `spawn_bolt` after the bolt entity is spawned.
///
/// Consumed by the spawn coordinator in the node subdomain.
#[derive(Message, Clone, Debug)]
pub struct BoltSpawned;

/// Sent when the bolt collides with the breaker.
///
/// Consumed by breaker (`grade_bump`).
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltImpactBreaker {
    /// The bolt entity that hit the breaker.
    pub bolt:    Entity,
    /// The breaker entity that was hit.
    pub breaker: Entity,
}

/// Sent when the bolt collides with a cell.
///
/// Consumed by chips, cells, and audio.
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltImpactCell {
    /// The cell entity that was hit.
    pub cell: Entity,
    /// The bolt entity that caused the hit.
    pub bolt: Entity,
}

/// Sent when the bolt falls below the breaker.
///
/// Consumed by the breaker plugin (applies penalty per breaker type).
#[derive(Message, Clone, Debug)]
pub struct BoltLost {
    /// The bolt entity that was lost.
    pub bolt:    Entity,
    /// The breaker entity that lost the bolt.
    pub breaker: Entity,
}

/// Sent when the bolt reflects off a wall.
///
/// Consumed by bolt/behaviors (overclock wall impact bridge).
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltImpactWall {
    /// The bolt entity that hit the wall.
    pub bolt: Entity,
    /// The wall entity that was hit.
    pub wall: Entity,
}

/// Sent by `bolt_lost` when an extra bolt falls off screen. Entity is still alive.
///
/// Consumed by `bridge_bolt_death` (evaluates `OnDeath` `BoundEffects`) and
/// `cleanup_destroyed_bolts` (despawns the entity).
#[derive(Message, Clone, Debug)]
pub(crate) struct RequestBoltDestroyed {
    /// The bolt entity to be destroyed.
    pub bolt: Entity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_debug_format() {
        let msg = BoltSpawned;
        assert!(format!("{msg:?}").contains("BoltSpawned"));
    }

    #[test]
    fn bolt_hit_cell_carries_both_cell_and_bolt_fields() {
        let cell_entity = Entity::PLACEHOLDER;
        let bolt_entity = Entity::PLACEHOLDER;
        let msg = BoltImpactCell {
            cell: cell_entity,
            bolt: bolt_entity,
        };
        assert_eq!(
            msg.cell, cell_entity,
            "BoltImpactCell.cell should be accessible and match the entity passed in"
        );
        assert_eq!(
            msg.bolt, bolt_entity,
            "BoltImpactCell.bolt should be accessible and match the entity passed in"
        );
    }

    // =========================================================================
    // C7 Wave 2a: Two-Phase Destruction bolt message types
    // =========================================================================

    #[test]
    fn request_bolt_destroyed_debug_format() {
        let msg = RequestBoltDestroyed {
            bolt: Entity::PLACEHOLDER,
        };
        let debug = format!("{msg:?}");
        assert!(debug.contains("RequestBoltDestroyed"));
        assert!(debug.contains("bolt"));
    }

    #[test]
    fn collision_messages_debug_format() {
        let a = BoltImpactBreaker {
            bolt:    Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        };
        let a_fmt = format!("{a:?}");
        assert!(a_fmt.contains("BoltImpactBreaker"));
        assert!(
            a_fmt.contains("bolt"),
            "BoltImpactBreaker debug format should include 'bolt' field name"
        );

        let b = BoltImpactCell {
            cell: Entity::PLACEHOLDER,
            bolt: Entity::PLACEHOLDER,
        };
        let b_fmt = format!("{b:?}");
        assert!(b_fmt.contains("BoltImpactCell"));
        assert!(
            b_fmt.contains("bolt"),
            "debug format should include 'bolt' field name"
        );

        let c = BoltLost {
            bolt:    Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        };
        let c_fmt = format!("{c:?}");
        assert!(c_fmt.contains("BoltLost"));
        assert!(
            c_fmt.contains("bolt"),
            "BoltLost debug format should include 'bolt' field name"
        );
        assert!(
            c_fmt.contains("breaker"),
            "BoltLost debug format should include 'breaker' field name"
        );

        let d = BoltImpactWall {
            bolt: Entity::PLACEHOLDER,
            wall: Entity::PLACEHOLDER,
        };
        let d_fmt = format!("{d:?}");
        assert!(d_fmt.contains("BoltImpactWall"));
        assert!(
            d_fmt.contains("bolt"),
            "BoltImpactWall debug format should include 'bolt' field name"
        );
    }

    #[test]
    fn bolt_lost_carries_bolt_and_breaker_entity_fields() {
        let mut world = World::new();
        let bolt_entity = world.spawn_empty().id();
        let breaker_entity = world.spawn_empty().id();
        let msg = BoltLost {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        };
        assert_eq!(
            msg.bolt, bolt_entity,
            "BoltLost.bolt should be accessible and match the entity passed in"
        );
        assert_eq!(
            msg.breaker, breaker_entity,
            "BoltLost.breaker should be accessible and match the entity passed in"
        );
    }

    #[test]
    fn bolt_lost_placeholder_entities_are_accessible() {
        let msg = BoltLost {
            bolt:    Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        };
        assert_eq!(
            msg.bolt,
            Entity::PLACEHOLDER,
            "BoltLost.bolt should equal PLACEHOLDER"
        );
        assert_eq!(
            msg.breaker,
            Entity::PLACEHOLDER,
            "BoltLost.breaker should equal PLACEHOLDER"
        );
    }
}
