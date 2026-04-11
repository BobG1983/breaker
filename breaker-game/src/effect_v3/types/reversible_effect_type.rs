//! `ReversibleEffectType` — the subset of effects that can be cleanly reversed.

use serde::{Deserialize, Serialize};

use crate::effect_v3::effects::*;

/// The subset of `EffectType` that can appear as direct Fire children
/// inside During/Until scoped trees. These effects can be cleanly reversed
/// when the scope ends.
///
/// Config structs in this enum implement both `Fireable` and `Reversible` traits.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReversibleEffectType {
    /// Passive: multiply bolt speed.
    SpeedBoost(SpeedBoostConfig),
    /// Passive: multiply breaker size.
    SizeBoost(SizeBoostConfig),
    /// Passive: multiply bolt damage.
    DamageBoost(DamageBoostConfig),
    /// Passive: multiply bump force.
    BumpForce(BumpForceConfig),
    /// Passive: multiply breaker deceleration.
    QuickStop(QuickStopConfig),
    /// Toggle: enable flash step dash.
    FlashStep(FlashStepConfig),
    /// Passive: grant piercing hits.
    Piercing(PiercingConfig),
    /// Passive: multiply incoming damage.
    Vulnerable(VulnerableConfig),
    /// Passive: accumulate ramping damage.
    RampingDamage(RampingDamageConfig),
    /// Spawned: attraction steering toward entities.
    Attraction(AttractionConfig),
    /// Spawned: anchor bolt in place.
    Anchor(AnchorConfig),
    /// Spawned: periodic shockwave emitter.
    Pulse(PulseConfig),
    /// Spawned: shield wall protection.
    Shield(ShieldConfig),
    /// Spawned: second wind wall.
    SecondWind(SecondWindConfig),
    /// Stateful: circuit breaker counter.
    CircuitBreaker(CircuitBreakerConfig),
    /// Stateful: entropy engine counter.
    EntropyEngine(EntropyConfig),
}
