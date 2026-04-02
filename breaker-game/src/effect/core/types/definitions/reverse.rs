//! [`EffectKind::reverse`] dispatch -- routes each variant to its per-module `reverse()` function.

use bevy::prelude::*;

use super::enums::EffectKind;

impl EffectKind {
    /// Reverse this effect on the given entity. Dispatches to the per-module `reverse()` function.
    pub(crate) fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shockwave { .. } => {
                super::super::super::super::effects::shockwave::reverse(entity, source_chip, world);
            }
            Self::SpeedBoost { multiplier } => {
                super::super::super::super::effects::speed_boost::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::DamageBoost(v) => {
                super::super::super::super::effects::damage_boost::reverse(
                    entity,
                    *v,
                    source_chip,
                    world,
                );
            }
            Self::Vulnerable { multiplier } => {
                super::super::super::super::effects::vulnerable::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::Piercing(v) => {
                super::super::super::super::effects::piercing::reverse(
                    entity,
                    *v,
                    source_chip,
                    world,
                );
            }
            Self::SizeBoost(v) => {
                super::super::super::super::effects::size_boost::reverse(
                    entity,
                    *v,
                    source_chip,
                    world,
                );
            }
            Self::BumpForce(v) => {
                super::super::super::super::effects::bump_force::reverse(
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
                super::super::super::super::effects::attraction::reverse(
                    entity,
                    *attraction_type,
                    *force,
                    *max_force,
                    source_chip,
                    world,
                );
            }
            Self::LoseLife => {
                super::super::super::super::effects::life_lost::reverse(entity, source_chip, world);
            }
            Self::TimePenalty { seconds } => {
                super::super::super::super::effects::time_penalty::reverse(
                    entity,
                    *seconds,
                    source_chip,
                    world,
                );
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
            } => super::super::super::super::effects::spawn_bolts::reverse(
                entity,
                *count,
                *lifespan,
                *inherit,
                source_chip,
                world,
            ),
            Self::ChainBolt { tether_distance } => {
                super::super::super::super::effects::chain_bolt::reverse(
                    entity,
                    *tether_distance,
                    source_chip,
                    world,
                );
            }
            Self::Shield { .. } => {
                super::super::super::super::effects::shield::reverse(entity, source_chip, world);
            }
            Self::ChainLightning { .. } => {
                super::super::super::super::effects::chain_lightning::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::PiercingBeam { .. } => {
                super::super::super::super::effects::piercing_beam::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::Pulse { .. } => {
                super::super::super::super::effects::pulse::reverse(entity, source_chip, world);
            }
            _ => self.reverse_utility(entity, source_chip, world),
        }
    }

    /// Reverse utility and persistent effects -- extracted from [`reverse_aoe_and_spawn`].
    fn reverse_utility(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::SecondWind => {
                super::super::super::super::effects::second_wind::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::SpawnPhantom { .. } => {
                super::super::super::super::effects::spawn_phantom::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::GravityWell { .. } => {
                super::super::super::super::effects::gravity_well::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::RandomEffect(pool) => {
                super::super::super::super::effects::random_effect::reverse(
                    entity,
                    pool,
                    source_chip,
                    world,
                );
            }
            Self::EntropyEngine { .. } => {
                super::super::super::super::effects::entropy_engine::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::RampingDamage { .. } => {
                super::super::super::super::effects::ramping_damage::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::Explode { .. } => {
                super::super::super::super::effects::explode::reverse(entity, source_chip, world);
            }
            _ => self.reverse_breaker_effects(entity, source_chip, world),
        }
    }

    /// Reverse breaker-targeted utility effects -- extracted from [`reverse_utility`] for line count.
    fn reverse_breaker_effects(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::QuickStop { multiplier } => {
                super::super::super::super::effects::quick_stop::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::TetherBeam { damage_mult, chain } => {
                super::super::super::super::effects::tether_beam::reverse(
                    entity,
                    *damage_mult,
                    *chain,
                    source_chip,
                    world,
                );
            }
            Self::MirrorProtocol { inherit } => {
                super::super::super::super::effects::mirror_protocol::reverse(
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
                super::super::super::super::effects::anchor::reverse(
                    entity,
                    *bump_force_multiplier,
                    *perfect_window_multiplier,
                    *plant_delay,
                    source_chip,
                    world,
                );
            }
            Self::FlashStep => {
                super::super::super::super::effects::flash_step::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            Self::CircuitBreaker { .. } => {
                super::super::super::super::effects::circuit_breaker::reverse(
                    entity,
                    source_chip,
                    world,
                );
            }
            _ => {}
        }
    }
}
