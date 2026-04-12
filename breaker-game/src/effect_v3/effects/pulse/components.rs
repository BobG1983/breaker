//! Pulse runtime components.

use bevy::prelude::*;

/// Periodically emits expanding pulse shockwaves from the bolt.
#[derive(Component, Debug, Clone)]
pub struct PulseEmitter {
    /// Base range of each pulse.
    pub base_range: f32,
    /// Additional range per stack level.
    pub range_per_level: f32,
    /// Current stack count (affects range).
    pub stacks: u32,
    /// Expansion speed of emitted pulses.
    pub speed: f32,
    /// Interval in seconds between pulse emissions.
    pub interval: f32,
    /// Current countdown timer until next pulse.
    pub timer: f32,
}

/// An expanding pulse ring spawned by a [`PulseEmitter`].
#[derive(Component, Debug)]
pub struct PulseRing;
