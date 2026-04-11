//! `reverse_dispatch` — match `ReversibleEffectType` variant to `config.reverse()` call.

use bevy::prelude::*;

use crate::effect_v3::{
    traits::{Fireable, Reversible},
    types::ReversibleEffectType,
};

/// Dispatch a `ReversibleEffectType` to the appropriate config's `reverse()` method.
///
/// This is a mechanical match — each arm unwraps the config and calls
/// `config.reverse(entity, source, world)`.
pub fn reverse_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match effect {
        ReversibleEffectType::SpeedBoost(config) => config.reverse(entity, source, world),
        ReversibleEffectType::SizeBoost(config) => config.reverse(entity, source, world),
        ReversibleEffectType::DamageBoost(config) => config.reverse(entity, source, world),
        ReversibleEffectType::BumpForce(config) => config.reverse(entity, source, world),
        ReversibleEffectType::QuickStop(config) => config.reverse(entity, source, world),
        ReversibleEffectType::FlashStep(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Piercing(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Vulnerable(config) => config.reverse(entity, source, world),
        ReversibleEffectType::RampingDamage(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Attraction(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Anchor(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Pulse(config) => config.reverse(entity, source, world),
        ReversibleEffectType::Shield(config) => config.reverse(entity, source, world),
        ReversibleEffectType::SecondWind(config) => config.reverse(entity, source, world),
        ReversibleEffectType::CircuitBreaker(config) => config.reverse(entity, source, world),
        ReversibleEffectType::EntropyEngine(config) => config.reverse(entity, source, world),
    }
}

/// Helper to fire a `ReversibleEffectType` (all reversible effects are also fireable).
pub fn fire_reversible_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match effect {
        ReversibleEffectType::SpeedBoost(config) => config.fire(entity, source, world),
        ReversibleEffectType::SizeBoost(config) => config.fire(entity, source, world),
        ReversibleEffectType::DamageBoost(config) => config.fire(entity, source, world),
        ReversibleEffectType::BumpForce(config) => config.fire(entity, source, world),
        ReversibleEffectType::QuickStop(config) => config.fire(entity, source, world),
        ReversibleEffectType::FlashStep(config) => config.fire(entity, source, world),
        ReversibleEffectType::Piercing(config) => config.fire(entity, source, world),
        ReversibleEffectType::Vulnerable(config) => config.fire(entity, source, world),
        ReversibleEffectType::RampingDamage(config) => config.fire(entity, source, world),
        ReversibleEffectType::Attraction(config) => config.fire(entity, source, world),
        ReversibleEffectType::Anchor(config) => config.fire(entity, source, world),
        ReversibleEffectType::Pulse(config) => config.fire(entity, source, world),
        ReversibleEffectType::Shield(config) => config.fire(entity, source, world),
        ReversibleEffectType::SecondWind(config) => config.fire(entity, source, world),
        ReversibleEffectType::CircuitBreaker(config) => config.fire(entity, source, world),
        ReversibleEffectType::EntropyEngine(config) => config.fire(entity, source, world),
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack, traits::Fireable};

    #[test]
    fn reverse_dispatch_speed_boost_removes_from_stack() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        };

        // Fire first so there's something to reverse.
        config.fire(entity, "test_chip", &mut world);
        assert_eq!(
            world
                .get::<EffectStack<SpeedBoostConfig>>(entity)
                .unwrap()
                .len(),
            1
        );

        let effect = ReversibleEffectType::SpeedBoost(config);
        reverse_dispatch(&effect, entity, "test_chip", &mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert!(stack.is_empty());
    }
}
