//! Messages sent by the cells domain.

use bevy::prelude::*;

/// Sent by `handle_cell_hit` when a cell's HP reaches 0. The entity is still alive.
///
/// Consumed by `bridge_cell_death` (evaluates `OnDeath` `BoundEffects` while entity
/// is still alive) and `cleanup_cell` (despawns the entity).
#[derive(Message, Clone, Debug)]
pub(crate) struct RequestCellDestroyed {
    /// The cell entity to be destroyed.
    pub cell: Entity,
    /// World-space position of the destroyed cell.
    pub position: Vec2,
    /// Whether this cell had the `RequiredToClear` component.
    pub was_required_to_clear: bool,
}

/// Sent by `cleanup_cell` after extracting entity data from the still-alive cell.
///
/// Replaces `CellDestroyed` for all downstream consumers (run tracking, lock release, etc.).
#[derive(Message, Clone, Debug)]
pub(crate) struct CellDestroyedAt {
    /// World-space position of the destroyed cell.
    pub position: Vec2,
    /// Whether this cell counted toward node completion.
    pub was_required_to_clear: bool,
}

/// Sent when a cell collides with a wall.
///
/// Consumed by `bridge_wall_impact` and `bridge_cell_impacted` in the effect domain.
/// Relevant for future moving-cell mechanics.
#[derive(Message, Clone, Debug)]
pub(crate) struct CellImpactWall {
    /// The cell entity that collided with the wall.
    pub cell: Entity,
    /// The wall entity that was hit.
    pub wall: Entity,
}

/// Sent by physics (`bolt_cell_collision`) and bolt/behaviors (shockwave) to
/// request damage application on a cell.
///
/// A "command" message — owned by the receiving domain (cells), written by
/// multiple senders. The `damage` field is pre-calculated by the sender
/// (includes `EffectiveDamageMultiplier`). `source_bolt` is pass-through for VFX attachment.
#[derive(Message, Clone, Debug)]
pub(crate) struct DamageCell {
    /// The cell entity to damage.
    pub cell: Entity,
    /// Pre-calculated damage amount.
    pub damage: f32,
    // FUTURE: may be used for upcoming phases
    // /// The bolt entity that caused this damage (for VFX attachment), if any.
    // pub source_bolt: Option<Entity>,
    /// The chip name that originated this damage, for evolution attribution.
    pub source_chip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // C7 Wave 2a: Two-Phase Destruction cell message types
    // =========================================================================

    #[test]
    fn request_cell_destroyed_debug_format() {
        let msg = RequestCellDestroyed {
            cell: Entity::PLACEHOLDER,
            position: Vec2::ZERO,
            was_required_to_clear: false,
        };
        let debug = format!("{msg:?}");
        assert!(debug.contains("RequestCellDestroyed"));
        assert!(debug.contains("cell"));
    }

    #[test]
    fn cell_destroyed_at_debug_format() {
        let msg = CellDestroyedAt {
            position: Vec2::ZERO,
            was_required_to_clear: true,
        };
        let debug = format!("{msg:?}");
        assert!(debug.contains("CellDestroyedAt"));
        assert!(debug.contains("was_required_to_clear"));
    }

    #[test]
    fn cell_destroyed_at_non_required() {
        let msg = CellDestroyedAt {
            position: Vec2::ZERO,
            was_required_to_clear: false,
        };
        assert!(!msg.was_required_to_clear);
    }

    #[test]
    fn damage_cell_debug_format() {
        let msg = DamageCell {
            cell: Entity::PLACEHOLDER,
            damage: 10.0,
            source_chip: None,
        };
        assert!(format!("{msg:?}").contains("DamageCell"));
    }
}
