//! Pulse runtime components.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::effect_v3::components::EffectSourceChip;

/// Periodically emits expanding pulse shockwaves from the bolt.
#[derive(Component, Debug, Clone)]
pub struct PulseEmitter {
    /// Base range of each pulse.
    pub base_range:      f32,
    /// Additional range per stack level.
    pub range_per_level: f32,
    /// Current stack count (affects range).
    pub stacks:          u32,
    /// Expansion speed of emitted pulses.
    pub speed:           f32,
    /// Interval in seconds between pulse emissions.
    pub interval:        f32,
    /// Current countdown timer until next pulse.
    pub timer:           f32,
    /// Chip that installed this emitter. Propagated into each spawned ring's
    /// [`EffectSourceChip`] at `tick_pulse` spawn time.
    pub source_chip:     EffectSourceChip,
}

/// An expanding pulse ring spawned by a [`PulseEmitter`].
#[derive(Component, Debug)]
pub struct PulseRing;

/// Current expanding radius of a pulse ring (world units).
#[derive(Component, Debug, Clone)]
pub struct PulseRingRadius(pub f32);

/// Maximum radius a pulse ring can reach before being despawned.
#[derive(Component, Debug, Clone)]
pub struct PulseRingMaxRadius(pub f32);

/// Expansion speed of a pulse ring in world units per second.
#[derive(Component, Debug, Clone)]
pub struct PulseRingSpeed(pub f32);

/// Set of cell entities already damaged by this pulse ring (prevents double-hit).
#[derive(Component, Debug, Clone)]
pub struct PulseRingDamaged(pub HashSet<Entity>);

/// Snapshot of the emitter's `BoltBaseDamage` at the tick this ring was spawned.
#[derive(Component, Debug, Clone)]
pub struct PulseRingBaseDamage(pub f32);

/// Snapshot of `EffectStack<DamageBoostConfig>.aggregate()` at the tick this ring was spawned.
#[derive(Component, Debug, Clone)]
pub struct PulseRingDamageMultiplier(pub f32);
