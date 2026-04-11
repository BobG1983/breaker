//! `fire_dispatch` — match `EffectType` variant to `config.fire()` call.

use bevy::prelude::*;

use crate::effect_v3::{traits::Fireable, types::EffectType};

/// Dispatch an `EffectType` to the appropriate config's `fire()` method.
///
/// This is a mechanical match — each arm unwraps the config and calls
/// `config.fire(entity, source, world)`.
pub fn fire_dispatch(effect: &EffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        EffectType::SpeedBoost(config) => config.fire(entity, source, world),
        EffectType::SizeBoost(config) => config.fire(entity, source, world),
        EffectType::DamageBoost(config) => config.fire(entity, source, world),
        EffectType::BumpForce(config) => config.fire(entity, source, world),
        EffectType::QuickStop(config) => config.fire(entity, source, world),
        EffectType::FlashStep(config) => config.fire(entity, source, world),
        EffectType::Piercing(config) => config.fire(entity, source, world),
        EffectType::Vulnerable(config) => config.fire(entity, source, world),
        EffectType::RampingDamage(config) => config.fire(entity, source, world),
        EffectType::Attraction(config) => config.fire(entity, source, world),
        EffectType::Anchor(config) => config.fire(entity, source, world),
        EffectType::Pulse(config) => config.fire(entity, source, world),
        EffectType::Shield(config) => config.fire(entity, source, world),
        EffectType::SecondWind(config) => config.fire(entity, source, world),
        EffectType::Shockwave(config) => config.fire(entity, source, world),
        EffectType::Explode(config) => config.fire(entity, source, world),
        EffectType::ChainLightning(config) => config.fire(entity, source, world),
        EffectType::PiercingBeam(config) => config.fire(entity, source, world),
        EffectType::SpawnBolts(config) => config.fire(entity, source, world),
        EffectType::SpawnPhantom(config) => config.fire(entity, source, world),
        EffectType::ChainBolt(config) => config.fire(entity, source, world),
        EffectType::MirrorProtocol(config) => config.fire(entity, source, world),
        EffectType::TetherBeam(config) => config.fire(entity, source, world),
        EffectType::GravityWell(config) => config.fire(entity, source, world),
        EffectType::LoseLife(config) => config.fire(entity, source, world),
        EffectType::TimePenalty(config) => config.fire(entity, source, world),
        EffectType::Die(config) => config.fire(entity, source, world),
        EffectType::CircuitBreaker(config) => config.fire(entity, source, world),
        EffectType::EntropyEngine(config) => config.fire(entity, source, world),
        EffectType::RandomEffect(config) => config.fire(entity, source, world),
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

    #[test]
    fn fire_dispatch_speed_boost_creates_stack() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });

        fire_dispatch(&effect, entity, "test_chip", &mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }
}
