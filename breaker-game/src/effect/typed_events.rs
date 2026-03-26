//! Typed per-effect observer events and dispatch helpers.
//!
//! Each event replaces the catchall `EffectFired` / `ChipEffectApplied` with a
//! concrete struct per `Effect` variant. Dispatch helpers convert `Effect`
//! values and fire the corresponding typed events.

use bevy::prelude::*;
use tracing::warn;

use super::definition::{AttractionType, EffectTarget, Target};

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
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The chip name that originated this chain, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a lose-life effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct LoseLifeFired {
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a time penalty effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct TimePenaltyFired {
    /// Duration of the penalty in seconds.
    pub seconds: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a spawn-bolts effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpawnBoltsFired {
    /// Number of bolts to spawn.
    pub count: u32,
    /// Optional lifespan in seconds (temporary bolts).
    pub lifespan: Option<f32>,
    /// Whether spawned bolts inherit the parent bolt's velocity.
    pub inherit: bool,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Backward-compatible alias — production code still references this name.
///
/// Will be removed when all handler files are updated.
pub(crate) type SpawnBoltFired = SpawnBoltsFired;

/// Fired when a speed boost effect resolves via a triggered chain.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpeedBoostFired {
    /// Which entity to apply the speed change to.
    pub target: Target,
    /// Multiplier applied to the current velocity magnitude.
    pub multiplier: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a chain bolt effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChainBoltFired {
    /// Maximum distance the chain bolt can travel from its anchor.
    pub tether_distance: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
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
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
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
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
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
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
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
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
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
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
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
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a second wind effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SecondWindFired {
    /// Duration of invulnerability in seconds.
    pub invuln_secs: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when a random effect pool needs to be resolved.
#[derive(Event, Clone, Debug)]
pub(crate) struct RandomEffectFired {
    /// Weighted pool of `EffectNode` entries to select from.
    pub pool: Vec<(f32, super::definition::EffectNode)>,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Fired when an entropy engine effect needs counting and potential resolution.
#[derive(Event, Clone, Debug)]
pub(crate) struct EntropyEngineFired {
    /// Number of cell destructions needed before firing.
    pub threshold: u32,
    /// Weighted pool of `EffectNode` entries to select from on trigger.
    pub pool: Vec<(f32, super::definition::EffectNode)>,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
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
    /// The type of entity attraction targets.
    pub attraction_type: AttractionType,
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
// Bridge dispatch — converts Effect -> typed event
// ===========================================================================

/// Converts a resolved `Effect` leaf into the appropriate typed event trigger.
///
/// Called by bridge systems after `evaluate_node()` returns `Fire(effect)`.
pub(crate) fn fire_typed_event(
    effect: super::definition::Effect,
    targets: Vec<EffectTarget>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use super::definition::Effect;

    match effect {
        // -- Bolt combat effects --
        effect @ (Effect::Shockwave { .. }
        | Effect::SpeedBoost { .. }
        | Effect::ChainBolt { .. }
        | Effect::MultiBolt { .. }
        | Effect::ChainLightning { .. }
        | Effect::PiercingBeam { .. }) => {
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
        effect @ (Effect::Piercing(_)
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
                    "fire_typed_event called with passive-only effect {effect:?} — should use fire_passive_event"
                );
            }
        }
    }
}

/// Dispatches bolt combat effects (shockwave, speed boost, chain bolt, etc.).
fn fire_bolt_effect(
    effect: super::definition::Effect,
    targets: Vec<EffectTarget>,
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
                targets,
                source_chip,
            });
        }
        Effect::SpeedBoost { target, multiplier } => {
            commands.trigger(SpeedBoostFired {
                target,
                multiplier,
                targets,
                source_chip,
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
                targets,
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
        _ => {}
    }
}

/// Dispatches life, penalty, spawn, and defensive effects.
fn fire_global_effect(
    effect: super::definition::Effect,
    targets: Vec<EffectTarget>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use super::definition::Effect;

    match effect {
        Effect::LoseLife => {
            commands.trigger(LoseLifeFired {
                targets,
                source_chip,
            });
        }
        Effect::TimePenalty { seconds } => {
            commands.trigger(TimePenaltyFired {
                seconds,
                targets,
                source_chip,
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
                targets,
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
                targets,
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
                targets,
                source_chip,
            });
        }
        Effect::SecondWind { invuln_secs } => {
            commands.trigger(SecondWindFired {
                invuln_secs,
                targets,
                source_chip,
            });
        }
        _ => {}
    }
}

/// Dispatches pool / random effects (`RandomEffect`, `EntropyEngine`).
fn fire_pool_effect(
    effect: super::definition::Effect,
    targets: Vec<EffectTarget>,
    source_chip: Option<String>,
    commands: &mut Commands,
) {
    use super::definition::Effect;

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
        Effect::Attraction(attraction_type, per_stack) => {
            commands.trigger(AttractionApplied {
                attraction_type,
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

#[cfg(test)]
mod tests {
    use super::{
        super::definition::{AttractionType, EffectNode, EffectTarget},
        *,
    };

    // =========================================================================
    // C7 Wave 1 Part E: Typed events with targets: Vec<EffectTarget> (behaviors 29-30)
    // =========================================================================

    #[test]
    fn shockwave_fired_with_targets_vec() {
        let entity = Entity::PLACEHOLDER;
        let event = ShockwaveFired {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
            targets: vec![EffectTarget::Entity(entity)],
            source_chip: None,
        };
        assert_eq!(event.targets.len(), 1);
        assert_eq!(event.targets[0], EffectTarget::Entity(entity));
    }

    #[test]
    fn lose_life_fired_empty_targets_equivalent_to_old_none_bolt() {
        let event = LoseLifeFired {
            targets: vec![],
            source_chip: None,
        };
        assert!(
            event.targets.is_empty(),
            "empty targets semantically equivalent to old bolt: None"
        );
    }

    #[test]
    fn lose_life_fired_multiple_targets() {
        let event = LoseLifeFired {
            targets: vec![
                EffectTarget::Entity(Entity::PLACEHOLDER),
                EffectTarget::Entity(Entity::PLACEHOLDER),
            ],
            source_chip: None,
        };
        assert_eq!(event.targets.len(), 2);
    }

    #[test]
    fn spawn_bolts_fired_carries_new_fields() {
        let event = SpawnBoltsFired {
            count: 2,
            lifespan: Some(5.0),
            inherit: true,
            targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
            source_chip: Some("Reflex".to_owned()),
        };
        assert_eq!(event.count, 2);
        assert_eq!(event.lifespan, Some(5.0));
        assert!(event.inherit);
        assert_eq!(event.targets.len(), 1);
    }

    #[test]
    fn speed_boost_fired_with_targets() {
        let event = SpeedBoostFired {
            target: Target::Bolt,
            multiplier: 1.3,
            targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
            source_chip: None,
        };
        assert_eq!(event.target, Target::Bolt);
        assert!((event.multiplier - 1.3).abs() < f32::EPSILON);
        assert_eq!(event.targets.len(), 1);
    }

    #[test]
    fn random_effect_fired_pool_uses_effect_node() {
        let event = RandomEffectFired {
            pool: vec![(
                1.0,
                EffectNode::Do(super::super::definition::Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
            )],
            targets: vec![],
            source_chip: None,
        };
        assert_eq!(event.pool.len(), 1);
    }

    #[test]
    fn entropy_engine_fired_pool_uses_effect_node() {
        let event = EntropyEngineFired {
            threshold: 5,
            pool: vec![(
                1.0,
                EffectNode::Do(super::super::definition::Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
            )],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        };
        assert_eq!(event.threshold, 5);
        assert_eq!(event.pool.len(), 1);
    }

    // =========================================================================
    // C7 Wave 1 Part G: AttractionApplied with AttractionType (behavior 39)
    // =========================================================================

    #[test]
    fn attraction_applied_carries_attraction_type_cell() {
        let event = AttractionApplied {
            attraction_type: AttractionType::Cell,
            per_stack: 1.0,
            max_stacks: 3,
            chip_name: "Magnet".to_owned(),
        };
        assert_eq!(event.attraction_type, AttractionType::Cell);
        assert!((event.per_stack - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn attraction_applied_carries_attraction_type_wall() {
        let event = AttractionApplied {
            attraction_type: AttractionType::Wall,
            per_stack: 0.5,
            max_stacks: 3,
            chip_name: "Wall Magnet".to_owned(),
        };
        assert_eq!(event.attraction_type, AttractionType::Wall);
    }

    // =========================================================================
    // fire_passive_event dispatch (behavior 39)
    // =========================================================================

    #[derive(Resource, Default)]
    struct CapturedAttraction(Vec<AttractionApplied>);

    fn capture_attraction(
        trigger: On<AttractionApplied>,
        mut captured: ResMut<CapturedAttraction>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[test]
    fn fire_passive_event_dispatches_attraction_with_type() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedAttraction>()
            .add_observer(capture_attraction);

        let effect = Effect::Attraction(AttractionType::Cell, 1.0);
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_passive_event(effect, 3, "Magnet".to_owned(), &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedAttraction>();
        assert_eq!(
            captured.0.len(),
            1,
            "fire_passive_event should dispatch AttractionApplied for Effect::Attraction"
        );
        assert_eq!(captured.0[0].attraction_type, AttractionType::Cell);
        assert!((captured.0[0].per_stack - 1.0).abs() < f32::EPSILON);
        assert_eq!(captured.0[0].max_stacks, 3);
        assert_eq!(captured.0[0].chip_name, "Magnet");
    }

    // =========================================================================
    // Preserved: passive event construction tests
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
    // Preserved: triggered event construction tests (updated for targets)
    // =========================================================================

    #[test]
    fn time_penalty_fired_carries_seconds() {
        let event = TimePenaltyFired {
            seconds: 3.0,
            targets: vec![],
            source_chip: None,
        };
        assert!((event.seconds - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn chain_bolt_fired_carries_tether_distance() {
        let event = ChainBoltFired {
            tether_distance: 150.0,
            targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
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
            targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
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
            targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
            source_chip: None,
        };
        assert!((event.base_duration - 3.0).abs() < f32::EPSILON);
        assert_eq!(event.stacks, 2);
    }
}
