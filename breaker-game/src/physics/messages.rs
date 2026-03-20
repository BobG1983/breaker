//! Messages sent by the physics domain.

use bevy::prelude::*;

/// Sent when the bolt collides with the breaker.
///
/// Consumed by breaker (`grade_bump`).
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltHitBreaker;

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn messages_debug_format() {
        let a = BoltHitBreaker;
        assert!(format!("{a:?}").contains("BoltHitBreaker"));

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
    }
}
