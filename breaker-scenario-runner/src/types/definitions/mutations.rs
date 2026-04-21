//! Frame mutation types for self-test scenarios.

use serde::Deserialize;

/// A mutation to apply at a specific frame during a scenario run.
///
/// Used by self-test scenarios to intentionally trigger invariant violations
/// at scripted points in the run.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FrameMutation {
    /// The fixed-update frame on which this mutation is applied.
    pub frame:    u32,
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
    /// Toggle pause via `Time<Virtual>` — pauses or unpauses the game.
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
        chip_name:  String,
        /// Stack count to set (should exceed `max_stacks` to trigger violation).
        stacks:     u32,
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
    /// Spawn N extra `ShieldWall` marker entities.
    ///
    /// Used by the `shield_wall_at_most_one` self-test to trigger a
    /// [`InvariantKind::ShieldWallAtMostOne`] violation.
    SpawnExtraShieldWalls(usize),
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
    /// Spawn N extra [`GravityWell`] entities to push count above threshold.
    ///
    /// Used by the `gravity_well_count_reasonable` self-test to trigger a
    /// [`InvariantKind::GravityWellCountReasonable`] violation.
    SpawnExtraGravityWells(usize),
    /// Spawn N extra [`PrimaryBreaker`] marker entities.
    ///
    /// Used by the `breaker_count_reasonable` self-test to trigger a
    /// [`InvariantKind::BreakerCountReasonable`] violation.
    SpawnExtraPrimaryBreakers(usize),
    /// Set the first tagged bolt's [`CollisionLayers`] to non-zero values
    /// while it has a [`Birthing`] component.
    ///
    /// Used by the `bolt_birthing_layers_zeroed` self-test to trigger a
    /// [`InvariantKind::BoltBirthingLayersZeroed`] violation.
    InjectNonZeroBirthingLayers,
    /// Spawn `count` extra entities each carrying `(Bolt, PrimaryBolt,
    /// Position2D(Vec2::ZERO), BaseSpeed(400.0))`.
    ///
    /// Used by the `exactly_one_primary_bolt_self_test` self-test scenario to
    /// deliberately push the `PrimaryBolt` count above 1, triggering an
    /// [`InvariantKind::ExactlyOnePrimaryBolt`] violation.
    SpawnExtraPrimaryBolts {
        /// Number of extra `(Bolt, PrimaryBolt)` entities to spawn.
        count: u32,
    },
    /// Override the `heat` and `still_timer` fields of the first tagged
    /// breaker's [`BurnoutHeat`] component.
    ///
    /// Used by the `burnout_heat_clamped` self-test to force `heat` outside
    /// `[0.0, 1.0]`, intentionally triggering an
    /// [`InvariantKind::BurnoutHeatClamped`] violation.
    SetBurnoutHeat {
        /// Value to write into `BurnoutHeat.heat`.
        heat:        f32,
        /// Value to write into `BurnoutHeat.still_timer`.
        still_timer: f32,
    },
    /// Insert a 0-stack entry into [`ActiveHazards`] via the
    /// `force_insert_entry` backdoor, bypassing `add_stack`. Used by the
    /// `hazard_stack_valid` self-test to trigger a
    /// [`InvariantKind::HazardStackValid`] violation.
    InjectZeroStackHazard {
        /// Which hazard kind to inject (matches the [`HazardKind`] variant
        /// name, e.g. `"Decay"`, `"Drift"`).
        kind_name: String,
    },
    /// Install a hazard with a real stack count, running the normal
    /// activation path so the per-kind config resource is inserted. Used by
    /// hazard-runtime scenarios that need to verify the gameplay effect of
    /// a hazard without going through the `HazardSelect` UI.
    InjectHazardStack {
        /// Which hazard kind to inject (matches the [`HazardKind`] variant
        /// name, e.g. `"Decay"`).
        kind_name: String,
        /// How many stacks to install. Each stack runs the full activation
        /// path (increment `ActiveHazards`, insert/refresh config).
        stacks:    u32,
    },
    /// Activate a protocol via the normal dispatch path: insert into
    /// `ActiveProtocols`, stamp effect trees onto every tagged breaker, and
    /// (for custom-system protocols) insert the per-kind config resource.
    /// Used by protocol-runtime scenarios to install a protocol without
    /// going through the chip-select UI.
    InjectProtocol {
        /// Which protocol kind to inject (matches the [`ProtocolKind`]
        /// variant name, e.g. `"Deadline"`).
        kind_name: String,
    },
    /// Directly write a value into `GreedStacks.skips`, bypassing normal
    /// chip-skip accounting.
    ///
    /// Used by the `greed_stacks_orphaned` self-test to seed a non-zero skip
    /// count before removing Greed from `ActiveProtocols`, triggering an
    /// [`InvariantKind::GreedStacksOrphaned`] violation.
    SetGreedStacks {
        /// Value to write into `GreedStacks.skips`.
        skips: u32,
    },
    /// Directly write a value into `SiphonStreak.kill_count`, bypassing
    /// normal kill-streak accumulation.
    ///
    /// Used by the `siphon_streak_orphaned` self-test to seed a non-zero
    /// `kill_count` before removing Siphon from `ActiveProtocols`, triggering
    /// an [`InvariantKind::SiphonStreakOrphaned`] violation.
    SetSiphonStreak {
        /// Value to write into `SiphonStreak.kill_count`.
        kill_count:       u32,
        /// Value to write into `SiphonStreak.window_remaining`.
        window_remaining: f32,
    },
    /// Remove a protocol from `ActiveProtocols` by kind, bypassing any
    /// normal deactivation/cleanup path.
    ///
    /// Used by contract self-test scenarios to deliberately orphan
    /// per-protocol resources (e.g. `GreedStacks`, `SiphonStreak`) after
    /// the protocol is active, triggering
    /// [`InvariantKind::GreedStacksOrphaned`] or
    /// [`InvariantKind::SiphonStreakOrphaned`] on the next frame.
    RemoveFromActiveProtocols {
        /// Which protocol kind to remove (matches the [`ProtocolKind`]
        /// variant name, e.g. `"Greed"`).
        kind_name: String,
    },
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
