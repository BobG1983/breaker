//! Invariant kinds the runner can check during a scenario run.

use serde::Deserialize;

/// Invariant kinds the runner can check during a scenario run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum InvariantKind {
    /// Bolt position stays within playfield bounds.
    BoltInBounds,
    /// Bolt speed stays within configured min/max bounds.
    BoltSpeedInRange,
    /// Bolt count stays within `invariant_params.max_bolt_count`.
    BoltCountReasonable,
    /// Breaker position stays within playfield bounds.
    BreakerInBounds,
    /// No unexpected entity accumulation over time.
    NoEntityLeaks,
    /// No NaN values in transform/velocity components.
    NoNaN,
    /// Node timer never goes negative.
    TimerNonNegative,
    /// Breaker state machine only takes valid transitions.
    ValidStateTransitions,
    /// Breaker movement state machine only takes legal transitions.
    ValidBreakerState,
    /// Node timer decreases monotonically (never increases mid-node).
    TimerMonotonicallyDecreasing,
    /// Breaker x position stays within playfield bounds minus half-width.
    BreakerPositionClamped,
    /// Physics entities do not move while game is paused.
    PhysicsFrozenDuringPause,
    /// No duplicate chip names in a single offering.
    OfferingNoDuplicates,
    /// Maxed chips never appear in offerings.
    MaxedChipNeverOffered,
    /// Held chip stacks never exceed `max_stacks` in [`ChipInventory`].
    ChipStacksConsistent,
    /// [`RunStats`] counters never decrease during a run.
    RunStatsMonotonic,
    /// Expected chip name not found in offerings during chip select.
    ChipOfferExpected,
    /// At most 1 [`SecondWindWall`](breaker::effect::effects::second_wind::SecondWindWall)
    /// entity should exist at any frame.
    SecondWindWallAtMostOne,
    /// [`ShieldActive`](breaker::effect::effects::shield::ShieldActive) must never
    /// have `charges == 0` -- zero-charge shields should be removed immediately.
    ShieldChargesConsistent,
    /// [`PulseRing`](breaker::effect::effects::pulse::PulseRing) entity count stays
    /// within `invariant_params.max_pulse_ring_count`.
    PulseRingAccumulation,
    /// [`EffectiveSpeedMultiplier`](breaker::effect::EffectiveSpeedMultiplier) must
    /// equal the product of all [`ActiveSpeedBoosts`](breaker::effect::effects::speed_boost::ActiveSpeedBoosts)
    /// entries within floating-point epsilon.
    EffectiveSpeedConsistent,
    /// Chain lightning chain + arc entity count stays within `invariant_params.max_chain_arc_count`.
    ChainArcCountReasonable,
}

impl InvariantKind {
    /// All variants of [`InvariantKind`], for exhaustive iteration.
    ///
    /// Keep in sync when adding new variants -- the
    /// `all_variants_covered_by_invariant_kind_all` test enforces this via
    /// the `fail_reason()` exhaustive match.
    pub const ALL: &[Self] = &[
        Self::BoltInBounds,
        Self::BoltSpeedInRange,
        Self::BoltCountReasonable,
        Self::BreakerInBounds,
        Self::NoEntityLeaks,
        Self::NoNaN,
        Self::TimerNonNegative,
        Self::ValidStateTransitions,
        Self::ValidBreakerState,
        Self::TimerMonotonicallyDecreasing,
        Self::BreakerPositionClamped,
        Self::PhysicsFrozenDuringPause,
        Self::OfferingNoDuplicates,
        Self::MaxedChipNeverOffered,
        Self::ChipStacksConsistent,
        Self::RunStatsMonotonic,
        Self::ChipOfferExpected,
        Self::SecondWindWallAtMostOne,
        Self::ShieldChargesConsistent,
        Self::PulseRingAccumulation,
        Self::EffectiveSpeedConsistent,
        Self::ChainArcCountReasonable,
    ];

    /// Standard human-readable fail reason for this invariant violation.
    ///
    /// Used by [`crate::verdict::ScenarioVerdict`] to build structured failure reasons
    /// without string construction at evaluation time.
    #[must_use]
    pub const fn fail_reason(&self) -> &'static str {
        match self {
            Self::BoltInBounds => "bolt position outside playfield bounds",
            Self::BoltSpeedInRange => "bolt speed outside configured min/max",
            Self::BoltCountReasonable => "bolt count exceeds maximum",
            Self::BreakerInBounds => "breaker position outside playfield bounds",
            Self::NoEntityLeaks => "unexpected entity accumulation detected",
            Self::NoNaN => "NaN detected in transform or velocity",
            Self::TimerNonNegative => "node timer went negative",
            Self::ValidStateTransitions => "invalid game state transition",
            Self::ValidBreakerState => "invalid breaker movement state transition",
            Self::TimerMonotonicallyDecreasing => "node timer increased mid-node",
            Self::BreakerPositionClamped => "breaker position not clamped to playfield",
            Self::PhysicsFrozenDuringPause => "physics entity moved while paused",
            Self::OfferingNoDuplicates => "duplicate chip in offering",
            Self::MaxedChipNeverOffered => "maxed chip appeared in offering",
            Self::ChipStacksConsistent => "held chip stacks exceed max_stacks",
            Self::RunStatsMonotonic => "run stats counter decreased mid-run",
            Self::ChipOfferExpected => "expected chip not found in offerings",
            Self::SecondWindWallAtMostOne => "more than one SecondWindWall entity exists",
            Self::ShieldChargesConsistent => "ShieldActive with zero charges not removed",
            Self::PulseRingAccumulation => "PulseRing entity count exceeds maximum",
            Self::EffectiveSpeedConsistent => {
                "EffectiveSpeedMultiplier diverged from ActiveSpeedBoosts product"
            }
            Self::ChainArcCountReasonable => "chain lightning arc/chain count exceeds maximum",
        }
    }
}
