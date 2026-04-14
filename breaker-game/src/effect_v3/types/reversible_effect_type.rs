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

use super::EffectType;

/// Narrowing conversion: succeeds for the 16 `EffectType` variants that
/// have a corresponding `ReversibleEffectType` variant. Returns `Err(())`
/// for non-reversible variants.
impl TryFrom<EffectType> for ReversibleEffectType {
    type Error = ();

    fn try_from(effect: EffectType) -> Result<Self, Self::Error> {
        match effect {
            EffectType::SpeedBoost(c) => Ok(Self::SpeedBoost(c)),
            EffectType::SizeBoost(c) => Ok(Self::SizeBoost(c)),
            EffectType::DamageBoost(c) => Ok(Self::DamageBoost(c)),
            EffectType::BumpForce(c) => Ok(Self::BumpForce(c)),
            EffectType::QuickStop(c) => Ok(Self::QuickStop(c)),
            EffectType::FlashStep(c) => Ok(Self::FlashStep(c)),
            EffectType::Piercing(c) => Ok(Self::Piercing(c)),
            EffectType::Vulnerable(c) => Ok(Self::Vulnerable(c)),
            EffectType::RampingDamage(c) => Ok(Self::RampingDamage(c)),
            EffectType::Attraction(c) => Ok(Self::Attraction(c)),
            EffectType::Anchor(c) => Ok(Self::Anchor(c)),
            EffectType::Pulse(c) => Ok(Self::Pulse(c)),
            EffectType::Shield(c) => Ok(Self::Shield(c)),
            EffectType::SecondWind(c) => Ok(Self::SecondWind(c)),
            EffectType::CircuitBreaker(c) => Ok(Self::CircuitBreaker(c)),
            EffectType::EntropyEngine(c) => Ok(Self::EntropyEngine(c)),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;

    // ================================================================
    // Behavior 25: TryFrom<EffectType> for ReversibleEffectType succeeds
    //              for reversible variants
    // ================================================================

    #[test]
    fn try_from_speed_boost_succeeds() {
        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        let result = ReversibleEffectType::try_from(effect);
        assert_eq!(
            result,
            Ok(ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
            "SpeedBoost should narrow successfully"
        );
    }

    #[test]
    fn try_from_circuit_breaker_succeeds() {
        let effect = EffectType::CircuitBreaker(CircuitBreakerConfig {
            bumps_required:  5,
            spawn_count:     2,
            inherit:         false,
            shockwave_range: OrderedFloat(64.0),
            shockwave_speed: OrderedFloat(200.0),
        });
        let result = ReversibleEffectType::try_from(effect);
        assert_eq!(
            result,
            Ok(ReversibleEffectType::CircuitBreaker(CircuitBreakerConfig {
                bumps_required:  5,
                spawn_count:     2,
                inherit:         false,
                shockwave_range: OrderedFloat(64.0),
                shockwave_speed: OrderedFloat(200.0),
            })),
            "CircuitBreaker should narrow successfully"
        );
    }

    // ================================================================
    // Behavior 26: TryFrom<EffectType> for ReversibleEffectType fails
    //              for non-reversible variants
    // ================================================================

    #[test]
    fn try_from_shockwave_fails() {
        let effect = EffectType::Shockwave(ShockwaveConfig {
            base_range:      OrderedFloat(64.0),
            range_per_level: OrderedFloat(16.0),
            stacks:          1,
            speed:           OrderedFloat(200.0),
        });
        let result = ReversibleEffectType::try_from(effect);
        assert_eq!(result, Err(()), "Shockwave is not reversible — should fail");
    }

    #[test]
    fn try_from_explode_fails() {
        let effect = EffectType::Explode(ExplodeConfig {
            range:  OrderedFloat(48.0),
            damage: OrderedFloat(10.0),
        });
        let result = ReversibleEffectType::try_from(effect);
        assert_eq!(result, Err(()), "Explode is not reversible — should fail");
    }
}
