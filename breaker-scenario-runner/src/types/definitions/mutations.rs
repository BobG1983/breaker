//! Frame mutation types for self-test scenarios.

use serde::Deserialize;

/// A mutation to apply at a specific frame during a scenario run.
///
/// Used by self-test scenarios to intentionally trigger invariant violations
/// at scripted points in the run.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FrameMutation {
    /// The fixed-update frame on which this mutation is applied.
    pub frame: u32,
    /// The kind of mutation to apply.
    pub mutation: MutationKind,
}

/// Which [`RunStats`] counter to target in a [`MutationKind::DecrementRunStat`] mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum RunStatCounter {
    /// `RunStats::nodes_cleared`.
    NodesCleared,
    /// `RunStats::cells_destroyed`.
    CellsDestroyed,
    /// `RunStats::bumps_performed`.
    BumpsPerformed,
    /// `RunStats::perfect_bumps`.
    PerfectBumps,
    /// `RunStats::bolts_lost`.
    BoltsLost,
}

/// The kind of mutation to apply at a given frame.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum MutationKind {
    /// Override the breaker's movement state.
    SetDashState(ScenarioDashState),
    /// Override `NodeTimer::remaining` to this value.
    SetTimerRemaining(f32),
    /// Spawn N extra entities with `Transform` (for entity leak testing).
    SpawnExtraEntities(usize),
    /// Move the first tagged bolt to `(x, y)`, preserving z.
    MoveBolt(f32, f32),
    /// Toggle between `PlayingState::Active` and `PlayingState::Paused`.
    TogglePause,
    /// Set the named [`RunStats`] counter to a specific value.
    ///
    /// Used by the `run_stats_monotonic` self-test to seed a counter before
    /// decrementing it, making the violation deterministic.
    SetRunStat(RunStatCounter, u32),
    /// Decrement the named [`RunStats`] counter by 1.
    ///
    /// Used by the `run_stats_monotonic` self-test to intentionally trigger
    /// a [`InvariantKind::RunStatsMonotonic`] violation.
    DecrementRunStat(RunStatCounter),
    /// Inject a chip entry with `stacks > max_stacks` into [`ChipInventory`].
    ///
    /// Inserts a chip named `chip_name` with the given `stacks` and `max_stacks`
    /// bypassing the normal `add_chip` cap enforcement. Used by the
    /// `chip_stacks_consistent` self-test to trigger a
    /// [`InvariantKind::ChipStacksConsistent`] violation.
    InjectOverStackedChip {
        /// Name of the chip to inject.
        chip_name: String,
        /// Stack count to set (should exceed `max_stacks` to trigger violation).
        stacks: u32,
        /// Maximum stacks declared for this chip.
        max_stacks: u32,
    },
    /// Insert a [`ChipOffers`] resource with duplicate chip names.
    ///
    /// Used by the `offering_no_duplicates` self-test to trigger an
    /// [`InvariantKind::OfferingNoDuplicates`] violation.
    InjectDuplicateOffers {
        /// The chip name to duplicate in the offering.
        chip_name: String,
    },
    /// Insert a [`ChipOffers`] resource containing a chip that is already
    /// at max stacks in [`ChipInventory`].
    ///
    /// Used by the `maxed_chip_never_offered` self-test to trigger an
    /// [`InvariantKind::MaxedChipNeverOffered`] violation.
    InjectMaxedChipOffer {
        /// The chip name to inject as maxed in both inventory and offers.
        chip_name: String,
    },
    /// Spawn N extra `SecondWindWall` marker entities (no physics components).
    ///
    /// Used by the `second_wind_wall_at_most_one` self-test to trigger an
    /// [`InvariantKind::SecondWindWallAtMostOne`] violation.
    SpawnExtraSecondWindWalls(usize),
    /// Inject a `ShieldActive { charges: 0 }` component on the breaker entity.
    ///
    /// Used by the `shield_charges_consistent` self-test to trigger a
    /// [`InvariantKind::ShieldChargesConsistent`] violation.
    InjectZeroChargeShield,
    /// Spawn N extra `PulseRing` marker entities to push count above the threshold.
    ///
    /// Used by the `pulse_ring_accumulation` self-test to trigger a
    /// [`InvariantKind::PulseRingAccumulation`] violation.
    SpawnExtraPulseRings(usize),
    /// Spawn N [`ChainLightningChain`] + N [`ChainLightningArc`] marker entities (2N total).
    ///
    /// Used by the `chain_arc_count_exceeded` self-test to trigger a
    /// [`InvariantKind::ChainArcCountReasonable`] violation.
    SpawnExtraChainArcs(usize),
    /// Set the first tagged bolt's `Aabb2D.half_extents` to a wrong value.
    ///
    /// Used by the `aabb_matches_entity_dimensions` self-test to trigger a
    /// [`InvariantKind::AabbMatchesEntityDimensions`] violation.
    InjectMismatchedBoltAabb,
    /// Spawn N extra [`GravityWellMarker`] entities to push count above threshold.
    ///
    /// Used by the `gravity_well_count_reasonable` self-test to trigger a
    /// [`InvariantKind::GravityWellCountReasonable`] violation.
    SpawnExtraGravityWells(usize),
}

/// Mirrors `DashState` for RON deserialization in the scenario runner crate.
///
/// The game crate's `DashState` derives `Component` (which brings in Bevy
/// dependencies). This enum carries the same variants and is mapped to
/// `DashState` at runtime by
/// [`crate::lifecycle::map_scenario_dash_state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ScenarioDashState {
    /// Corresponds to `DashState::Idle`.
    Idle,
    /// Corresponds to `DashState::Dashing`.
    Dashing,
    /// Corresponds to `DashState::Braking`.
    Braking,
    /// Corresponds to `DashState::Settling`.
    Settling,
}
