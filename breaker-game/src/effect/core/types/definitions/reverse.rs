//! [`EffectKind::reverse`] dispatch -- routes each variant to its per-module `reverse()` function.

use bevy::prelude::*;

use super::enums::EffectKind;

impl EffectKind {
    /// Reverse this effect on the given entity. Dispatches to the per-module `reverse()` function.
    pub(crate) fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shockwave { .. } => {
                crate::effect::effects::shockwave::reverse(entity, source_chip, world);
            }
            Self::SpeedBoost { multiplier } => {
                crate::effect::effects::speed_boost::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::DamageBoost(v) => {
                crate::effect::effects::damage_boost::reverse(entity, *v, source_chip, world);
            }
            Self::Vulnerable { multiplier } => {
                crate::effect::effects::vulnerable::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::Piercing(v) => {
                crate::effect::effects::piercing::reverse(entity, *v, source_chip, world);
            }
            Self::SizeBoost(v) => {
                crate::effect::effects::size_boost::reverse(entity, *v, source_chip, world);
            }
            Self::BumpForce(v) => {
                crate::effect::effects::bump_force::reverse(entity, *v, source_chip, world);
            }
            Self::Attraction {
                attraction_type,
                force,
                max_force,
            } => {
                crate::effect::effects::attraction::reverse(
                    entity,
                    *attraction_type,
                    *force,
                    *max_force,
                    source_chip,
                    world,
                );
            }
            Self::LoseLife => {
                crate::effect::effects::life_lost::reverse(entity, source_chip, world);
            }
            Self::TimePenalty { seconds } => {
                crate::effect::effects::time_penalty::reverse(entity, *seconds, source_chip, world);
            }
            _ => self.reverse_aoe_and_spawn(entity, source_chip, world),
        }
    }

    /// Reverse AOE, spawn, and utility effects -- extracted from [`reverse`] for line count.
    fn reverse_aoe_and_spawn(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => crate::effect::effects::spawn_bolts::reverse(
                entity,
                *count,
                *lifespan,
                *inherit,
                source_chip,
                world,
            ),
            Self::ChainBolt { tether_distance } => {
                crate::effect::effects::chain_bolt::reverse(
                    entity,
                    *tether_distance,
                    source_chip,
                    world,
                );
            }
            Self::Shield { .. } => {
                crate::effect::effects::shield::reverse(entity, source_chip, world);
            }
            Self::ChainLightning { .. } => {
                crate::effect::effects::chain_lightning::reverse(entity, source_chip, world);
            }
            Self::PiercingBeam { .. } => {
                crate::effect::effects::piercing_beam::reverse(entity, source_chip, world);
            }
            Self::Pulse { .. } => {
                crate::effect::effects::pulse::reverse(entity, source_chip, world);
            }
            _ => self.reverse_utility(entity, source_chip, world),
        }
    }

    /// Reverse utility and persistent effects -- extracted from [`reverse_aoe_and_spawn`].
    fn reverse_utility(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::SecondWind => {
                crate::effect::effects::second_wind::reverse(entity, source_chip, world);
            }
            Self::SpawnPhantom { .. } => {
                crate::effect::effects::spawn_phantom::reverse(entity, source_chip, world);
            }
            Self::GravityWell { .. } => {
                crate::effect::effects::gravity_well::reverse(entity, source_chip, world);
            }
            Self::RandomEffect(pool) => {
                crate::effect::effects::random_effect::reverse(entity, pool, source_chip, world);
            }
            Self::EntropyEngine { .. } => {
                crate::effect::effects::entropy_engine::reverse(entity, source_chip, world);
            }
            Self::RampingDamage { .. } => {
                crate::effect::effects::ramping_damage::reverse(entity, source_chip, world);
            }
            Self::Explode { .. } => {
                crate::effect::effects::explode::reverse(entity, source_chip, world);
            }
            _ => self.reverse_breaker_effects(entity, source_chip, world),
        }
    }

    /// Reverse breaker-targeted utility effects -- extracted from [`reverse_utility`] for line count.
    fn reverse_breaker_effects(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::QuickStop { multiplier } => {
                crate::effect::effects::quick_stop::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::TetherBeam { damage_mult, chain } => {
                crate::effect::effects::tether_beam::reverse(
                    entity,
                    *damage_mult,
                    *chain,
                    source_chip,
                    world,
                );
            }
            Self::MirrorProtocol { inherit } => {
                crate::effect::effects::mirror_protocol::reverse(
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
                crate::effect::effects::anchor::reverse(
                    entity,
                    *bump_force_multiplier,
                    *perfect_window_multiplier,
                    *plant_delay,
                    source_chip,
                    world,
                );
            }
            Self::FlashStep => {
                crate::effect::effects::flash_step::reverse(entity, source_chip, world);
            }
            Self::CircuitBreaker { .. } => {
                crate::effect::effects::circuit_breaker::reverse(entity, source_chip, world);
            }
            _ => {}
        }
    }
}
