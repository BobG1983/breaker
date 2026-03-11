//! Messages sent by the breaker domain.

use bevy::prelude::*;

/// Grade of a bump timing relative to bolt contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpGrade {
    /// No bump attempted.
    None,
    /// Bump pressed too early.
    Early,
    /// Bump timed within the perfect window.
    Perfect,
    /// Bump pressed too late.
    Late,
}

/// Sent when the breaker performs a bump.
///
/// Consumed by audio, upgrades (overclock triggers), and UI.
#[derive(Message, Clone, Debug)]
pub struct BumpPerformed {
    /// The timing grade of the bump.
    pub grade: BumpGrade,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_grade_has_four_variants() {
        let grades = [
            BumpGrade::None,
            BumpGrade::Early,
            BumpGrade::Perfect,
            BumpGrade::Late,
        ];
        assert_eq!(grades.len(), 4);
    }

    #[test]
    fn bump_performed_debug_format() {
        let msg = BumpPerformed {
            grade: BumpGrade::Perfect,
        };
        assert!(format!("{msg:?}").contains("Perfect"));
    }
}
