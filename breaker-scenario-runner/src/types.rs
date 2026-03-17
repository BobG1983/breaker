//! Scenario definition types loaded from RON files.
//!
//! Types here are pure data — no Bevy components or resources. They are
//! deserialized from `.scenario.ron` files and consumed by the runner.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InvariantParams {
    /// Maximum bolt count before [`InvariantKind::BoltCountReasonable`] fires.
    #[serde(default = "InvariantParams::default_max_bolt_count")]
    pub max_bolt_count: usize,
}

impl InvariantParams {
    fn default_max_bolt_count() -> usize {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // InputStrategy — Chaos
    // -------------------------------------------------------------------------

    #[test]
    fn chaos_input_strategy_parses_from_ron() {
        // RON newtype-variant syntax: Chaos((field: val, ...))
        let ron = "Chaos((seed: 42, action_prob: 0.3))";
        let result: InputStrategy = ron::de::from_str(ron).expect("Chaos should parse");
        assert_eq!(
            result,
            InputStrategy::Chaos(ChaosParams {
                seed: 42,
                action_prob: 0.3,
            })
        );
    }

    // -------------------------------------------------------------------------
    // InputStrategy — Scripted
    // -------------------------------------------------------------------------

    #[test]
    fn scripted_input_strategy_parses_from_ron() {
        let ron = r"Scripted((actions: [
            (frame: 10, actions: [MoveLeft]),
            (frame: 20, actions: [Bump, MoveRight]),
        ]))";

        let result: InputStrategy = ron::de::from_str(ron).expect("Scripted should parse");
        match result {
            InputStrategy::Scripted(params) => {
                assert_eq!(params.actions.len(), 2, "expected 2 scripted entries");
                assert_eq!(params.actions[0].frame, 10);
                assert_eq!(params.actions[0].actions, vec![GameAction::MoveLeft]);
                assert_eq!(params.actions[1].frame, 20);
                assert_eq!(
                    params.actions[1].actions,
                    vec![GameAction::Bump, GameAction::MoveRight]
                );
            }
            other => panic!("expected Scripted variant, got {other:?}"),
        }
    }

    #[test]
    fn scripted_input_strategy_empty_actions_list_parses() {
        let ron = "Scripted((actions: []))";
        let result: InputStrategy = ron::de::from_str(ron).expect("empty Scripted should parse");
        assert_eq!(
            result,
            InputStrategy::Scripted(ScriptedParams { actions: vec![] })
        );
    }

    // -------------------------------------------------------------------------
    // InputStrategy — Hybrid
    // -------------------------------------------------------------------------

    #[test]
    fn hybrid_input_strategy_parses_from_ron() {
        let ron = "Hybrid((scripted_frames: 100, seed: 7, action_prob: 0.5))";
        let result: InputStrategy = ron::de::from_str(ron).expect("Hybrid should parse");
        assert_eq!(
            result,
            InputStrategy::Hybrid(HybridParams {
                scripted_frames: 100,
                seed: 7,
                action_prob: 0.5,
            })
        );
    }

    // -------------------------------------------------------------------------
    // InvariantKind — all variants
    // -------------------------------------------------------------------------

    #[test]
    fn invariant_kind_bolt_in_bounds_parses() {
        let result: InvariantKind =
            ron::de::from_str("BoltInBounds").expect("BoltInBounds should parse");
        assert_eq!(result, InvariantKind::BoltInBounds);
    }

    #[test]
    fn invariant_kind_breaker_in_bounds_parses() {
        let result: InvariantKind =
            ron::de::from_str("BreakerInBounds").expect("BreakerInBounds should parse");
        assert_eq!(result, InvariantKind::BreakerInBounds);
    }

    #[test]
    fn invariant_kind_no_entity_leaks_parses() {
        let result: InvariantKind =
            ron::de::from_str("NoEntityLeaks").expect("NoEntityLeaks should parse");
        assert_eq!(result, InvariantKind::NoEntityLeaks);
    }

    #[test]
    fn invariant_kind_no_nan_parses() {
        let result: InvariantKind = ron::de::from_str("NoNaN").expect("NoNaN should parse");
        assert_eq!(result, InvariantKind::NoNaN);
    }

    #[test]
    fn invariant_kind_valid_state_transitions_parses() {
        let result: InvariantKind =
            ron::de::from_str("ValidStateTransitions").expect("ValidStateTransitions should parse");
        assert_eq!(result, InvariantKind::ValidStateTransitions);
    }

    // -------------------------------------------------------------------------
    // ScenarioDefinition — expected_violations field
    // -------------------------------------------------------------------------

    #[test]
    fn scenario_definition_expected_violations_some_parses() {
        let ron = r#"(
            breaker: "aegis",
            layout: "corridor",
            input: Chaos((seed: 1, action_prob: 0.1)),
            max_frames: 1000,
            invariants: [BoltInBounds, NoNaN],
            expected_violations: Some([BoltInBounds, NoNaN]),
            debug_setup: None,
        )"#;
        let result: ScenarioDefinition =
            ron::de::from_str(ron).expect("ScenarioDefinition with Some violations should parse");
        assert_eq!(
            result.expected_violations,
            Some(vec![InvariantKind::BoltInBounds, InvariantKind::NoNaN])
        );
    }

    #[test]
    fn scenario_definition_expected_violations_none_parses() {
        let ron = r#"(
            breaker: "aegis",
            layout: "corridor",
            input: Chaos((seed: 1, action_prob: 0.1)),
            max_frames: 1000,
            invariants: [],
            expected_violations: None,
            debug_setup: None,
        )"#;
        let result: ScenarioDefinition =
            ron::de::from_str(ron).expect("ScenarioDefinition with None violations should parse");
        assert!(result.expected_violations.is_none());
    }

    // -------------------------------------------------------------------------
    // DebugSetup — partial fields
    // -------------------------------------------------------------------------

    #[test]
    fn debug_setup_with_bolt_position_only_parses() {
        let ron = "(bolt_position: Some((0.0, -500.0)))";
        let result: DebugSetup =
            ron::de::from_str(ron).expect("DebugSetup with bolt_position should parse");
        assert_eq!(result.bolt_position, Some((0.0_f32, -500.0_f32)));
        assert!(
            !result.disable_physics,
            "disable_physics should default to false"
        );
    }

    #[test]
    fn debug_setup_default_has_no_overrides() {
        let default = DebugSetup::default();
        assert!(default.bolt_position.is_none());
        assert!(!default.disable_physics);
    }

    // -------------------------------------------------------------------------
    // ScenarioDefinition — full round-trip
    // -------------------------------------------------------------------------

    #[test]
    fn full_scenario_definition_parses_all_fields() {
        let ron = r#"(
            breaker: "aegis",
            layout: "corridor",
            input: Chaos((seed: 99, action_prob: 0.25)),
            max_frames: 20000,
            invariants: [BoltInBounds, NoNaN],
            expected_violations: None,
            debug_setup: None,
        )"#;
        let result: ScenarioDefinition =
            ron::de::from_str(ron).expect("full ScenarioDefinition should parse");

        assert_eq!(result.breaker, "aegis");
        assert_eq!(result.layout, "corridor");
        assert_eq!(
            result.input,
            InputStrategy::Chaos(ChaosParams {
                seed: 99,
                action_prob: 0.25,
            })
        );
        assert_eq!(result.max_frames, 20_000);
        assert_eq!(
            result.invariants,
            vec![InvariantKind::BoltInBounds, InvariantKind::NoNaN]
        );
        assert!(result.expected_violations.is_none());
        assert!(result.debug_setup.is_none());
    }
}
