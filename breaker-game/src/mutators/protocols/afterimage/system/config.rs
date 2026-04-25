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
    /// Seconds a spawned phantom-bolt entity exists before
    /// `tick_phantom_lifetime` despawns it.
    pub(crate) phantom_bolt_duration: f32,
}
