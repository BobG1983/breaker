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
            } => super::super::super::super::effects::shockwave::fire(
                entity,
                *base_range,
                *range_per_level,
                *stacks,
                *speed,
                source_chip,
                world,
            ),
            Self::SpeedBoost { multiplier } => {
                super::super::super::super::effects::speed_boost::fire(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::DamageBoost(v) => {
                super::super::super::super::effects::damage_boost::fire(
                    entity,
                    *v,
                    source_chip,
                    world,
                );
            }
            Self::Piercing(v) => {
                super::super::super::super::effects::piercing::fire(entity, *v, source_chip, world);
            }
            Self::SizeBoost(v) => {
                super::super::super::super::effects::size_boost::fire(
                    entity,
                    *v,
                    source_chip,
                    world,
                );
            }
            Self::BumpForce(v) => {
                super::super::super::super::effects::bump_force::fire(
                    entity,
                    *v,
                    source_chip,
                    world,
                );
            }
            Self::Attraction {
                attraction_type,
                force,
                max_force,
            } => {
                super::super::super::super::effects::attraction::fire(
                    entity,
                    *attraction_type,
                    *force,
                    *max_force,
                    source_chip,
                    world,
                );
            }
            Self::LoseLife => {
                super::super::super::super::effects::life_lost::fire(entity, source_chip, world);
            }
            Self::TimePenalty { seconds } => {
                super::super::super::super::effects::time_penalty::fire(
                    entity,
                    *seconds,
                    source_chip,
                    world,
                );
            }
            Self::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => {
                super::super::super::super::effects::spawn_bolts::fire(
                    entity,
                    *count,
                    *lifespan,
                    *inherit,
                    source_chip,
                    world,
                );
            }
            Self::ChainBolt { tether_distance } => {
                super::super::super::super::effects::chain_bolt::fire(
                    entity,
                    *tether_distance,
                    source_chip,
                    world,
                );
            }
            _ => self.fire_aoe_and_spawn(entity, source_chip, world),
        }
    }

    /// Fire AOE, spawn, and utility effects -- extracted from [`fire`] for line count.
    fn fire_aoe_and_spawn(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shield { stacks } => {
                super::super::super::super::effects::shield::fire(
                    entity,
                    *stacks,
                    source_chip,
                    world,
                );
            }
            Self::ChainLightning {
                arcs,
                range,
                damage_mult,
                arc_speed,
            } => super::super::super::super::effects::chain_lightning::fire(
                entity,
                *arcs,
                *range,
                *damage_mult,
                *arc_speed,
                source_chip,
                world,
            ),
            Self::PiercingBeam { damage_mult, width } => {
                super::super::super::super::effects::piercing_beam::fire(
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
            } => super::super::super::super::effects::pulse::fire(
                entity,
                super::super::super::super::effects::pulse::PulseEmitter {
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
                super::super::super::super::effects::second_wind::fire(entity, source_chip, world);
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
            } => super::super::super::super::effects::spawn_phantom::fire(
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
            } => super::super::super::super::effects::gravity_well::fire(
                entity,
                *strength,
                *duration,
                *radius,
                *max,
                source_chip,
                world,
            ),
            Self::RandomEffect(pool) => {
                super::super::super::super::effects::random_effect::fire(
                    entity,
                    pool,
                    source_chip,
                    world,
                );
            }
            Self::EntropyEngine { max_effects, pool } => {
                super::super::super::super::effects::entropy_engine::fire(
                    entity,
                    *max_effects,
                    pool,
                    source_chip,
                    world,
                );
            }
            Self::RampingDamage { damage_per_trigger } => {
                super::super::super::super::effects::ramping_damage::fire(
                    entity,
                    *damage_per_trigger,
                    source_chip,
                    world,
                );
            }
            Self::Explode { range, damage } => {
                super::super::super::super::effects::explode::fire(
                    entity,
                    *range,
                    *damage,
                    source_chip,
                    world,
                );
            }
            _ => self.fire_breaker_effects(entity, source_chip, world),
        }
    }

    /// Fire breaker-targeted utility effects -- extracted from [`fire_utility_and_spawn`] for line count.
    fn fire_breaker_effects(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::QuickStop { multiplier } => {
                super::super::super::super::effects::quick_stop::fire(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::TetherBeam { damage_mult, chain } => {
                super::super::super::super::effects::tether_beam::fire(
                    entity,
                    *damage_mult,
                    *chain,
                    source_chip,
                    world,
                );
            }
            Self::MirrorProtocol { inherit } => {
                super::super::super::super::effects::mirror_protocol::fire(
                    entity,
                    *inherit,
                    source_chip,
                    world,
                );
            }
            Self::Anchor {
                bump_force_multiplier,
                perfect_window_multiplier,
                plant_delay,
            } => {
                super::super::super::super::effects::anchor::fire(
                    entity,
                    *bump_force_multiplier,
                    *perfect_window_multiplier,
                    *plant_delay,
                    source_chip,
                    world,
                );
            }
            Self::FlashStep => {
                super::super::super::super::effects::flash_step::fire(entity, source_chip, world);
            }
            Self::CircuitBreaker {
                bumps_required,
                spawn_count,
                inherit,
                shockwave_range,
                shockwave_speed,
            } => {
                use super::super::super::super::effects::circuit_breaker;
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
