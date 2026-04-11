//! [`EffectKind::fire`] dispatch -- routes each variant to its per-module `fire()` function.

use bevy::prelude::*;

use super::enums::EffectKind;

impl EffectKind {
    /// Fire this effect on the given entity. Dispatches to the per-module `fire()` function.
    pub(crate) fn fire(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shockwave {
                base_range,
                range_per_level,
                stacks,
                speed,
            } => crate::effect::effects::shockwave::fire(
                entity,
                *base_range,
                *range_per_level,
                *stacks,
                *speed,
                source_chip,
                world,
            ),
            Self::SpeedBoost { multiplier } => {
                crate::effect::effects::speed_boost::fire(entity, *multiplier, source_chip, world);
            }
            Self::DamageBoost(v) => {
                crate::effect::effects::damage_boost::fire(entity, *v, source_chip, world);
            }
            Self::Vulnerable { multiplier } => {
                crate::effect::effects::vulnerable::fire(entity, *multiplier, source_chip, world);
            }
            Self::Piercing(v) => {
                crate::effect::effects::piercing::fire(entity, *v, source_chip, world);
            }
            Self::SizeBoost(v) => {
                crate::effect::effects::size_boost::fire(entity, *v, source_chip, world);
            }
            Self::BumpForce(v) => {
                crate::effect::effects::bump_force::fire(entity, *v, source_chip, world);
            }
            Self::Attraction {
                attraction_type,
                force,
                max_force,
            } => {
                crate::effect::effects::attraction::fire(
                    entity,
                    *attraction_type,
                    *force,
                    *max_force,
                    source_chip,
                    world,
                );
            }
            Self::LoseLife => {
                crate::effect::effects::life_lost::fire(entity, source_chip, world);
            }
            Self::TimePenalty { seconds } => {
                crate::effect::effects::time_penalty::fire(entity, *seconds, source_chip, world);
            }
            _ => self.fire_aoe_and_spawn(entity, source_chip, world),
        }
    }

    /// Fire AOE, spawn, and utility effects -- extracted from [`fire`] for line count.
    fn fire_aoe_and_spawn(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => {
                crate::effect::effects::spawn_bolts::fire(
                    entity,
                    *count,
                    *lifespan,
                    *inherit,
                    source_chip,
                    world,
                );
            }
            Self::ChainBolt { tether_distance } => {
                crate::effect::effects::chain_bolt::fire(
                    entity,
                    *tether_distance,
                    source_chip,
                    world,
                );
            }
            Self::Shield {
                duration,
                reflection_cost,
            } => {
                crate::effect::effects::shield::fire(
                    entity,
                    *duration,
                    *reflection_cost,
                    source_chip,
                    world,
                );
            }
            Self::ChainLightning {
                arcs,
                range,
                damage_mult,
                arc_speed,
            } => crate::effect::effects::chain_lightning::fire(
                entity,
                *arcs,
                *range,
                *damage_mult,
                *arc_speed,
                source_chip,
                world,
            ),
            Self::PiercingBeam { damage_mult, width } => {
                crate::effect::effects::piercing_beam::fire(
                    entity,
                    *damage_mult,
                    *width,
                    source_chip,
                    world,
                );
            }
            Self::Pulse {
                base_range,
                range_per_level,
                stacks,
                speed,
                interval,
            } => crate::effect::effects::pulse::fire(
                entity,
                crate::effect::effects::pulse::PulseEmitter {
                    base_range: *base_range,
                    range_per_level: *range_per_level,
                    stacks: *stacks,
                    speed: *speed,
                    interval: *interval,
                    timer: 0.0,
                },
                source_chip,
                world,
            ),
            Self::SecondWind => {
                crate::effect::effects::second_wind::fire(entity, source_chip, world);
            }
            _ => self.fire_utility_and_spawn(entity, source_chip, world),
        }
    }

    /// Fire utility, random, and spawn effects -- extracted from [`fire_aoe_and_spawn`] for line count.
    fn fire_utility_and_spawn(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::SpawnPhantom {
                duration,
                max_active,
            } => crate::effect::effects::spawn_phantom::fire(
                entity,
                *duration,
                *max_active,
                source_chip,
                world,
            ),
            Self::GravityWell {
                strength,
                duration,
                radius,
                max,
            } => crate::effect::effects::gravity_well::fire(
                entity,
                *strength,
                *duration,
                *radius,
                *max,
                source_chip,
                world,
            ),
            Self::RandomEffect(pool) => {
                crate::effect::effects::random_effect::fire(entity, pool, source_chip, world);
            }
            Self::EntropyEngine { max_effects, pool } => {
                crate::effect::effects::entropy_engine::fire(
                    entity,
                    *max_effects,
                    pool,
                    source_chip,
                    world,
                );
            }
            Self::RampingDamage { damage_per_trigger } => {
                crate::effect::effects::ramping_damage::fire(
                    entity,
                    *damage_per_trigger,
                    source_chip,
                    world,
                );
            }
            Self::Explode { range, damage } => {
                crate::effect::effects::explode::fire(entity, *range, *damage, source_chip, world);
            }
            _ => self.fire_breaker_effects(entity, source_chip, world),
        }
    }

    /// Fire breaker-targeted utility effects -- extracted from [`fire_utility_and_spawn`] for line count.
    fn fire_breaker_effects(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::QuickStop { multiplier } => {
                crate::effect::effects::quick_stop::fire(entity, *multiplier, source_chip, world);
            }
            Self::TetherBeam { damage_mult, chain } => {
                crate::effect::effects::tether_beam::fire(
                    entity,
                    *damage_mult,
                    *chain,
                    source_chip,
                    world,
                );
            }
            Self::MirrorProtocol { inherit } => {
                crate::effect::effects::mirror_protocol::fire(entity, *inherit, source_chip, world);
            }
            Self::Anchor {
                bump_force_multiplier,
                perfect_window_multiplier,
                plant_delay,
            } => {
                crate::effect::effects::anchor::fire(
                    entity,
                    *bump_force_multiplier,
                    *perfect_window_multiplier,
                    *plant_delay,
                    source_chip,
                    world,
                );
            }
            Self::FlashStep => {
                crate::effect::effects::flash_step::fire(entity, source_chip, world);
            }
            Self::CircuitBreaker {
                bumps_required,
                spawn_count,
                inherit,
                shockwave_range,
                shockwave_speed,
            } => {
                use crate::effect::effects::circuit_breaker;
                let config = circuit_breaker::CircuitBreakerConfig {
                    bumps_required: *bumps_required,
                    spawn_count: *spawn_count,
                    inherit: *inherit,
                    shockwave_range: *shockwave_range,
                    shockwave_speed: *shockwave_speed,
                };
                circuit_breaker::fire(entity, &config, source_chip, world);
            }
            _ => {
                // Stat effects (SpeedBoost, DamageBoost, etc.) handled in primary fire() match.
                // If this arm is reached with an unhandled variant, it's a programmer error.
            }
        }
    }
}
