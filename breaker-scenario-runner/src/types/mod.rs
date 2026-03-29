//! Scenario definition types loaded from RON files.
//!
//! Types here are pure data — no Bevy components or resources. They are
//! deserialized from `.scenario.ron` files and consumed by the runner.

#[cfg(test)]
mod tests;

use breaker::effect::RootEffect;
use serde::Deserialize;

/// All gameplay actions that can be injected by an input strategy.
///
/// Mirrors `breaker::input::resources::GameAction` but lives here so it can
/// derive [`Deserialize`] independently (the game crate does not expose that
/// derive on the original type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum GameAction {
    /// Continuous horizontal movement left.
    MoveLeft,
    /// Continuous horizontal movement right.
    MoveRight,
    /// Bump activation (also launches serving bolt).
    Bump,
    /// Dash left (double-tap detected).
    DashLeft,
    /// Dash right (double-tap detected).
    DashRight,
    /// Menu navigate up.
    MenuUp,
    /// Menu navigate down.
    MenuDown,
    /// Menu navigate left (horizontal menus).
    MenuLeft,
    /// Menu navigate right (horizontal menus).
    MenuRight,
    /// Menu confirm selection.
    MenuConfirm,
    /// Toggle pause state.
    TogglePause,
}

/// Bump timing mode for the [`InputStrategy::Perfect`] variant.
///
/// Controls how the `PerfectDriver` times its bump actions relative to the
/// bolt's proximity to the breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum BumpMode {
    /// Always produces a perfectly-timed bump.
    AlwaysPerfect,
    /// Always produces an early bump.
    AlwaysEarly,
    /// Always produces a late bump.
    AlwaysLate,
    /// Always produces a whiff (miss).
    AlwaysWhiff,
    /// Never bumps at all.
    NeverBump,
    /// Randomly chooses a bump timing per frame.
    Random,
}

/// A single scripted frame entry — a frame index and the actions to inject.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ScriptedFrame {
    /// The fixed-update frame on which these actions are injected.
    pub frame: u32,
    /// Actions to inject on that frame.
    pub actions: Vec<GameAction>,
}

/// Parameters for the [`InputStrategy::Chaos`] variant.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChaosParams {
    /// Probability (0.0–1.0) of injecting any action on a given frame.
    pub action_prob: f32,
}

/// Parameters for the [`InputStrategy::Scripted`] variant.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ScriptedParams {
    /// Ordered list of frame-action pairs.
    pub actions: Vec<ScriptedFrame>,
}

/// Parameters for the [`InputStrategy::Hybrid`] variant.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HybridParams {
    /// Number of frames to play back scripted actions before switching to chaos.
    pub scripted_frames: u32,
    /// Probability (0.0–1.0) for the random phase.
    pub action_prob: f32,
}

/// Input injection strategy for a scenario run.
///
/// RON newtype-variant syntax: `Chaos((action_prob: 0.3))`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum InputStrategy {
    /// Randomised action injection driven by an RNG seed and probability.
    Chaos(ChaosParams),
    /// Fully deterministic sequence of frame-action pairs.
    Scripted(ScriptedParams),
    /// Scripted actions up to `scripted_frames`, then random chaos afterwards.
    Hybrid(HybridParams),
    /// Perfect-timing input driven by a [`BumpMode`] strategy.
    Perfect(BumpMode),
}

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
    /// have `charges == 0` — zero-charge shields should be removed immediately.
    ShieldChargesConsistent,
    /// [`PulseRing`](breaker::effect::effects::pulse::PulseRing) entity count stays
    /// within `invariant_params.max_pulse_ring_count`.
    PulseRingAccumulation,
    /// [`EffectiveSpeedMultiplier`](breaker::effect::EffectiveSpeedMultiplier) must
    /// equal the product of all [`ActiveSpeedBoosts`](breaker::effect::effects::speed_boost::ActiveSpeedBoosts)
    /// entries within floating-point epsilon.
    EffectiveSpeedConsistent,
}

impl InvariantKind {
    /// All variants of [`InvariantKind`], for exhaustive iteration.
    ///
    /// Keep in sync when adding new variants — the
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
        }
    }
}

/// Mirrors `GameState` for RON deserialization in the scenario runner crate.
///
/// The game crate's `GameState` derives `States` (which brings in `Bevy`
/// dependencies that cannot appear in plain-data RON files). This enum
/// carries the same variants and is mapped to `GameState` at runtime by
/// [`crate::lifecycle::map_forced_game_state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ForcedGameState {
    /// Corresponds to `GameState::Loading`.
    Loading,
    /// Corresponds to `GameState::MainMenu`.
    MainMenu,
    /// Corresponds to `GameState::RunSetup`.
    RunSetup,
    /// Corresponds to `GameState::Playing`.
    Playing,
    /// Corresponds to `GameState::TransitionOut`.
    TransitionOut,
    /// Corresponds to `GameState::ChipSelect`.
    ChipSelect,
    /// Corresponds to `GameState::TransitionIn`.
    TransitionIn,
    /// Corresponds to `GameState::RunEnd`.
    RunEnd,
    /// Corresponds to `GameState::MetaProgression`.
    MetaProgression,
}

/// Optional debug overrides applied after entity spawn (used in self-test scenarios).
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct DebugSetup {
    /// Place bolt at this `(x, y)` world-space position instead of the default spawn.
    pub bolt_position: Option<(f32, f32)>,
    /// Place breaker at this `(x, y)` world-space position instead of the default spawn.
    #[serde(default)]
    pub breaker_position: Option<(f32, f32)>,
    /// When `true`, freeze physics so the bolt stays at the injected position.
    #[serde(default)]
    pub disable_physics: bool,
    /// Override bolt velocity for all tagged bolts.
    #[serde(default)]
    pub bolt_velocity: Option<(f32, f32)>,
    /// Number of extra bare `ScenarioTagBolt` entities to spawn (no physics components).
    #[serde(default)]
    pub extra_tagged_bolts: Option<usize>,
    /// Override `NodeTimer::remaining` to this value.
    #[serde(default)]
    pub node_timer_remaining: Option<f32>,
    /// Force `PreviousGameState` to this value (mapped to `GameState`).
    #[serde(default)]
    pub force_previous_game_state: Option<ForcedGameState>,
}

/// Tunable parameters for invariant checkers.
///
/// All fields have sensible defaults and can be overridden per-scenario
/// in the RON file via `invariant_params: (max_bolt_count: 12)`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InvariantParams {
    /// Maximum bolt count before [`InvariantKind::BoltCountReasonable`] fires.
    #[serde(default = "InvariantParams::default_max_bolt_count")]
    pub max_bolt_count: usize,
    /// Maximum [`PulseRing`](breaker::effect::effects::pulse::PulseRing) entity count before [`InvariantKind::PulseRingAccumulation`] fires.
    ///
    /// Default 20: conservative ceiling that catches accumulation bugs while tolerating
    /// burst spawning from multiple simultaneous bolt emitters.
    #[serde(default = "InvariantParams::default_max_pulse_ring_count")]
    pub max_pulse_ring_count: usize,
}

impl InvariantParams {
    const fn default_max_bolt_count() -> usize {
        8
    }

    const fn default_max_pulse_ring_count() -> usize {
        20
    }
}

impl Default for InvariantParams {
    fn default() -> Self {
        Self {
            max_bolt_count: Self::default_max_bolt_count(),
            max_pulse_ring_count: Self::default_max_pulse_ring_count(),
        }
    }
}

/// Stress-test configuration embedded in a scenario definition.
///
/// When present, the runner executes the scenario `runs` times with up to
/// `parallelism` instances running concurrently. All fields default to 32 so
/// that `stress: Some(())` in RON is a valid minimal stress config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StressConfig {
    /// Number of times to run the scenario. Defaults to 32.
    #[serde(default = "StressConfig::default_runs")]
    pub runs: usize,
    /// Maximum number of concurrent instances. Defaults to 32.
    #[serde(default = "StressConfig::default_parallelism")]
    pub parallelism: usize,
}

impl StressConfig {
    const fn default_runs() -> usize {
        32
    }

    const fn default_parallelism() -> usize {
        32
    }
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            runs: Self::default_runs(),
            parallelism: Self::default_parallelism(),
        }
    }
}

/// Full scenario definition loaded from a `.scenario.ron` file.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioDefinition {
    /// Breaker name (e.g. `"Aegis"`, `"Prism"`, `"Chrono"`).
    pub breaker: String,
    /// Layout name to look up via `NodeLayoutRegistry::get_by_name`.
    pub layout: String,
    /// Input injection strategy for this scenario.
    pub input: InputStrategy,
    /// Maximum number of fixed-update frames before the runner exits.
    pub max_frames: u32,
    /// Invariants to check continuously during the run.
    pub invariants: Vec<InvariantKind>,
    /// If `Some`, the scenario expects exactly these invariant violations to fire
    /// (used for self-test scenarios that intentionally trigger invariants).
    pub expected_violations: Option<Vec<InvariantKind>>,
    /// Optional debug overrides for self-test scenarios.
    pub debug_setup: Option<DebugSetup>,
    /// Tunable thresholds for invariant checkers.
    #[serde(default)]
    pub invariant_params: InvariantParams,
    /// When `true` (default), the scenario exits when the game naturally reaches
    /// `RunEnd` (timer, lives, node cleared). When `false`, `RunEnd` is
    /// intercepted and the run restarts — only `max_frames` triggers exit.
    #[serde(default = "ScenarioDefinition::default_allow_early_end")]
    pub allow_early_end: bool,
    /// Optional stress-test configuration. When `Some`, the runner executes the
    /// scenario multiple times in parallel instead of a single run.
    #[serde(default)]
    pub stress: Option<StressConfig>,
    /// Optional deterministic seed for the game RNG. All scenarios default to
    /// seed 0 for determinism. Override in RON to test different seeds.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Optional chip names to pre-select at scenario start.
    ///
    /// Each string is a chip name dispatched via [`ChipSelected`] in
    /// [`crate::lifecycle::bypass_menu_to_playing`].
    #[serde(default)]
    pub initial_chips: Option<Vec<String>>,
    /// Optional per-frame mutations for self-test scenarios.
    ///
    /// When `Some`, each [`FrameMutation`] is applied at its specified frame
    /// by [`crate::lifecycle::apply_debug_frame_mutations`].
    #[serde(default)]
    pub frame_mutations: Option<Vec<FrameMutation>>,
    /// Optional chip selections to pre-populate at scenario start.
    ///
    /// Each string is a chip name (e.g. `"Surge"`) that will be added to the
    /// chip inventory before the run begins.
    #[serde(default)]
    pub chip_selections: Option<Vec<String>>,
    /// Optional root effects to pre-populate at scenario start.
    ///
    /// Each [`RootEffect`] is injected into the active effects before the run
    /// begins.
    #[serde(default)]
    pub initial_effects: Option<Vec<RootEffect>>,
    /// Chip names that MUST appear in offerings when chip select is entered.
    ///
    /// Checked by the [`InvariantKind::ChipOfferExpected`] invariant. Each name
    /// is matched against `ChipOffering::name()` in the `ChipOffers` resource.
    #[serde(default)]
    pub expected_offerings: Option<Vec<String>>,
}

impl ScenarioDefinition {
    const fn default_allow_early_end() -> bool {
        true
    }
}

impl Default for ScenarioDefinition {
    fn default() -> Self {
        Self {
            breaker: String::new(),
            layout: String::new(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 1000,
            invariants: vec![],
            expected_violations: None,
            debug_setup: None,
            invariant_params: InvariantParams::default(),
            allow_early_end: true,
            stress: None,
            seed: None,
            initial_chips: None,
            frame_mutations: None,
            chip_selections: None,
            initial_effects: None,
            expected_offerings: None,
        }
    }
}

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
    SetBreakerState(ScenarioBreakerState),
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
    /// Override `EffectiveSpeedMultiplier` to a wrong value on all entities that
    /// also have `ActiveSpeedBoosts`.
    ///
    /// Used by the `effective_speed_consistent` self-test to trigger a
    /// [`InvariantKind::EffectiveSpeedConsistent`] violation by creating
    /// a stale/diverged multiplier.
    InjectWrongEffectiveSpeed {
        /// The incorrect value to set on `EffectiveSpeedMultiplier`.
        wrong_value: f32,
    },
}

/// Mirrors `BreakerState` for RON deserialization in the scenario runner crate.
///
/// The game crate's `BreakerState` derives `Component` (which brings in Bevy
/// dependencies). This enum carries the same variants and is mapped to
/// `BreakerState` at runtime by
/// [`crate::lifecycle::map_scenario_breaker_state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ScenarioBreakerState {
    /// Corresponds to `BreakerState::Idle`.
    Idle,
    /// Corresponds to `BreakerState::Dashing`.
    Dashing,
    /// Corresponds to `BreakerState::Braking`.
    Braking,
    /// Corresponds to `BreakerState::Settling`.
    Settling,
}
