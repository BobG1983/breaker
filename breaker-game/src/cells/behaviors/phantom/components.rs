//! Phantom behavior components.
//!
//! Marker, phase, timer, and config components for phantom cells. Behavior
//! lives in the sibling `systems/` module: `tick_phantom_phase` cycles cells
//! through Solid, Telegraph, and Ghost phases, zeroing collision layers
//! during Ghost.

use bevy::prelude::*;
use serde::Deserialize;

/// Permanent marker identifying a cell as a phantom-type cell.
///
/// Never removed. Inserted alongside `PhantomPhase`, `PhantomTimer`, and
/// `PhantomConfig` when `CellBehavior::Phantom` is resolved at spawn time.
/// Lets systems query "is this cell phantom?" via `With<PhantomCell>` without
/// fetching phase/timer data.
#[derive(Component, Debug)]
pub struct PhantomCell;

/// Current phase of the phantom cycle.
///
/// Cycles `Solid -> Telegraph -> Ghost -> Solid`. During `Ghost`, collision
/// layers are zeroed so bolts pass through. `Telegraph` is a warning phase
/// (still collidable).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize)]
pub enum PhantomPhase {
    /// Normal collidable state. Default starting phase.
    #[default]
    Solid,
    /// Warning state before becoming intangible. Still collidable.
    Telegraph,
    /// Intangible state. Collision layers are zeroed.
    Ghost,
}

impl PhantomPhase {
    /// Returns the next phase in the cycle.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Solid => Self::Telegraph,
            Self::Telegraph => Self::Ghost,
            Self::Ghost => Self::Solid,
        }
    }
}

/// Remaining seconds in the current phantom phase.
#[derive(Component, Debug, Clone, Copy)]
pub struct PhantomTimer(pub f32);

/// Per-cell timing configuration for the phantom cycle.
///
/// - `cycle_secs`: duration of the Ghost phase (also the base for computing
///   Solid duration as `cycle_secs - telegraph_secs`).
/// - `telegraph_secs`: duration of the Telegraph (warning) phase.
#[derive(Component, Debug, Clone, Copy)]
pub struct PhantomConfig {
    /// Total cycle time base. Ghost phase lasts this long.
    pub cycle_secs:     f32,
    /// Telegraph (warning) phase duration.
    pub telegraph_secs: f32,
}

impl PhantomConfig {
    /// Returns the duration for the given phase.
    ///
    /// - Solid: `cycle_secs - telegraph_secs`
    /// - Telegraph: `telegraph_secs`
    /// - Ghost: `cycle_secs`
    #[must_use]
    pub fn duration_for(&self, phase: PhantomPhase) -> f32 {
        match phase {
            PhantomPhase::Solid => self.cycle_secs - self.telegraph_secs,
            PhantomPhase::Telegraph => self.telegraph_secs,
            PhantomPhase::Ghost => self.cycle_secs,
        }
    }

    /// Validates that the config values are well-formed.
    ///
    /// Rules:
    /// - `cycle_secs` must be positive and finite.
    /// - `telegraph_secs` must be non-negative, finite, and strictly less than `cycle_secs`.
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid field found.
    pub fn validate(&self) -> Result<(), String> {
        crate::shared::validation::positive_finite_f32("cycle_secs", self.cycle_secs)?;
        if self.telegraph_secs < 0.0 || !self.telegraph_secs.is_finite() {
            return Err(format!(
                "telegraph_secs must be non-negative and finite, got {}",
                self.telegraph_secs
            ));
        }
        if self.telegraph_secs >= self.cycle_secs {
            return Err(format!(
                "telegraph_secs must be strictly less than cycle_secs, got telegraph_secs={} >= cycle_secs={}",
                self.telegraph_secs, self.cycle_secs
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Group A — Component Definitions and Defaults ──────────────

    // Behavior 1: PhantomPhase default is Solid
    #[test]
    fn phantom_phase_default_is_solid() {
        assert_eq!(
            PhantomPhase::default(),
            PhantomPhase::Solid,
            "PhantomPhase::default() should be Solid"
        );
    }

    // Behavior 1 edge: all three variants are distinct
    #[test]
    fn phantom_phase_variants_are_distinct() {
        let solid = PhantomPhase::Solid;
        let telegraph = PhantomPhase::Telegraph;
        let ghost = PhantomPhase::Ghost;

        assert_ne!(solid, telegraph, "Solid and Telegraph should be distinct");
        assert_ne!(telegraph, ghost, "Telegraph and Ghost should be distinct");
        assert_ne!(solid, ghost, "Solid and Ghost should be distinct");
    }

    // Behavior 2: PhantomPhase::Solid.next() == Telegraph
    #[test]
    fn phantom_phase_solid_next_is_telegraph() {
        assert_eq!(
            PhantomPhase::Solid.next(),
            PhantomPhase::Telegraph,
            "Solid.next() should be Telegraph"
        );
    }

    // Behavior 3: PhantomPhase::Telegraph.next() == Ghost
    #[test]
    fn phantom_phase_telegraph_next_is_ghost() {
        assert_eq!(
            PhantomPhase::Telegraph.next(),
            PhantomPhase::Ghost,
            "Telegraph.next() should be Ghost"
        );
    }

    // Behavior 4: PhantomPhase::Ghost.next() == Solid
    #[test]
    fn phantom_phase_ghost_next_is_solid() {
        assert_eq!(
            PhantomPhase::Ghost.next(),
            PhantomPhase::Solid,
            "Ghost.next() should be Solid"
        );
    }

    // Behavior 5: duration_for(Solid) == cycle_secs - telegraph_secs
    #[test]
    fn config_duration_for_solid_phase() {
        let config = PhantomConfig {
            cycle_secs:     3.0,
            telegraph_secs: 0.5,
        };
        let duration = config.duration_for(PhantomPhase::Solid);
        assert!(
            (duration - 2.5).abs() < f32::EPSILON,
            "Solid duration should be cycle_secs - telegraph_secs = 2.5, got {duration}"
        );
    }

    // Behavior 5 edge: zero telegraph means Solid = full cycle
    #[test]
    fn config_duration_for_solid_with_zero_telegraph() {
        let config = PhantomConfig {
            cycle_secs:     1.0,
            telegraph_secs: 0.0,
        };
        let duration = config.duration_for(PhantomPhase::Solid);
        assert!(
            (duration - 1.0).abs() < f32::EPSILON,
            "Solid duration with zero telegraph should be 1.0, got {duration}"
        );
    }

    // Behavior 6: duration_for(Telegraph) == telegraph_secs
    #[test]
    fn config_duration_for_telegraph_phase() {
        let config = PhantomConfig {
            cycle_secs:     3.0,
            telegraph_secs: 0.5,
        };
        let duration = config.duration_for(PhantomPhase::Telegraph);
        assert!(
            (duration - 0.5).abs() < f32::EPSILON,
            "Telegraph duration should be telegraph_secs = 0.5, got {duration}"
        );
    }

    // Behavior 6 edge: zero telegraph
    #[test]
    fn config_duration_for_telegraph_with_zero_telegraph() {
        let config = PhantomConfig {
            cycle_secs:     3.0,
            telegraph_secs: 0.0,
        };
        let duration = config.duration_for(PhantomPhase::Telegraph);
        assert!(
            (duration - 0.0).abs() < f32::EPSILON,
            "Telegraph duration with telegraph_secs=0.0 should be 0.0, got {duration}"
        );
    }

    // Behavior 7: duration_for(Ghost) == cycle_secs
    #[test]
    fn config_duration_for_ghost_phase() {
        let config = PhantomConfig {
            cycle_secs:     3.0,
            telegraph_secs: 0.5,
        };
        let duration = config.duration_for(PhantomPhase::Ghost);
        assert!(
            (duration - 3.0).abs() < f32::EPSILON,
            "Ghost duration should be cycle_secs = 3.0, got {duration}"
        );
    }

    // Behavior 7 edge: small cycle
    #[test]
    fn config_duration_for_ghost_with_small_cycle() {
        let config = PhantomConfig {
            cycle_secs:     0.1,
            telegraph_secs: 0.05,
        };
        let duration = config.duration_for(PhantomPhase::Ghost);
        assert!(
            (duration - 0.1).abs() < f32::EPSILON,
            "Ghost duration should be cycle_secs = 0.1, got {duration}"
        );
    }
}
