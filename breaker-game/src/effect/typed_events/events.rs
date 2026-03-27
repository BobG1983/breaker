//! Typed per-effect observer events and dispatch helpers.
//!
//! Each event struct now lives in its corresponding effect handler file.
//! This module re-exports them for backward compatibility and contains
//! the dispatch helpers that convert `Effect` values into typed events.

use bevy::prelude::*;
#[cfg(debug_assertions)]
use tracing::warn;

use crate::effect::definition::EffectTarget;
// ===========================================================================
// Re-exports — passive effect events (canonical location: effects/<name>.rs)
// ===========================================================================
pub(crate) use crate::effect::effects::attraction::AttractionApplied;
// ===========================================================================
// Re-exports — triggered effect events (canonical location: effects/<name>.rs)
// ===========================================================================
pub(crate) use crate::effect::effects::chain_bolt::ChainBoltFired;
pub(crate) use crate::effect::effects::{
    bolt_size_boost::SizeBoostApplied,
    bolt_speed_boost::SpeedBoostApplied,
    bump_force_boost::BumpForceApplied,
    chain_hit::ChainHitApplied,
    chain_lightning::ChainLightningFired,
    damage_boost::DamageBoostApplied,
    entropy_engine::EntropyEngineFired,
    gravity_well::GravityWellFired,
    life_lost::LoseLifeFired,
    multi_bolt::MultiBoltFired,
    piercing::PiercingApplied,
    piercing_beam::PiercingBeamFired,
    pulse::PulseFired,
    ramping_damage::RampingDamageApplied,
    random_effect::RandomEffectFired,
    second_wind::SecondWindFired,
    shield::ShieldFired,
    shockwave::ShockwaveFired,
    spawn_bolt::SpawnBoltsFired,
    spawn_phantom::SpawnPhantomFired,
    speed_boost::SpeedBoostFired,
    tilt_control_boost::TiltControlApplied,
    time_penalty::TimePenaltyFired,
};
#[cfg(test)]
pub(crate) use crate::effect::effects::spawn_bolt::SpawnBoltFired;

// ===========================================================================
// Bridge dispatch — converts Effect -> typed event
// ===========================================================================

/// Converts a resolved `Effect` leaf into the appropriate typed event trigger.
///
/// Called by bridge systems after `evaluate_node()` returns `Fire(effect)`.
pub(crate) fn fire_typed_event(
    effect: crate::effect::definition::Effect,
    targets: Vec<EffectTarget>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use crate::effect::definition::Effect;

    match effect {
        // -- Bolt combat effects --
        effect @ (Effect::Shockwave { .. }
        | Effect::SpeedBoost { .. }
        | Effect::ChainBolt { .. }
        | Effect::MultiBolt { .. }
        | Effect::ChainLightning { .. }
        | Effect::PiercingBeam { .. }
        | Effect::Pulse { .. }) => {
            fire_bolt_effect(effect, targets, source_chip, commands);
        }

        // -- Life, penalty, spawn, and defensive effects --
        effect @ (Effect::LoseLife
        | Effect::TimePenalty { .. }
        | Effect::SpawnBolts { .. }
        | Effect::SpawnPhantom { .. }
        | Effect::Shield { .. }
        | Effect::GravityWell { .. }
        | Effect::SecondWind { .. }) => {
            fire_global_effect(effect, targets, source_chip, commands);
        }

        // -- Pool / random effects --
        effect @ (Effect::RandomEffect(_) | Effect::EntropyEngine { .. }) => {
            fire_pool_effect(effect, targets, source_chip, commands);
        }

        // Passive-only effects should not be fired via bridge dispatch.
        // If they end up here, it's a data error — log and skip.
        _effect @ (Effect::Piercing(_)
        | Effect::DamageBoost(_)
        | Effect::ChainHit(_)
        | Effect::SizeBoost(..)
        | Effect::Attraction(..)
        | Effect::BumpForce(_)
        | Effect::TiltControl(_)
        | Effect::RampingDamage { .. }) => {
            #[cfg(debug_assertions)]
            {
                warn!(
                    "fire_typed_event called with passive-only effect {_effect:?} — should use fire_passive_event"
                );
            }
        }
    }
}

/// Dispatches bolt combat effects (shockwave, speed boost, chain bolt, etc.).
fn fire_bolt_effect(
    effect: crate::effect::definition::Effect,
    targets: Vec<EffectTarget>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use crate::effect::definition::Effect;

    match effect {
        Effect::Shockwave {
            base_range,
            range_per_level,
            stacks,
            speed,
        } => {
            commands.trigger(ShockwaveFired {
                base_range,
                range_per_level,
                stacks,
                speed,
                targets,
                source_chip,
            });
        }
        Effect::SpeedBoost { multiplier } => {
            commands.trigger(SpeedBoostFired {
                multiplier,
                targets,
            });
        }
        Effect::ChainBolt { tether_distance } => {
            commands.trigger(ChainBoltFired {
                tether_distance,
                targets,
                source_chip,
            });
        }
        Effect::MultiBolt {
            base_count,
            count_per_level,
            stacks,
        } => {
            commands.trigger(MultiBoltFired {
                base_count,
                count_per_level,
                stacks,
                source_chip,
            });
        }
        Effect::ChainLightning {
            arcs,
            range,
            damage_mult,
        } => {
            commands.trigger(ChainLightningFired {
                arcs,
                range,
                damage_mult,
                targets,
                source_chip,
            });
        }
        Effect::PiercingBeam { damage_mult, width } => {
            commands.trigger(PiercingBeamFired {
                damage_mult,
                width,
                targets,
                source_chip,
            });
        }
        Effect::Pulse {
            base_range,
            range_per_level,
            stacks,
            speed,
        } => {
            commands.trigger(PulseFired {
                base_range,
                range_per_level,
                stacks,
                speed,
                source_chip,
            });
        }
        _ => {}
    }
}

/// Dispatches life, penalty, spawn, and defensive effects.
fn fire_global_effect(
    effect: crate::effect::definition::Effect,
    targets: Vec<EffectTarget>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use crate::effect::definition::Effect;

    match effect {
        Effect::LoseLife => {
            commands.trigger(LoseLifeFired {});
        }
        Effect::TimePenalty { seconds } => {
            commands.trigger(TimePenaltyFired {
                seconds,
            });
        }
        Effect::SpawnBolts {
            count,
            lifespan,
            inherit,
        } => {
            commands.trigger(SpawnBoltsFired {
                count,
                lifespan,
                inherit,
                source_chip,
            });
        }
        Effect::SpawnPhantom {
            duration,
            max_active,
        } => {
            commands.trigger(SpawnPhantomFired {
                duration,
                max_active,
                targets,
            });
        }
        Effect::Shield {
            base_duration,
            duration_per_level,
            stacks,
        } => {
            commands.trigger(ShieldFired {
                base_duration,
                duration_per_level,
                stacks,
            });
        }
        Effect::GravityWell {
            strength: _,
            duration: _,
            radius: _,
            max,
        } => {
            commands.trigger(GravityWellFired {
                max,
                targets,
            });
        }
        Effect::SecondWind { invuln_secs: _ } => {
            commands.trigger(SecondWindFired {});
        }
        _ => {}
    }
}

/// Dispatches pool / random effects (`RandomEffect`, `EntropyEngine`).
fn fire_pool_effect(
    effect: crate::effect::definition::Effect,
    targets: Vec<EffectTarget>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use crate::effect::definition::Effect;

    match effect {
        Effect::RandomEffect(pool) => {
            commands.trigger(RandomEffectFired {
                pool: pool.into_iter().collect(),
                targets,
                source_chip,
            });
        }
        Effect::EntropyEngine { threshold, pool } => {
            commands.trigger(EntropyEngineFired {
                threshold,
                pool: pool.into_iter().collect(),
                targets,
                source_chip,
            });
        }
        _ => {}
    }
}

/// Converts a resolved passive `Effect` leaf into the appropriate typed passive event trigger.
///
/// Called by `dispatch_chip_effects` after extracting leaf effects from `OnSelected` nodes.
pub(crate) fn fire_passive_event(
    effect: crate::effect::definition::Effect,
    max_stacks: u32,
    _chip_name: String,
    commands: &mut Commands,
) {
    use crate::effect::definition::Effect;

    match effect {
        Effect::Piercing(per_stack) => {
            commands.trigger(PiercingApplied {
                per_stack,
                max_stacks,
            });
        }
        Effect::DamageBoost(per_stack) => {
            commands.trigger(DamageBoostApplied {
                per_stack,
                max_stacks,
            });
        }
        Effect::SpeedBoost { multiplier } => {
            commands.trigger(SpeedBoostApplied {
                multiplier,
                max_stacks,
            });
        }
        Effect::ChainHit(per_stack) => {
            commands.trigger(ChainHitApplied {
                per_stack,
                max_stacks,
            });
        }
        Effect::SizeBoost(per_stack) => {
            commands.trigger(SizeBoostApplied {
                per_stack,
                max_stacks,
            });
        }
        Effect::Attraction(attraction_type, per_stack) => {
            commands.trigger(AttractionApplied {
                attraction_type,
                per_stack,
                max_stacks,
            });
        }
        Effect::BumpForce(per_stack) => {
            commands.trigger(BumpForceApplied {
                per_stack,
                max_stacks,
            });
        }
        Effect::TiltControl(per_stack) => {
            commands.trigger(TiltControlApplied {
                per_stack,
                max_stacks,
            });
        }
        Effect::RampingDamage { bonus_per_hit } => {
            commands.trigger(RampingDamageApplied {
                bonus_per_hit,
            });
        }
        // Triggered-only effects should not be fired via passive dispatch.
        _ => {
            #[cfg(debug_assertions)]
            {
                warn!(
                    "fire_passive_event called with non-passive effect {effect:?} — should use fire_typed_event"
                );
            }
        }
    }
}
