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
/// Consumed by bolt (velocity multiplier), audio, chips (overclock
/// triggers), and UI.
#[derive(Message, Clone, Debug)]
pub struct BumpPerformed {
    /// The timing grade of the bump.
    pub grade: BumpGrade,
    /// Velocity multiplier for this bump grade.
    pub multiplier: f32,
    /// The bolt entity involved in this bump.
    pub bolt: Entity,
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
            multiplier: 1.5,
            bolt: Entity::PLACEHOLDER,
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
