use crate::types::{ChaosParams, InputStrategy, InvariantKind, ScenarioDefinition};

// -----------------------------------------------------------------------
// Helper — minimal ScenarioDefinition builder
// -----------------------------------------------------------------------

/// Creates a minimal `ScenarioDefinition` with sensible defaults.
/// Override fields after construction for test-specific values.
pub(super) fn minimal_scenario(
    layout: &str,
    allowed_failures: Option<Vec<InvariantKind>>,
) -> ScenarioDefinition {
    ScenarioDefinition {
        breaker: "aegis".to_owned(),
        layout: layout.to_owned(),
        input: InputStrategy::Chaos(ChaosParams { action_prob: 0.1 }),
        max_frames: 100,
        disallowed_failures: vec![],
        allowed_failures,
        ..Default::default()
    }
}
