//! Messages sent by the cells domain.

use bevy::prelude::*;

/// Sent when a cell is destroyed.
///
/// Consumed by run (progress tracking) and chips (overclock triggers).
#[derive(Message, Clone, Debug)]
pub(crate) struct CellDestroyed {
    /// Whether this cell counted toward node completion.
    pub was_required_to_clear: bool,
}

/// Sent by physics (`bolt_cell_collision`) and bolt/behaviors (shockwave) to
/// request damage application on a cell.
///
/// A "command" message — owned by the receiving domain (cells), written by
/// multiple senders. The `damage` field is pre-calculated by the sender
/// (includes `DamageBoost`). `source_bolt` is pass-through for VFX attachment.
#[derive(Message, Clone, Debug)]
pub(crate) struct DamageCell {
    /// The cell entity to damage.
    pub cell: Entity,
    /// Pre-calculated damage amount.
    pub damage: f32,
    /// The bolt entity that caused this damage (for VFX attachment).
    pub source_bolt: Entity,
    /// The chip name that originated this damage, for evolution attribution.
    pub source_chip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_destroyed_debug_format() {
        let msg = CellDestroyed {
            was_required_to_clear: true,
        };
        assert!(format!("{msg:?}").contains("CellDestroyed"));
    }

    #[test]
    fn damage_cell_debug_format() {
        let msg = DamageCell {
            cell: Entity::PLACEHOLDER,
            damage: 10.0,
            source_bolt: Entity::PLACEHOLDER,
            source_chip: None,
        };
        assert!(format!("{msg:?}").contains("DamageCell"));
    }
}
