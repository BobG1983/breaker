//! Top-level scenario definition and supporting config types.

use breaker::effect::RootEffect;
use serde::Deserialize;

use super::{
    input::{InputStrategy, ScriptedParams},
    invariants::InvariantKind,
    mutations::FrameMutation,
};

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
    /// Maximum combined [`ChainLightningChain`](breaker::effect::effects::chain_lightning::ChainLightningChain) +
    /// [`ChainLightningArc`](breaker::effect::effects::chain_lightning::ChainLightningArc) entity count
    /// before [`InvariantKind::ChainArcCountReasonable`] fires.
    ///
    /// Default 50: conservative ceiling -- a single chain lightning fires at most ~10 arcs,
    /// so 50 allows multiple simultaneous chains while catching unbounded accumulation.
    #[serde(default = "InvariantParams::default_max_chain_arc_count")]
    pub max_chain_arc_count: usize,
    /// Maximum [`GravityWellMarker`] entity count before
    /// [`InvariantKind::GravityWellCountReasonable`] fires.
    ///
    /// Default 10: conservative ceiling for gravity well entities.
    #[serde(default = "InvariantParams::default_max_gravity_well_count")]
    pub max_gravity_well_count: usize,
}

impl InvariantParams {
    const fn default_max_bolt_count() -> usize {
        8
    }

    const fn default_max_pulse_ring_count() -> usize {
        20
    }

    const fn default_max_chain_arc_count() -> usize {
        50
    }

    const fn default_max_gravity_well_count() -> usize {
        10
    }
}

impl Default for InvariantParams {
    fn default() -> Self {
        Self {
            max_bolt_count: Self::default_max_bolt_count(),
            max_pulse_ring_count: Self::default_max_pulse_ring_count(),
            max_chain_arc_count: Self::default_max_chain_arc_count(),
            max_gravity_well_count: Self::default_max_gravity_well_count(),
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
    pub disallowed_failures: Vec<InvariantKind>,
    /// If `Some`, the scenario expects exactly these invariant violations to fire
    /// (used for self-test scenarios that intentionally trigger invariants).
    pub allowed_failures: Option<Vec<InvariantKind>>,
    /// Optional debug overrides for self-test scenarios.
    pub debug_setup: Option<DebugSetup>,
    /// Tunable thresholds for invariant checkers.
    #[serde(default)]
    pub invariant_params: InvariantParams,
    /// When `true` (default), the scenario exits when the game naturally reaches
    /// `RunEnd` (timer, lives, node cleared). When `false`, `RunEnd` is
    /// intercepted and the run restarts -- only `max_frames` triggers exit.
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
            disallowed_failures: vec![],
            allowed_failures: None,
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
