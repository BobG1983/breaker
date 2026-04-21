//! Invariant kinds the runner can check during a scenario run.

use serde::{Deserialize, Serialize};

/// Invariant kinds the runner can check during a scenario run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum InvariantKind {
    /// Bolt position stays within playfield bounds.
    BoltInBounds,
    /// Bolt speed matches the expected derived value.
    BoltSpeedAccurate,
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
    /// Breaker movement state machine only takes legal transitions.
    ValidDashState,
    /// Node timer decreases monotonically (never increases mid-node).
    TimerMonotonicallyDecreasing,
    /// Breaker x position stays within playfield bounds minus half-width.
    BreakerPositionClamped,
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
    /// At most 1 [`ShieldWall`](breaker::effect::effects::shield::ShieldWall)
    /// entity should exist at any frame.
    ShieldWallAtMostOne,
    /// [`PulseRing`](breaker::effect::effects::pulse::PulseRing) entity count stays
    /// within `invariant_params.max_pulse_ring_count`.
    PulseRingAccumulation,
    /// Chain lightning chain + arc entity count stays within `invariant_params.max_chain_arc_count`.
    ChainArcCountReasonable,
    /// Bolt `Aabb2D` `half_extents` match the entity's actual dimensions.
    AabbMatchesEntityDimensions,
    /// Gravity well entity count stays within `invariant_params.max_gravity_well_count`.
    GravityWellCountReasonable,
    /// Exactly one `PrimaryBreaker` entity should exist during gameplay.
    BreakerCountReasonable,
    /// Bolts with `Birthing` component must have zeroed `CollisionLayers`.
    BoltBirthingLayersZeroed,
    /// [`ActiveHazards`](breaker::hazard::resources::ActiveHazards) must never
    /// contain an entry with 0 stacks — the map-insert path is `add_stack`
    /// which always increments to ≥1.
    HazardStackValid,
    /// Exactly one `PrimaryBolt` entity should exist whenever any `Bolt`
    /// entity exists. Catches Conductor-protocol swap bugs where the marker is
    /// briefly absent or double-assigned.
    ExactlyOnePrimaryBolt,
    /// Every [`BurnoutHeat`](breaker::protocol::protocols::burnout::system::BurnoutHeat)
    /// component in the world has `heat` within `[0.0, 1.0]` (inclusive) and
    /// `still_timer >= 0.0` at every frame.
    BurnoutHeatClamped,
    /// CONTRACT: `GreedStacks.skips > 0` is only legal when
    /// `ActiveProtocols` contains `ProtocolKind::Greed`.
    ///
    /// Fires when Greed is NOT active but `GreedStacks.skips > 0`, indicating
    /// orphaned skip state leaked from a prior run or protocol removal without
    /// cleanup.
    GreedStacksOrphaned,
    /// CONTRACT: `SiphonStreak.kill_count > 0` is only legal when
    /// `ActiveProtocols` contains `ProtocolKind::Siphon`.
    ///
    /// Fires when Siphon is NOT active but `SiphonStreak.kill_count > 0`,
    /// indicating orphaned streak state leaked from a prior run or protocol
    /// removal without cleanup.
    SiphonStreakOrphaned,
    /// CONTRACT: `FissionCounter.kills > 0` is only legal when
    /// `ActiveProtocols` contains `ProtocolKind::Fission`.
    ///
    /// Fires when Fission is NOT active but `FissionCounter.kills > 0`,
    /// indicating orphaned kill-count state leaked from a prior run or
    /// protocol removal without cleanup.
    FissionCounterOrphaned,
    /// CONTRACT: no bolt carries `RiskyDamageBoost` and
    /// `RecklessDashDoubledBolts` is empty only when `ActiveProtocols`
    /// contains `ProtocolKind::RecklessDash`.
    ///
    /// Fires when `RecklessDash` is NOT active but either a bolt has
    /// `RiskyDamageBoost` or `RecklessDashDoubledBolts` is non-empty,
    /// indicating orphaned boost/penalty state.
    RecklessDashOrphaned,
    /// CONTRACT: no bolt carries `EchoNetwork` or `EchoPrimed` when
    /// `ActiveProtocols` does not contain `ProtocolKind::EchoStrike`.
    ///
    /// Fires when `EchoStrike` is NOT active but any bolt has `EchoNetwork`
    /// or `EchoPrimed`, indicating orphaned echo state.
    EchoStrikeOrphaned,
    /// CONTRACT: no bolt carries `DebtStack` (value > 0.0) or `DebtCashOut`
    /// when `ActiveProtocols` does not contain `ProtocolKind::DebtCollector`.
    ///
    /// Fires when `DebtCollector` is NOT active but any bolt has a non-zero
    /// `DebtStack` or carries `DebtCashOut`, indicating orphaned debt state.
    DebtCollectorOrphaned,
    /// CONTRACT: no breaker carries `BurnoutHeat` with `heat > 0.0` and no
    /// bolt carries `BurnoutDamageBoost` when `ActiveProtocols` does not
    /// contain `ProtocolKind::Burnout`.
    ///
    /// Fires when Burnout is NOT active but any breaker has positive heat or
    /// any bolt has `BurnoutDamageBoost`, indicating orphaned burnout state.
    BurnoutStateOrphaned,
}

impl InvariantKind {
    /// All variants of [`InvariantKind`], for exhaustive iteration.
    ///
    /// Keep in sync when adding new variants -- the
    /// `all_variants_covered_by_invariant_kind_all` test enforces this via
    /// the `fail_reason()` exhaustive match.
    pub const ALL: &[Self] = &[
        Self::BoltInBounds,
        Self::BoltSpeedAccurate,
        Self::BoltCountReasonable,
        Self::BreakerInBounds,
        Self::NoEntityLeaks,
        Self::NoNaN,
        Self::TimerNonNegative,
        Self::ValidDashState,
        Self::TimerMonotonicallyDecreasing,
        Self::BreakerPositionClamped,
        Self::OfferingNoDuplicates,
        Self::MaxedChipNeverOffered,
        Self::ChipStacksConsistent,
        Self::RunStatsMonotonic,
        Self::ChipOfferExpected,
        Self::SecondWindWallAtMostOne,
        Self::ShieldWallAtMostOne,
        Self::PulseRingAccumulation,
        Self::ChainArcCountReasonable,
        Self::AabbMatchesEntityDimensions,
        Self::GravityWellCountReasonable,
        Self::BreakerCountReasonable,
        Self::BoltBirthingLayersZeroed,
        Self::HazardStackValid,
        Self::ExactlyOnePrimaryBolt,
        Self::BurnoutHeatClamped,
        Self::GreedStacksOrphaned,
        Self::SiphonStreakOrphaned,
        Self::FissionCounterOrphaned,
        Self::RecklessDashOrphaned,
        Self::EchoStrikeOrphaned,
        Self::DebtCollectorOrphaned,
        Self::BurnoutStateOrphaned,
    ];

    /// Standard human-readable fail reason for this invariant violation.
    ///
    /// Used by [`crate::verdict::ScenarioVerdict`] to build structured failure reasons
    /// without string construction at evaluation time.
    #[must_use]
    pub const fn fail_reason(&self) -> &'static str {
        match self {
            Self::BoltInBounds => "bolt position outside playfield bounds",
            Self::BoltSpeedAccurate => "bolt speed outside configured min/max",
            Self::BoltCountReasonable => "bolt count exceeds maximum",
            Self::BreakerInBounds => "breaker position outside playfield bounds",
            Self::NoEntityLeaks => "unexpected entity accumulation detected",
            Self::NoNaN => "NaN detected in transform or velocity",
            Self::TimerNonNegative => "node timer went negative",
            Self::ValidDashState => "invalid breaker movement state transition",
            Self::TimerMonotonicallyDecreasing => "node timer increased mid-node",
            Self::BreakerPositionClamped => "breaker position not clamped to playfield",
            Self::OfferingNoDuplicates => "duplicate chip in offering",
            Self::MaxedChipNeverOffered => "maxed chip appeared in offering",
            Self::ChipStacksConsistent => "held chip stacks exceed max_stacks",
            Self::RunStatsMonotonic => "run stats counter decreased mid-run",
            Self::ChipOfferExpected => "expected chip not found in offerings",
            Self::SecondWindWallAtMostOne => "more than one SecondWindWall entity exists",
            Self::ShieldWallAtMostOne => "more than one ShieldWall entity exists",
            Self::PulseRingAccumulation => "PulseRing entity count exceeds maximum",
            Self::ChainArcCountReasonable => "chain lightning arc/chain count exceeds maximum",
            Self::AabbMatchesEntityDimensions => {
                "Aabb2D half_extents do not match entity dimensions"
            }
            Self::GravityWellCountReasonable => "gravity well entity count exceeds maximum",
            Self::BreakerCountReasonable => "primary breaker count is not exactly 1",
            Self::BoltBirthingLayersZeroed => "birthing bolt has non-zero collision layers",
            Self::HazardStackValid => "hazard stack count is zero (should never happen)",
            Self::ExactlyOnePrimaryBolt => "PrimaryBolt count is not exactly 1 while bolts exist",
            Self::BurnoutHeatClamped => {
                "BurnoutHeat.heat outside [0.0, 1.0] or still_timer negative"
            }
            Self::GreedStacksOrphaned => {
                "GreedStacks.skips > 0 while Greed is not active in ActiveProtocols"
            }
            Self::SiphonStreakOrphaned => {
                "SiphonStreak.kill_count > 0 while Siphon is not active in ActiveProtocols"
            }
            Self::FissionCounterOrphaned => {
                "FissionCounter.kills > 0 while Fission is not active in ActiveProtocols"
            }
            Self::RecklessDashOrphaned => {
                "RiskyDamageBoost or RecklessDashDoubledBolts non-empty while RecklessDash is not active"
            }
            Self::EchoStrikeOrphaned => {
                "EchoNetwork or EchoPrimed on bolt while EchoStrike is not active in ActiveProtocols"
            }
            Self::DebtCollectorOrphaned => {
                "DebtStack (>0) or DebtCashOut on bolt while DebtCollector is not active in ActiveProtocols"
            }
            Self::BurnoutStateOrphaned => {
                "BurnoutHeat.heat > 0 on breaker or BurnoutDamageBoost on bolt while Burnout is not active"
            }
        }
    }
}
