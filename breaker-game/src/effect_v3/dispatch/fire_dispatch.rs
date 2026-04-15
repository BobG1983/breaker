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
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::*;
    use crate::{
        effect_v3::{
            effects::{
                AnchorConfig, AttractionConfig, BumpForceConfig, ChainBoltConfig,
                ChainLightningConfig, CircuitBreakerConfig, DamageBoostConfig, DieConfig,
                EntropyConfig, ExplodeConfig, FlashStepConfig, GravityWellConfig, LoseLifeConfig,
                MirrorConfig, PiercingBeamConfig, PiercingConfig, PulseConfig, QuickStopConfig,
                RampingDamageConfig, RandomEffectConfig, SecondWindConfig, ShieldConfig,
                ShockwaveConfig, SizeBoostConfig, SpawnBoltsConfig, SpawnPhantomConfig,
                SpeedBoostConfig, TetherBeamConfig, TimePenaltyConfig, VulnerableConfig,
            },
            stacking::EffectStack,
            types::AttractionType,
        },
        shared::{PlayfieldConfig, rng::GameRng},
    };

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

    fn passive_effect_types() -> Vec<EffectType> {
        vec![
            EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }),
            EffectType::SizeBoost(SizeBoostConfig {
                multiplier: OrderedFloat(1.2),
            }),
            EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            }),
            EffectType::BumpForce(BumpForceConfig {
                multiplier: OrderedFloat(1.3),
            }),
            EffectType::QuickStop(QuickStopConfig {
                multiplier: OrderedFloat(1.1),
            }),
            EffectType::FlashStep(FlashStepConfig {}),
            EffectType::Piercing(PiercingConfig { charges: 3 }),
            EffectType::Vulnerable(VulnerableConfig {
                multiplier: OrderedFloat(1.5),
            }),
            EffectType::RampingDamage(RampingDamageConfig {
                increment: OrderedFloat(0.5),
            }),
            EffectType::Attraction(AttractionConfig {
                attraction_type: AttractionType::Cell,
                force:           OrderedFloat(100.0),
                max_force:       None,
            }),
            EffectType::Anchor(AnchorConfig {
                bump_force_multiplier:     OrderedFloat(2.0),
                perfect_window_multiplier: OrderedFloat(1.5),
                plant_delay:               OrderedFloat(0.5),
            }),
        ]
    }

    fn active_effect_types() -> Vec<EffectType> {
        vec![
            EffectType::Pulse(PulseConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(16.0),
                stacks:          1,
                speed:           OrderedFloat(200.0),
                interval:        OrderedFloat(1.0),
            }),
            EffectType::Shield(ShieldConfig {
                duration:        OrderedFloat(5.0),
                reflection_cost: OrderedFloat(1.0),
            }),
            EffectType::SecondWind(SecondWindConfig {}),
            EffectType::Shockwave(ShockwaveConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(16.0),
                stacks:          1,
                speed:           OrderedFloat(200.0),
            }),
            EffectType::Explode(ExplodeConfig {
                range:  OrderedFloat(48.0),
                damage: OrderedFloat(10.0),
            }),
            EffectType::ChainLightning(ChainLightningConfig {
                arcs:        3,
                range:       OrderedFloat(80.0),
                damage_mult: OrderedFloat(1.0),
                arc_speed:   OrderedFloat(400.0),
            }),
            EffectType::PiercingBeam(PiercingBeamConfig {
                damage_mult: OrderedFloat(1.0),
                width:       OrderedFloat(8.0),
            }),
            EffectType::SpawnBolts(SpawnBoltsConfig {
                count:    2,
                lifespan: None,
                inherit:  false,
            }),
            EffectType::SpawnPhantom(SpawnPhantomConfig {
                duration:   OrderedFloat(5.0),
                max_active: 3,
            }),
            EffectType::ChainBolt(ChainBoltConfig {
                tether_distance: OrderedFloat(100.0),
            }),
            EffectType::MirrorProtocol(MirrorConfig { inherit: false }),
            EffectType::TetherBeam(TetherBeamConfig {
                damage_mult: OrderedFloat(1.0),
                chain:       false,
                width:       OrderedFloat(4.0),
            }),
            EffectType::GravityWell(GravityWellConfig {
                strength: OrderedFloat(50.0),
                duration: OrderedFloat(3.0),
                radius:   OrderedFloat(64.0),
                max:      2,
            }),
            EffectType::LoseLife(LoseLifeConfig {}),
            EffectType::TimePenalty(TimePenaltyConfig {
                seconds: OrderedFloat(5.0),
            }),
            EffectType::Die(DieConfig {}),
            EffectType::CircuitBreaker(CircuitBreakerConfig {
                bumps_required:  5,
                spawn_count:     2,
                inherit:         false,
                shockwave_range: OrderedFloat(64.0),
                shockwave_speed: OrderedFloat(200.0),
            }),
            EffectType::EntropyEngine(EntropyConfig {
                max_effects: 3,
                pool:        vec![],
            }),
            EffectType::RandomEffect(RandomEffectConfig { pool: vec![] }),
        ]
    }

    /// Returns one instance of every `EffectType` variant with sensible test values.
    fn all_effect_types() -> Vec<EffectType> {
        let mut types = passive_effect_types();
        types.extend(active_effect_types());
        types
    }

    #[test]
    fn fire_dispatch_does_not_panic_for_any_effect_type_variant() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        world.insert_resource(PlayfieldConfig::default());
        world.init_resource::<rantzsoft_physics2d::resources::CollisionQuadtree>();

        let types = all_effect_types();
        assert_eq!(
            types.len(),
            30,
            "update all_effect_types when EffectType gains variants"
        );
        for effect in types {
            let entity = world
                .spawn((
                    Position2D(Vec2::new(100.0, 200.0)),
                    Velocity2D(Vec2::new(0.0, 300.0)),
                ))
                .id();

            fire_dispatch(&effect, entity, "smoke_test", &mut world);
        }
        // If we reach here, no variant panicked.
    }
}
