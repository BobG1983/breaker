//! Circuit breaker runtime components.

use bevy::prelude::*;

/// Tracks bump counts toward triggering an automatic shockwave.
#[derive(Component, Debug, Clone)]
pub struct CircuitBreakerCounter {
    /// Bumps remaining until the next shockwave fires.
    pub remaining: u32,
    /// Total bumps required per cycle.
    pub bumps_required: u32,
    /// Number of shockwaves to spawn when the counter reaches zero.
    pub spawn_count: u32,
    /// Whether child bolts inherit this counter.
    pub inherit: bool,
    /// Radius of the triggered shockwave.
    pub shockwave_range: f32,
    /// Expansion speed of the triggered shockwave.
    pub shockwave_speed: f32,
}
