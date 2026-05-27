use bevy::prelude::*;

// ── AfterimageConfig ────────────────────────────────────────────────────────

/// Per-run Afterimage tuning extracted from `ProtocolTuning::Afterimage` at
/// activation time.
///
/// NOTE: `PartialEq` is required by Group A5 (config equality comparisons in
/// tests) and by the wire-quiet-tick stability assertions.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct AfterimageConfig {
    /// Seconds a `PhantomBreaker` entity exists before its lifetime tick
    /// despawns it.
    pub(crate) phantom_duration:      f32,
    /// Seconds the mutated real bolt remains a phantom before `Lifespan`
    /// expiry triggers the `LifetimeEndBehavior` dispatch (revert or despawn)
    /// in `tick_bolt_lifespan`.
    pub(crate) phantom_bolt_duration: f32,
}
