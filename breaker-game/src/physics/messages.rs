//! Messages sent by the physics domain.

use bevy::prelude::*;

/// Sent when the bolt collides with the breaker.
///
/// Consumed by audio, chips, and UI.
#[derive(Message, Clone, Debug)]
pub struct BoltHitBreaker {
    /// The bolt entity involved in the collision.
    pub bolt: Entity,
}

/// Sent when the bolt collides with a cell.
///
/// Consumed by chips, cells, and audio.
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltHitCell {
    /// The cell entity that was hit.
    pub cell: Entity,
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
    fn messages_debug_format() {
        let a = BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        };
        assert!(format!("{a:?}").contains("BoltHitBreaker"));

        let b = BoltHitCell {
            cell: Entity::PLACEHOLDER,
        };
        assert!(format!("{b:?}").contains("BoltHitCell"));

        let c = BoltLost;
        assert!(format!("{c:?}").contains("BoltLost"));
    }
}
