//! Typed per-effect observer events and dispatch helpers.
//!
//! Each event replaces the catchall `EffectFired` / `ChipEffectApplied` with a
//! concrete struct per `Effect` variant. Dispatch helpers convert `TriggerChain`
//! leaves into `Effect` values and fire the corresponding typed events.

use bevy::prelude::*;

use super::definition::Target;

// ===========================================================================
// Triggered effect events (fired by bridge systems)
// ===========================================================================

/// Fired when a shockwave effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ShockwaveFired {
    /// Base radius of the shockwave effect.
    pub base_range: f32,
    /// Additional radius per stack beyond the first.
    pub range_per_level: f32,
    /// Current stack count.
    pub stacks: u32,
    /// Expansion speed in world units per second.
    pub speed: f32,
    /// The bolt entity that triggered the effect, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The chip name that originated this chain, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a lose-life effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct LoseLifeFired {
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a time penalty effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct TimePenaltyFired {
    /// Duration of the penalty in seconds.
    pub seconds: f32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a spawn-bolt effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpawnBoltFired {
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a speed boost effect resolves via a triggered chain.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpeedBoostFired {
    /// Which entity to apply the speed change to.
    pub target: Target,
    /// Multiplier applied to the current velocity magnitude.
    pub multiplier: f32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a chain bolt effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChainBoltFired {
    /// Maximum distance the chain bolt can travel from its anchor.
    pub tether_distance: f32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a multi-bolt effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct MultiBoltFired {
    /// Base number of extra bolts to spawn.
    pub base_count: u32,
    /// Additional bolts per stack beyond the first.
    pub count_per_level: u32,
    /// Current stack count.
    pub stacks: u32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a shield effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ShieldFired {
    /// Base duration in seconds.
    pub base_duration: f32,
    /// Additional duration per stack beyond the first.
    pub duration_per_level: f32,
    /// Current stack count.
    pub stacks: u32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a chain lightning effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChainLightningFired {
    /// Number of arcs from the origin cell.
    pub arcs: u32,
    /// Maximum arc range in world units.
    pub range: f32,
    /// Damage multiplier per arc.
    pub damage_mult: f32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a spawn phantom effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpawnPhantomFired {
    /// How long the phantom persists in seconds.
    pub duration: f32,
    /// Maximum active phantoms at once.
    pub max_active: u32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a piercing beam effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct PiercingBeamFired {
    /// Damage multiplier for the beam.
    pub damage_mult: f32,
    /// Width of the beam in world units.
    pub width: f32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a gravity well effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct GravityWellFired {
    /// Attraction strength.
    pub strength: f32,
    /// Duration in seconds.
    pub duration: f32,
    /// Effect radius in world units.
    pub radius: f32,
    /// Maximum active wells at once.
    pub max: u32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a second wind effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SecondWindFired {
    /// Duration of invulnerability in seconds.
    pub invuln_secs: f32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a timed speed burst effect resolves via a triggered chain.
#[derive(Event, Clone, Debug)]
pub(crate) struct TimedSpeedBurstFired {
    /// Multiplier applied to bolt velocity.
    pub speed_mult: f32,
    /// Duration of the burst in seconds.
    pub duration_secs: f32,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a random effect pool needs to be resolved.
#[derive(Event, Clone, Debug)]
pub(crate) struct RandomEffectFired {
    /// Weighted pool of `TriggerChain` entries to select from.
    pub pool: Vec<(f32, crate::chips::definition::TriggerChain)>,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when an entropy engine effect needs counting and potential resolution.
#[derive(Event, Clone, Debug)]
pub(crate) struct EntropyEngineFired {
    /// Number of cell destructions needed before firing.
    pub threshold: u32,
    /// Weighted pool of `TriggerChain` entries to select from on trigger.
    pub pool: Vec<(f32, crate::chips::definition::TriggerChain)>,
    /// The bolt entity, or `None` for global triggers.
    pub bolt: Option<Entity>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

// ===========================================================================
// Passive effect events (fired by dispatch_chip_effects)
// ===========================================================================

/// Fired when a piercing passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct PiercingApplied {
    /// Piercing count per stack.
    pub per_stack: u32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a damage boost passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct DamageBoostApplied {
    /// Damage boost per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a speed boost passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpeedBoostApplied {
    /// Which entity to apply the speed change to.
    pub target: Target,
    /// Speed multiplier per stack.
    pub multiplier: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a chain hit passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChainHitApplied {
    /// Chain hit count per stack.
    pub per_stack: u32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a size boost passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct SizeBoostApplied {
    /// Which entity to apply the size change to.
    pub target: Target,
    /// Size boost per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when an attraction passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct AttractionApplied {
    /// Attraction force per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a bump force passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct BumpForceApplied {
    /// Bump force increase per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a tilt control passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct TiltControlApplied {
    /// Tilt control sensitivity increase per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a time pressure boost passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct TimePressureBoostApplied {
    /// Speed multiplier applied to bolt velocity when active.
    pub speed_mult: f32,
    /// Timer ratio threshold (remaining/total) below which boost activates.
    pub threshold_pct: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Fired when a ramping damage passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct RampingDamageApplied {
    /// Damage bonus added per cell hit.
    pub bonus_per_hit: f32,
    /// Maximum cumulative damage bonus before capping.
    pub max_bonus: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

// ===========================================================================
// TriggerChain → Effect conversion (for bridge + dispatch_chip_effects call sites)
// ===========================================================================

/// Converts a `TriggerChain` leaf into the corresponding `Effect`.
///
/// # Panics
///
/// Panics if the chain is not a leaf (has a trigger wrapper). This is an
/// invariant violation — callers must only pass leaf variants.
pub(crate) fn trigger_chain_to_effect(
    chain: &crate::chips::definition::TriggerChain,
) -> super::definition::Effect {
    if let Some(effect) = chain_to_passive_effect(chain) {
        return effect;
    }
    if let Some(effect) = chain_to_triggered_effect(chain) {
        return effect;
    }
    unreachable!("trigger_chain_to_effect called on non-leaf TriggerChain: {chain:?}")
}

/// Converts passive leaf `TriggerChain` variants into their `Effect` equivalents.
fn chain_to_passive_effect(
    chain: &crate::chips::definition::TriggerChain,
) -> Option<super::definition::Effect> {
    use super::definition::Effect;
    use crate::chips::definition::TriggerChain;

    Some(match chain {
        TriggerChain::Piercing(n) => Effect::Piercing(*n),
        TriggerChain::DamageBoost(f) => Effect::DamageBoost(*f),
        TriggerChain::SpeedBoost { target, multiplier } => Effect::SpeedBoost {
            target: *target,
            multiplier: *multiplier,
        },
        TriggerChain::ChainHit(n) => Effect::ChainHit(*n),
        TriggerChain::SizeBoost(t, f) => Effect::SizeBoost(*t, *f),
        TriggerChain::Attraction(f) => Effect::Attraction(*f),
        TriggerChain::BumpForce(f) => Effect::BumpForce(*f),
        TriggerChain::TiltControl(f) => Effect::TiltControl(*f),
        TriggerChain::RampingDamage {
            bonus_per_hit,
            max_bonus,
        } => Effect::RampingDamage {
            bonus_per_hit: *bonus_per_hit,
            max_bonus: *max_bonus,
        },
        TriggerChain::TimePressureBoost {
            speed_mult,
            threshold_pct,
        } => Effect::TimePressureBoost {
            speed_mult: *speed_mult,
            threshold_pct: *threshold_pct,
        },
        _ => return None,
    })
}

/// Converts triggered leaf `TriggerChain` variants into their `Effect` equivalents.
fn chain_to_triggered_effect(
    chain: &crate::chips::definition::TriggerChain,
) -> Option<super::definition::Effect> {
    use super::definition::Effect;
    use crate::chips::definition::TriggerChain;

    Some(match chain {
        TriggerChain::Shockwave {
            base_range,
            range_per_level,
            stacks,
            speed,
        } => Effect::Shockwave {
            base_range: *base_range,
            range_per_level: *range_per_level,
            stacks: *stacks,
            speed: *speed,
        },
        TriggerChain::MultiBolt {
            base_count,
            count_per_level,
            stacks,
        } => Effect::MultiBolt {
            base_count: *base_count,
            count_per_level: *count_per_level,
            stacks: *stacks,
        },
        TriggerChain::Shield {
            base_duration,
            duration_per_level,
            stacks,
        } => Effect::Shield {
            base_duration: *base_duration,
            duration_per_level: *duration_per_level,
            stacks: *stacks,
        },
        TriggerChain::LoseLife => Effect::LoseLife,
        TriggerChain::TimePenalty { seconds } => Effect::TimePenalty { seconds: *seconds },
        TriggerChain::SpawnBolt => Effect::SpawnBolt,
        TriggerChain::ChainBolt { tether_distance } => Effect::ChainBolt {
            tether_distance: *tether_distance,
        },
        TriggerChain::ChainLightning {
            arcs,
            range,
            damage_mult,
        } => Effect::ChainLightning {
            arcs: *arcs,
            range: *range,
            damage_mult: *damage_mult,
        },
        TriggerChain::SpawnPhantom {
            duration,
            max_active,
        } => Effect::SpawnPhantom {
            duration: *duration,
            max_active: *max_active,
        },
        TriggerChain::PiercingBeam { damage_mult, width } => Effect::PiercingBeam {
            damage_mult: *damage_mult,
            width: *width,
        },
        TriggerChain::GravityWell {
            strength,
            duration,
            radius,
            max,
        } => Effect::GravityWell {
            strength: *strength,
            duration: *duration,
            radius: *radius,
            max: *max,
        },
        TriggerChain::SecondWind { invuln_secs } => Effect::SecondWind {
            invuln_secs: *invuln_secs,
        },
        TriggerChain::TimedSpeedBurst {
            speed_mult,
            duration_secs,
        } => Effect::TimedSpeedBurst {
            speed_mult: *speed_mult,
            duration_secs: *duration_secs,
        },
        TriggerChain::RandomEffect(pool) => Effect::RandomEffect(pool.clone()),
        TriggerChain::EntropyEngine(threshold, pool) => Effect::EntropyEngine {
            threshold: *threshold,
            pool: pool.clone(),
        },
        _ => return None,
    })
}

// ===========================================================================
// Bridge dispatch — converts Effect → typed event
// ===========================================================================

/// Converts a resolved `Effect` leaf into the appropriate typed event trigger.
///
/// Called by bridge systems after `evaluate()` returns `Fire(effect)`.
/// Replaces all `commands.trigger(EffectFired { ... })` calls.
pub(crate) fn fire_typed_event(
    effect: super::definition::Effect,
    bolt: Option<Entity>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use super::definition::Effect;

    match effect {
        Effect::Shockwave { .. }
        | Effect::LoseLife
        | Effect::TimePenalty { .. }
        | Effect::SpawnBolt
        | Effect::SpeedBoost { .. }
        | Effect::ChainBolt { .. }
        | Effect::MultiBolt { .. }
        | Effect::Shield { .. }
        | Effect::ChainLightning { .. }
        | Effect::SpawnPhantom { .. }
        | Effect::PiercingBeam { .. }
        | Effect::GravityWell { .. }
        | Effect::SecondWind { .. }
        | Effect::TimedSpeedBurst { .. }
        | Effect::RandomEffect(_)
        | Effect::EntropyEngine { .. } => {
            fire_triggered_effect(effect, bolt, source_chip, commands);
        }
        // Passive-only effects — should not be fired via triggered dispatch.
        Effect::Piercing(_)
        | Effect::DamageBoost(_)
        | Effect::ChainHit(_)
        | Effect::SizeBoost(..)
        | Effect::Attraction(_)
        | Effect::BumpForce(_)
        | Effect::TiltControl(_)
        | Effect::RampingDamage { .. }
        | Effect::TimePressureBoost { .. } => {
            warn!(
                "passive effect dispatched via fire_typed_event — should use fire_passive_event: {effect:?}"
            );
        }
    }
}

/// Dispatches a triggered (non-passive) `Effect` as its corresponding typed event.
///
/// Handles core triggered effects (shockwave through chain bolt).
fn fire_triggered_effect(
    effect: super::definition::Effect,
    bolt: Option<Entity>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use super::definition::Effect;

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
                bolt,
                source_chip,
            });
        }
        Effect::LoseLife => {
            commands.trigger(LoseLifeFired { bolt, source_chip });
        }
        Effect::TimePenalty { seconds } => {
            commands.trigger(TimePenaltyFired {
                seconds,
                bolt,
                source_chip,
            });
        }
        Effect::SpawnBolt => {
            commands.trigger(SpawnBoltFired { bolt, source_chip });
        }
        Effect::SpeedBoost { target, multiplier } => {
            commands.trigger(SpeedBoostFired {
                target,
                multiplier,
                bolt,
                source_chip,
            });
        }
        Effect::ChainBolt { tether_distance } => {
            commands.trigger(ChainBoltFired {
                tether_distance,
                bolt,
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
                bolt,
                source_chip,
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
                bolt,
                source_chip,
            });
        }
        _ => fire_exotic_triggered_effect(effect, bolt, source_chip, commands),
    }
}

/// Dispatches exotic triggered effects (chain lightning through second wind).
fn fire_exotic_triggered_effect(
    effect: super::definition::Effect,
    bolt: Option<Entity>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use super::definition::Effect;

    match effect {
        Effect::ChainLightning {
            arcs,
            range,
            damage_mult,
        } => {
            commands.trigger(ChainLightningFired {
                arcs,
                range,
                damage_mult,
                bolt,
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
                bolt,
                source_chip,
            });
        }
        Effect::PiercingBeam { damage_mult, width } => {
            commands.trigger(PiercingBeamFired {
                damage_mult,
                width,
                bolt,
                source_chip,
            });
        }
        Effect::GravityWell {
            strength,
            duration,
            radius,
            max,
        } => {
            commands.trigger(GravityWellFired {
                strength,
                duration,
                radius,
                max,
                bolt,
                source_chip,
            });
        }
        Effect::SecondWind { invuln_secs } => {
            commands.trigger(SecondWindFired {
                invuln_secs,
                bolt,
                source_chip,
            });
        }
        Effect::TimedSpeedBurst {
            speed_mult,
            duration_secs,
        } => {
            commands.trigger(TimedSpeedBurstFired {
                speed_mult,
                duration_secs,
                bolt,
                source_chip,
            });
        }
        Effect::RandomEffect(pool) => {
            commands.trigger(RandomEffectFired {
                pool,
                bolt,
                source_chip,
            });
        }
        Effect::EntropyEngine { threshold, pool } => {
            commands.trigger(EntropyEngineFired {
                threshold,
                pool,
                bolt,
                source_chip,
            });
        }
        _ => unreachable!("fire_exotic_triggered_effect called with non-exotic effect: {effect:?}"),
    }
}

/// Converts a resolved passive `Effect` leaf into the appropriate typed passive event trigger.
///
/// Called by `dispatch_chip_effects` after extracting leaf effects from `OnSelected` nodes.
pub(crate) fn fire_passive_event(
    effect: super::definition::Effect,
    max_stacks: u32,
    chip_name: String,
    commands: &mut Commands,
) {
    use super::definition::Effect;

    match effect {
        Effect::Piercing(per_stack) => {
            commands.trigger(PiercingApplied {
                per_stack,
                max_stacks,
                chip_name,
            });
        }
        Effect::DamageBoost(per_stack) => {
            commands.trigger(DamageBoostApplied {
                per_stack,
                max_stacks,
                chip_name,
            });
        }
        Effect::SpeedBoost { target, multiplier } => {
            commands.trigger(SpeedBoostApplied {
                target,
                multiplier,
                max_stacks,
                chip_name,
            });
        }
        Effect::ChainHit(per_stack) => {
            commands.trigger(ChainHitApplied {
                per_stack,
                max_stacks,
                chip_name,
            });
        }
        Effect::SizeBoost(target, per_stack) => {
            commands.trigger(SizeBoostApplied {
                target,
                per_stack,
                max_stacks,
                chip_name,
            });
        }
        Effect::Attraction(per_stack) => {
            commands.trigger(AttractionApplied {
                per_stack,
                max_stacks,
                chip_name,
            });
        }
        Effect::BumpForce(per_stack) => {
            commands.trigger(BumpForceApplied {
                per_stack,
                max_stacks,
                chip_name,
            });
        }
        Effect::TiltControl(per_stack) => {
            commands.trigger(TiltControlApplied {
                per_stack,
                max_stacks,
                chip_name,
            });
        }
        Effect::RampingDamage {
            bonus_per_hit,
            max_bonus,
        } => {
            commands.trigger(RampingDamageApplied {
                bonus_per_hit,
                max_bonus,
                max_stacks,
                chip_name,
            });
        }
        Effect::TimePressureBoost {
            speed_mult,
            threshold_pct,
        } => {
            commands.trigger(TimePressureBoostApplied {
                speed_mult,
                threshold_pct,
                max_stacks,
                chip_name,
            });
        }
        // Triggered-only effects should not arrive via passive dispatch.
        other => {
            warn!("unexpected triggered effect in passive dispatch: {other:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Test helper resources and observer functions (moved to module scope to
    // satisfy `items_after_statements` lint).
    // =========================================================================

    #[derive(Resource, Default)]
    struct CapturedShockwave(Vec<ShockwaveFired>);

    fn capture_shockwave(trigger: On<ShockwaveFired>, mut captured: ResMut<CapturedShockwave>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedLoseLife(Vec<LoseLifeFired>);

    fn capture_lose_life(trigger: On<LoseLifeFired>, mut captured: ResMut<CapturedLoseLife>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSpeedBoost(Vec<SpeedBoostFired>);

    fn capture_speed_boost(trigger: On<SpeedBoostFired>, mut captured: ResMut<CapturedSpeedBoost>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedPiercing(Vec<PiercingApplied>);

    fn capture_piercing(trigger: On<PiercingApplied>, mut captured: ResMut<CapturedPiercing>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSizeBoost(Vec<SizeBoostApplied>);

    fn capture_size_boost(trigger: On<SizeBoostApplied>, mut captured: ResMut<CapturedSizeBoost>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedPassiveSpeed(Vec<SpeedBoostApplied>);

    fn capture_passive_speed(
        trigger: On<SpeedBoostApplied>,
        mut captured: ResMut<CapturedPassiveSpeed>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    // =========================================================================
    // B12c Behaviors 1-9: Triggered typed event construction
    // =========================================================================

    #[test]
    fn shockwave_fired_carries_all_parameters() {
        let event = ShockwaveFired {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert!((event.base_range - 64.0).abs() < f32::EPSILON);
        assert!((event.speed - 400.0).abs() < f32::EPSILON);
        assert_eq!(event.bolt, Some(Entity::PLACEHOLDER));
        assert!(event.source_chip.is_none());
    }

    #[test]
    fn shockwave_fired_with_source_chip() {
        let event = ShockwaveFired {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: Some("Surge".to_owned()),
        };
        assert_eq!(event.source_chip, Some("Surge".to_owned()));
    }

    #[test]
    fn lose_life_fired_with_none_bolt() {
        let event = LoseLifeFired {
            bolt: None,
            source_chip: None,
        };
        assert_eq!(event.bolt, None);
        assert!(event.source_chip.is_none());
    }

    #[test]
    fn time_penalty_fired_carries_seconds() {
        let event = TimePenaltyFired {
            seconds: 3.0,
            bolt: None,
            source_chip: None,
        };
        assert!((event.seconds - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn spawn_bolt_fired_carries_source_chip() {
        let event = SpawnBoltFired {
            bolt: None,
            source_chip: Some("Reflex".to_owned()),
        };
        assert_eq!(event.source_chip, Some("Reflex".to_owned()));
    }

    #[test]
    fn speed_boost_fired_carries_target_and_multiplier() {
        let event = SpeedBoostFired {
            target: Target::Bolt,
            multiplier: 1.3,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert_eq!(event.target, Target::Bolt);
        assert!((event.multiplier - 1.3).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_boost_fired_all_bolts_target() {
        let event = SpeedBoostFired {
            target: Target::AllBolts,
            multiplier: 1.3,
            bolt: None,
            source_chip: None,
        };
        assert_eq!(event.target, Target::AllBolts);
    }

    #[test]
    fn chain_bolt_fired_carries_tether_distance() {
        let event = ChainBoltFired {
            tether_distance: 150.0,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert!((event.tether_distance - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multi_bolt_fired_carries_count_parameters() {
        let event = MultiBoltFired {
            base_count: 2,
            count_per_level: 1,
            stacks: 1,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert_eq!(event.base_count, 2);
    }

    #[test]
    fn shield_fired_carries_duration_and_stacks() {
        let event = ShieldFired {
            base_duration: 3.0,
            duration_per_level: 0.5,
            stacks: 2,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert!((event.base_duration - 3.0).abs() < f32::EPSILON);
        assert_eq!(event.stacks, 2);
    }

    #[test]
    fn stub_event_chain_lightning_fired_accessible() {
        let event = ChainLightningFired {
            arcs: 3,
            range: 64.0,
            damage_mult: 0.5,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert_eq!(event.arcs, 3);
    }

    #[test]
    fn stub_event_spawn_phantom_fired_accessible() {
        let event = SpawnPhantomFired {
            duration: 5.0,
            max_active: 2,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert!((event.duration - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stub_event_piercing_beam_fired_accessible() {
        let event = PiercingBeamFired {
            damage_mult: 1.5,
            width: 20.0,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert!((event.damage_mult - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn stub_event_gravity_well_fired_accessible() {
        let event = GravityWellFired {
            strength: 1.0,
            duration: 5.0,
            radius: 100.0,
            max: 2,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert!((event.strength - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stub_event_second_wind_fired_accessible() {
        let event = SecondWindFired {
            invuln_secs: 3.0,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: None,
        };
        assert!((event.invuln_secs - 3.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // B12c Behaviors 10-14: Passive typed event construction
    // =========================================================================

    #[test]
    fn piercing_applied_carries_per_stack_and_max() {
        let event = PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: "Piercing Shot".to_owned(),
        };
        assert_eq!(event.per_stack, 1);
        assert_eq!(event.max_stacks, 3);
    }

    #[test]
    fn damage_boost_applied_carries_per_stack() {
        let event = DamageBoostApplied {
            per_stack: 0.5,
            max_stacks: 3,
            chip_name: "Damage Up".to_owned(),
        };
        assert!((event.per_stack - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn speed_boost_applied_carries_target() {
        let event = SpeedBoostApplied {
            target: Target::Bolt,
            multiplier: 0.1,
            max_stacks: 3,
            chip_name: "Quick Shot".to_owned(),
        };
        assert_eq!(event.target, Target::Bolt);
    }

    #[test]
    fn speed_boost_applied_breaker_target() {
        let event = SpeedBoostApplied {
            target: Target::Breaker,
            multiplier: 0.2,
            max_stacks: 3,
            chip_name: "Breaker Speed".to_owned(),
        };
        assert_eq!(event.target, Target::Breaker);
    }

    #[test]
    fn size_boost_applied_bolt_target() {
        let event = SizeBoostApplied {
            target: Target::Bolt,
            per_stack: 5.0,
            max_stacks: 3,
            chip_name: "Big Shot".to_owned(),
        };
        assert_eq!(event.target, Target::Bolt);
    }

    #[test]
    fn size_boost_applied_breaker_target() {
        let event = SizeBoostApplied {
            target: Target::Breaker,
            per_stack: 20.0,
            max_stacks: 3,
            chip_name: "Wide Breaker".to_owned(),
        };
        assert_eq!(event.target, Target::Breaker);
    }

    #[test]
    fn chain_hit_applied_accessible() {
        let event = ChainHitApplied {
            per_stack: 2,
            max_stacks: 3,
            chip_name: "Chain Hit".to_owned(),
        };
        assert_eq!(event.per_stack, 2);
    }

    #[test]
    fn attraction_applied_accessible() {
        let event = AttractionApplied {
            per_stack: 8.0,
            max_stacks: 3,
            chip_name: "Attraction".to_owned(),
        };
        assert!((event.per_stack - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bump_force_applied_accessible() {
        let event = BumpForceApplied {
            per_stack: 0.2,
            max_stacks: 3,
            chip_name: "Bump Force".to_owned(),
        };
        assert!((event.per_stack - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn tilt_control_applied_accessible() {
        let event = TiltControlApplied {
            per_stack: 0.1,
            max_stacks: 3,
            chip_name: "Tilt Control".to_owned(),
        };
        assert!((event.per_stack - 0.1).abs() < f32::EPSILON);
    }

    // =========================================================================
    // B12c Behaviors 15-17: fire_typed_event dispatches correctly
    // =========================================================================

    #[test]
    fn fire_typed_event_dispatches_shockwave() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedShockwave>()
            .add_observer(capture_shockwave);

        let effect = Effect::Shockwave {
            base_range: 32.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, None, None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedShockwave>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_typed_event should dispatch ShockwaveFired for Effect::Shockwave"
        );
        assert!(
            (captured.0[0].base_range - 32.0).abs() < f32::EPSILON,
            "ShockwaveFired.base_range should be 32.0"
        );
    }

    #[test]
    fn fire_typed_event_dispatches_lose_life() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedLoseLife>()
            .add_observer(capture_lose_life);

        let effect = Effect::LoseLife;
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, None, None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedLoseLife>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_typed_event should dispatch LoseLifeFired for Effect::LoseLife"
        );
        assert_eq!(captured.0[0].bolt, None);
    }

    #[test]
    fn fire_typed_event_dispatches_speed_boost() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedSpeedBoost>()
            .add_observer(capture_speed_boost);

        let bolt = Entity::PLACEHOLDER;
        let effect = Effect::SpeedBoost {
            target: Target::Bolt,
            multiplier: 1.3,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, Some(bolt), None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpeedBoost>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_typed_event should dispatch SpeedBoostFired for Effect::SpeedBoost"
        );
        assert_eq!(captured.0[0].target, Target::Bolt);
        assert!((captured.0[0].multiplier - 1.3).abs() < f32::EPSILON);
    }

    // =========================================================================
    // B12c Behaviors 18-20: fire_passive_event dispatches correctly
    // =========================================================================

    #[test]
    fn fire_passive_event_dispatches_piercing() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedPiercing>()
            .add_observer(capture_piercing);

        let effect = Effect::Piercing(1);
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_passive_event(effect, 3, "Piercing Shot".to_owned(), &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedPiercing>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_passive_event should dispatch PiercingApplied for Effect::Piercing"
        );
        assert_eq!(captured.0[0].per_stack, 1);
        assert_eq!(captured.0[0].max_stacks, 3);
        assert_eq!(captured.0[0].chip_name, "Piercing Shot");
    }

    #[test]
    fn fire_passive_event_dispatches_size_boost() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedSizeBoost>()
            .add_observer(capture_size_boost);

        let effect = Effect::SizeBoost(Target::Bolt, 5.0);
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_passive_event(effect, 3, "Big Shot".to_owned(), &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSizeBoost>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_passive_event should dispatch SizeBoostApplied for Effect::SizeBoost"
        );
        assert_eq!(captured.0[0].target, Target::Bolt);
        assert!((captured.0[0].per_stack - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fire_passive_event_dispatches_speed_boost_for_both_targets() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedPassiveSpeed>()
            .add_observer(capture_passive_speed);

        // Fire for Bolt target
        let bolt_effect = Effect::SpeedBoost {
            target: Target::Bolt,
            multiplier: 0.1,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_passive_event(bolt_effect, 3, "Speed Mix".to_owned(), &mut commands);
        });
        app.world_mut().flush();

        // Fire for Breaker target
        let breaker_effect = Effect::SpeedBoost {
            target: Target::Breaker,
            multiplier: 0.2,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_passive_event(breaker_effect, 3, "Speed Mix".to_owned(), &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedPassiveSpeed>();
        assert_eq!(
            captured.0.len(),
            2,
            "fire_passive_event should dispatch SpeedBoostApplied for both Bolt and Breaker targets"
        );
        assert_eq!(captured.0[0].target, Target::Bolt);
        assert_eq!(captured.0[1].target, Target::Breaker);
    }

    // =========================================================================
    // C2-C4: RampingDamage conversion and passive dispatch tests
    // =========================================================================

    #[test]
    fn trigger_chain_to_effect_converts_ramping_damage() {
        use crate::chips::definition::TriggerChain;

        let chain = TriggerChain::RampingDamage {
            bonus_per_hit: 0.04,
            max_bonus: 0.4,
        };
        let effect = trigger_chain_to_effect(&chain);
        assert_eq!(
            effect,
            super::super::definition::Effect::RampingDamage {
                bonus_per_hit: 0.04,
                max_bonus: 0.4,
            }
        );
    }

    #[derive(Resource, Default)]
    struct CapturedRampingDamage(Vec<RampingDamageApplied>);

    fn capture_ramping_damage(
        trigger: On<RampingDamageApplied>,
        mut captured: ResMut<CapturedRampingDamage>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[test]
    fn fire_passive_event_dispatches_ramping_damage() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedRampingDamage>()
            .add_observer(capture_ramping_damage);

        let effect = Effect::RampingDamage {
            bonus_per_hit: 0.02,
            max_bonus: 0.2,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_passive_event(effect, 2, "Basic Amp".to_owned(), &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedRampingDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_passive_event should dispatch RampingDamageApplied for Effect::RampingDamage"
        );
        assert!(
            (captured.0[0].bonus_per_hit - 0.02).abs() < f32::EPSILON,
            "bonus_per_hit should be 0.02"
        );
        assert!(
            (captured.0[0].max_bonus - 0.2).abs() < f32::EPSILON,
            "max_bonus should be 0.2"
        );
        assert_eq!(captured.0[0].max_stacks, 2);
        assert_eq!(captured.0[0].chip_name, "Basic Amp");
    }

    // =========================================================================
    // C2-C4: TimedSpeedBurst conversion and triggered dispatch tests
    // =========================================================================

    #[test]
    fn trigger_chain_to_effect_converts_timed_speed_burst() {
        use crate::chips::definition::TriggerChain;

        let chain = TriggerChain::TimedSpeedBurst {
            speed_mult: 1.5,
            duration_secs: 3.0,
        };
        let effect = trigger_chain_to_effect(&chain);
        assert_eq!(
            effect,
            super::super::definition::Effect::TimedSpeedBurst {
                speed_mult: 1.5,
                duration_secs: 3.0,
            }
        );
    }

    #[derive(Resource, Default)]
    struct CapturedTimedSpeedBurst(Vec<TimedSpeedBurstFired>);

    fn capture_timed_speed_burst(
        trigger: On<TimedSpeedBurstFired>,
        mut captured: ResMut<CapturedTimedSpeedBurst>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[test]
    fn fire_typed_event_dispatches_timed_speed_burst() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedTimedSpeedBurst>()
            .add_observer(capture_timed_speed_burst);

        let bolt = Entity::PLACEHOLDER;
        let effect = Effect::TimedSpeedBurst {
            speed_mult: 1.3,
            duration_secs: 2.0,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(
                effect,
                Some(bolt),
                Some("Basic Overclock".to_owned()),
                &mut commands,
            );
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedTimedSpeedBurst>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_typed_event should dispatch TimedSpeedBurstFired for Effect::TimedSpeedBurst"
        );
        assert!(
            (captured.0[0].speed_mult - 1.3).abs() < f32::EPSILON,
            "speed_mult should be 1.3"
        );
        assert!(
            (captured.0[0].duration_secs - 2.0).abs() < f32::EPSILON,
            "duration_secs should be 2.0"
        );
        assert_eq!(captured.0[0].bolt, Some(bolt));
        assert_eq!(
            captured.0[0].source_chip,
            Some("Basic Overclock".to_owned())
        );
    }

    // =========================================================================
    // C2-C4: Event struct construction tests
    // =========================================================================

    #[test]
    fn ramping_damage_applied_carries_all_fields() {
        let event = RampingDamageApplied {
            bonus_per_hit: 0.04,
            max_bonus: 0.4,
            max_stacks: 2,
            chip_name: "Potent Amp".to_owned(),
        };
        assert!((event.bonus_per_hit - 0.04).abs() < f32::EPSILON);
        assert!((event.max_bonus - 0.4).abs() < f32::EPSILON);
        assert_eq!(event.max_stacks, 2);
        assert_eq!(event.chip_name, "Potent Amp");
    }

    #[test]
    fn timed_speed_burst_fired_carries_all_fields() {
        let event = TimedSpeedBurstFired {
            speed_mult: 1.5,
            duration_secs: 3.0,
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: Some("Charged Overclock".to_owned()),
        };
        assert!((event.speed_mult - 1.5).abs() < f32::EPSILON);
        assert!((event.duration_secs - 3.0).abs() < f32::EPSILON);
        assert_eq!(event.bolt, Some(Entity::PLACEHOLDER));
        assert_eq!(event.source_chip, Some("Charged Overclock".to_owned()));
    }

    #[test]
    fn timed_speed_burst_fired_none_bolt() {
        let event = TimedSpeedBurstFired {
            speed_mult: 1.3,
            duration_secs: 2.0,
            bolt: None,
            source_chip: None,
        };
        assert_eq!(event.bolt, None);
        assert!(event.source_chip.is_none());
    }

    // =========================================================================
    // C5-C6: TimePressureBoost conversion and passive dispatch tests
    // =========================================================================

    #[test]
    fn trigger_chain_to_effect_converts_time_pressure_boost() {
        use crate::chips::definition::TriggerChain;

        let chain = TriggerChain::TimePressureBoost {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        };
        let effect = trigger_chain_to_effect(&chain);
        assert_eq!(
            effect,
            super::super::definition::Effect::TimePressureBoost {
                speed_mult: 2.0,
                threshold_pct: 0.25,
            }
        );
    }

    #[derive(Resource, Default)]
    struct CapturedTimePressureBoost(Vec<TimePressureBoostApplied>);

    fn capture_time_pressure_boost(
        trigger: On<TimePressureBoostApplied>,
        mut captured: ResMut<CapturedTimePressureBoost>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[test]
    fn fire_passive_event_dispatches_time_pressure_boost() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedTimePressureBoost>()
            .add_observer(capture_time_pressure_boost);

        let effect = Effect::TimePressureBoost {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_passive_event(effect, 1, "Deadline".to_owned(), &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedTimePressureBoost>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_passive_event should dispatch TimePressureBoostApplied for Effect::TimePressureBoost"
        );
        assert!(
            (captured.0[0].speed_mult - 2.0).abs() < f32::EPSILON,
            "speed_mult should be 2.0"
        );
        assert!(
            (captured.0[0].threshold_pct - 0.25).abs() < f32::EPSILON,
            "threshold_pct should be 0.25"
        );
        assert_eq!(captured.0[0].max_stacks, 1);
        assert_eq!(captured.0[0].chip_name, "Deadline");
    }

    // =========================================================================
    // C5-C6: RandomEffect conversion and triggered dispatch tests
    // =========================================================================

    #[test]
    fn trigger_chain_to_effect_converts_random_effect() {
        use crate::chips::definition::TriggerChain;

        let chain = TriggerChain::RandomEffect(vec![
            (0.5, TriggerChain::SpawnBolt),
            (0.5, TriggerChain::test_speed_boost(1.1)),
        ]);
        let effect = trigger_chain_to_effect(&chain);
        match effect {
            super::super::definition::Effect::RandomEffect(pool) => {
                assert_eq!(pool.len(), 2, "pool should contain 2 entries");
                assert!((pool[0].0 - 0.5).abs() < f32::EPSILON);
                assert_eq!(pool[0].1, TriggerChain::SpawnBolt);
                assert!((pool[1].0 - 0.5).abs() < f32::EPSILON);
            }
            other => panic!("expected RandomEffect, got {other:?}"),
        }
    }

    #[derive(Resource, Default)]
    struct CapturedRandomEffect(Vec<RandomEffectFired>);

    fn capture_random_effect(
        trigger: On<RandomEffectFired>,
        mut captured: ResMut<CapturedRandomEffect>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[test]
    fn fire_typed_event_dispatches_random_effect() {
        use super::super::definition::Effect;
        use crate::chips::definition::TriggerChain;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedRandomEffect>()
            .add_observer(capture_random_effect);

        let pool = vec![
            (0.5, TriggerChain::SpawnBolt),
            (0.5, TriggerChain::test_speed_boost(1.1)),
        ];
        let effect = Effect::RandomEffect(pool);
        let bolt = Entity::PLACEHOLDER;
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(
                effect,
                Some(bolt),
                Some("Volatile Flux".to_owned()),
                &mut commands,
            );
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedRandomEffect>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_typed_event should dispatch RandomEffectFired for Effect::RandomEffect"
        );
        assert_eq!(captured.0[0].pool.len(), 2);
        assert_eq!(captured.0[0].bolt, Some(bolt));
        assert_eq!(captured.0[0].source_chip, Some("Volatile Flux".to_owned()));
    }

    // =========================================================================
    // C5-C6: EntropyEngine conversion and triggered dispatch tests
    // =========================================================================

    #[test]
    fn trigger_chain_to_effect_converts_entropy_engine() {
        use crate::chips::definition::TriggerChain;

        let chain = TriggerChain::EntropyEngine(5, vec![(1.0, TriggerChain::SpawnBolt)]);
        let effect = trigger_chain_to_effect(&chain);
        match effect {
            super::super::definition::Effect::EntropyEngine { threshold, pool } => {
                assert_eq!(threshold, 5);
                assert_eq!(pool.len(), 1);
                assert!((pool[0].0 - 1.0).abs() < f32::EPSILON);
                assert_eq!(pool[0].1, TriggerChain::SpawnBolt);
            }
            other => panic!("expected EntropyEngine, got {other:?}"),
        }
    }

    #[derive(Resource, Default)]
    struct CapturedEntropyEngine(Vec<EntropyEngineFired>);

    fn capture_entropy_engine(
        trigger: On<EntropyEngineFired>,
        mut captured: ResMut<CapturedEntropyEngine>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[test]
    fn fire_typed_event_dispatches_entropy_engine() {
        use super::super::definition::Effect;
        use crate::chips::definition::TriggerChain;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedEntropyEngine>()
            .add_observer(capture_entropy_engine);

        let pool = vec![(1.0, TriggerChain::SpawnBolt)];
        let effect = Effect::EntropyEngine { threshold: 5, pool };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(
                effect,
                None,
                Some("Entropy Engine".to_owned()),
                &mut commands,
            );
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedEntropyEngine>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_typed_event should dispatch EntropyEngineFired for Effect::EntropyEngine"
        );
        assert_eq!(captured.0[0].threshold, 5);
        assert_eq!(captured.0[0].pool.len(), 1);
        assert_eq!(captured.0[0].bolt, None);
        assert_eq!(captured.0[0].source_chip, Some("Entropy Engine".to_owned()));
    }

    // =========================================================================
    // C5-C6: Event struct construction tests
    // =========================================================================

    #[test]
    fn time_pressure_boost_applied_carries_all_fields() {
        let event = TimePressureBoostApplied {
            speed_mult: 2.0,
            threshold_pct: 0.25,
            max_stacks: 1,
            chip_name: "Deadline".to_owned(),
        };
        assert!((event.speed_mult - 2.0).abs() < f32::EPSILON);
        assert!((event.threshold_pct - 0.25).abs() < f32::EPSILON);
        assert_eq!(event.max_stacks, 1);
        assert_eq!(event.chip_name, "Deadline");
    }

    #[test]
    fn random_effect_fired_carries_all_fields() {
        use crate::chips::definition::TriggerChain;

        let event = RandomEffectFired {
            pool: vec![(1.0, TriggerChain::SpawnBolt)],
            bolt: Some(Entity::PLACEHOLDER),
            source_chip: Some("Volatile Flux".to_owned()),
        };
        assert_eq!(event.pool.len(), 1);
        assert_eq!(event.bolt, Some(Entity::PLACEHOLDER));
        assert_eq!(event.source_chip, Some("Volatile Flux".to_owned()));
    }

    #[test]
    fn entropy_engine_fired_carries_all_fields() {
        use crate::chips::definition::TriggerChain;

        let event = EntropyEngineFired {
            threshold: 5,
            pool: vec![(0.5, TriggerChain::SpawnBolt)],
            bolt: None,
            source_chip: Some("Entropy Engine".to_owned()),
        };
        assert_eq!(event.threshold, 5);
        assert_eq!(event.pool.len(), 1);
        assert_eq!(event.bolt, None);
        assert_eq!(event.source_chip, Some("Entropy Engine".to_owned()));
    }
}
