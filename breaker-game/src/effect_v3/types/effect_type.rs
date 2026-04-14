//! `EffectType` — all effect variants, each wrapping a config struct.

use serde::{Deserialize, Serialize};

use super::ReversibleEffectType;
use crate::effect_v3::effects::*;

/// Every effect in the game. Each variant wraps a config struct that
/// implements the `Fireable` trait. The enum is the dispatch layer;
/// the config struct is the implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
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
    /// Spawned: expanding damage shockwave.
    Shockwave(ShockwaveConfig),
    /// Fire-and-forget: area explosion.
    Explode(ExplodeConfig),
    /// Spawned: chain lightning arcs.
    ChainLightning(ChainLightningConfig),
    /// Fire-and-forget: piercing beam line.
    PiercingBeam(PiercingBeamConfig),
    /// Fire-and-forget: spawn extra bolts.
    SpawnBolts(SpawnBoltsConfig),
    /// Spawned: spawn phantom bolt with limited lifetime.
    SpawnPhantom(SpawnPhantomConfig),
    /// Fire-and-forget: chain bolt redirect.
    ChainBolt(ChainBoltConfig),
    /// Fire-and-forget: mirror protocol bolt duplication.
    MirrorProtocol(MirrorConfig),
    /// Spawned: tether beam damage link.
    TetherBeam(TetherBeamConfig),
    /// Spawned: gravity well pulling bolts.
    GravityWell(GravityWellConfig),
    /// Fire-and-forget: lose a life.
    LoseLife(LoseLifeConfig),
    /// Fire-and-forget: add time penalty.
    TimePenalty(TimePenaltyConfig),
    /// Fire-and-forget: kill entity.
    Die(DieConfig),
    /// Stateful: circuit breaker counter.
    CircuitBreaker(CircuitBreakerConfig),
    /// Stateful: entropy engine counter.
    EntropyEngine(EntropyConfig),
    /// Fire-and-forget: pick a random effect.
    RandomEffect(RandomEffectConfig),
}

/// Widening conversion: every `ReversibleEffectType` variant has a
/// corresponding `EffectType` variant with the same config.
impl From<ReversibleEffectType> for EffectType {
    fn from(reversible: ReversibleEffectType) -> Self {
        match reversible {
            ReversibleEffectType::SpeedBoost(c) => Self::SpeedBoost(c),
            ReversibleEffectType::SizeBoost(c) => Self::SizeBoost(c),
            ReversibleEffectType::DamageBoost(c) => Self::DamageBoost(c),
            ReversibleEffectType::BumpForce(c) => Self::BumpForce(c),
            ReversibleEffectType::QuickStop(c) => Self::QuickStop(c),
            ReversibleEffectType::FlashStep(c) => Self::FlashStep(c),
            ReversibleEffectType::Piercing(c) => Self::Piercing(c),
            ReversibleEffectType::Vulnerable(c) => Self::Vulnerable(c),
            ReversibleEffectType::RampingDamage(c) => Self::RampingDamage(c),
            ReversibleEffectType::Attraction(c) => Self::Attraction(c),
            ReversibleEffectType::Anchor(c) => Self::Anchor(c),
            ReversibleEffectType::Pulse(c) => Self::Pulse(c),
            ReversibleEffectType::Shield(c) => Self::Shield(c),
            ReversibleEffectType::SecondWind(c) => Self::SecondWind(c),
            ReversibleEffectType::CircuitBreaker(c) => Self::CircuitBreaker(c),
            ReversibleEffectType::EntropyEngine(c) => Self::EntropyEngine(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;

    // ================================================================
    // Behavior 24: ReversibleEffectType widens to EffectType
    // ================================================================

    #[test]
    fn reversible_speed_boost_converts_to_effect_type_speed_boost() {
        let reversible = ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        let widened = EffectType::from(reversible);
        assert_eq!(
            widened,
            EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            "SpeedBoost config should be preserved through widening conversion"
        );
    }

    #[test]
    fn reversible_circuit_breaker_converts_to_effect_type_circuit_breaker() {
        let reversible = ReversibleEffectType::CircuitBreaker(CircuitBreakerConfig {
            bumps_required:  5,
            spawn_count:     2,
            inherit:         false,
            shockwave_range: OrderedFloat(64.0),
            shockwave_speed: OrderedFloat(200.0),
        });
        let widened = EffectType::from(reversible);
        assert_eq!(
            widened,
            EffectType::CircuitBreaker(CircuitBreakerConfig {
                bumps_required:  5,
                spawn_count:     2,
                inherit:         false,
                shockwave_range: OrderedFloat(64.0),
                shockwave_speed: OrderedFloat(200.0),
            }),
            "CircuitBreaker config should be preserved through widening conversion"
        );
    }
}
