//! Messages sent by the breaker domain.

use bevy::prelude::*;

/// Grade of a bump timing relative to bolt contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpGrade {
    /// Bump pressed before the perfect zone.
    Early,
    /// Bump timed within the perfect window.
    Perfect,
    /// Bump pressed after the bolt hit.
    Late,
}

/// Sent when the breaker performs a bump.
///
/// Consumed by audio, chips (overclock triggers), behaviors (bridges),
/// and UI.
#[derive(Message, Clone, Debug)]
pub struct BumpPerformed {
    /// The timing grade of the bump.
    pub grade: BumpGrade,
    /// The bolt entity involved in this bump, if known.
    pub bolt: Option<Entity>,
}

/// Sent when a forward bump window expires without bolt contact.
///
/// Consumed by UI for "WHIFF" feedback text.
#[derive(Message, Clone, Debug)]
pub struct BumpWhiffed;

/// Sent by `spawn_breaker` after the breaker entity is spawned or confirmed to exist.
///
/// Consumed by the spawn coordinator in the node subdomain.
#[derive(Message, Clone, Debug)]
pub struct BreakerSpawned;

/// Sent when the breaker collides with a cell.
///
/// Consumed by `bridge_cell_impact` and `bridge_breaker_impacted` in the effect domain.
#[derive(Message, Clone, Debug)]
pub(crate) struct BreakerImpactCell {
    /// The breaker entity that collided with the cell.
    pub breaker: Entity,
    /// The cell entity that was hit.
    pub cell: Entity,
}

/// Sent when the breaker collides with a wall.
///
/// Consumed by `bridge_wall_impact` and `bridge_breaker_impacted` in the effect domain.
#[derive(Message, Clone, Debug)]
pub(crate) struct BreakerImpactWall {
    /// The breaker entity that collided with the wall.
    pub breaker: Entity,
    /// The wall entity that was hit.
    pub wall: Entity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_grade_exhaustive_match() {
        for grade in [BumpGrade::Early, BumpGrade::Perfect, BumpGrade::Late] {
            match grade {
                BumpGrade::Early | BumpGrade::Perfect | BumpGrade::Late => {}
            }
        }
    }

    #[test]
    fn bump_performed_debug_format() {
        let msg = BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
        };
        assert!(format!("{msg:?}").contains("Perfect"));
    }

    #[test]
    fn bump_whiffed_debug_format() {
        let msg = BumpWhiffed;
        assert!(format!("{msg:?}").contains("BumpWhiffed"));
    }

    #[test]
    fn breaker_spawned_debug_format() {
        let msg = BreakerSpawned;
        assert!(format!("{msg:?}").contains("BreakerSpawned"));
    }
}
