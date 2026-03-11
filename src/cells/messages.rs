//! Messages sent by the cells domain.

use bevy::prelude::*;

/// Sent when a cell is destroyed.
///
/// Consumed by run (progress tracking), upgrades (overclock triggers), and audio.
#[derive(Message, Clone, Debug)]
pub struct CellDestroyed {
    /// The entity that was destroyed.
    pub entity: Entity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_destroyed_debug_format() {
        let msg = CellDestroyed {
            entity: Entity::PLACEHOLDER,
        };
        assert!(format!("{msg:?}").contains("CellDestroyed"));
    }
}
