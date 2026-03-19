//! Scenario definition types loaded from RON files.
//!
//! Types here are pure data — no Bevy components or resources. They are
//! deserialized from `.scenario.ron` files and consumed by the runner.

#[cfg(test)]
mod tests;

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
    /// Seed for the RNG.
    pub seed: u64,
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
    /// Seed for the random phase.
    pub seed: u64,
    /// Probability (0.0–1.0) for the random phase.
    pub action_prob: f32,
}

/// Input injection strategy for a scenario run.
///
/// RON newtype-variant syntax: `Chaos((seed: 42, action_prob: 0.3))`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum InputStrategy {
    /// Randomised action injection driven by an RNG seed and probability.
    Chaos(ChaosParams),
    /// Fully deterministic sequence of frame-action pairs.
    Scripted(ScriptedParams),
    /// Scripted actions up to `scripted_frames`, then random chaos afterwards.
    Hybrid(HybridParams),
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
        }
    }
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
}

impl InvariantParams {
    const fn default_max_bolt_count() -> usize {
        8
    }
}

impl Default for InvariantParams {
    fn default() -> Self {
        Self {
            max_bolt_count: Self::default_max_bolt_count(),
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
    /// Breaker archetype name (e.g. `"aegis"`, `"prism"`, `"chrono"`).
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
}

impl ScenarioDefinition {
    const fn default_allow_early_end() -> bool {
        true
    }
}
