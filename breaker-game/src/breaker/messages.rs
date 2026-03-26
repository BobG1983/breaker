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

/// Sent when the breaker should be destroyed. Entity is still alive.
///
/// Stub for two-phase destruction. Cleanup system reads this but does nothing
/// if no messages arrive.
#[derive(Message, Clone, Debug)]
pub(crate) struct RequestBreakerDestroyed {
    /// The breaker entity to be destroyed.
    pub breaker: Entity,
}

/// Sent after extracting entity data from the still-alive breaker.
///
/// Stub for two-phase destruction.
#[derive(Message, Clone, Debug)]
pub(crate) struct BreakerDestroyedAt {
    /// World-space position of the destroyed breaker.
    pub position: Vec2,
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

    // =========================================================================
    // C7 Wave 2a: Two-Phase Destruction stub types (behavior 36)
    // =========================================================================

    #[test]
    fn request_breaker_destroyed_debug_format() {
        let msg = RequestBreakerDestroyed {
            breaker: Entity::PLACEHOLDER,
        };
        assert!(format!("{msg:?}").contains("RequestBreakerDestroyed"));
    }

    #[test]
    fn breaker_destroyed_at_debug_format() {
        let msg = BreakerDestroyedAt {
            position: Vec2::new(0.0, -300.0),
        };
        let debug = format!("{msg:?}");
        assert!(debug.contains("BreakerDestroyedAt"));
    }

    #[test]
    fn breaker_spawned_debug_format() {
        let msg = BreakerSpawned;
        assert!(format!("{msg:?}").contains("BreakerSpawned"));
    }
}
