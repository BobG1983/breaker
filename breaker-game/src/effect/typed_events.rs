//! Typed per-effect observer events and dispatch helpers.
//!
//! Each event struct now lives in its corresponding effect handler file.
//! This module re-exports them for backward compatibility and contains
//! the dispatch helpers that convert `Effect` values into typed events.

use bevy::prelude::*;
use tracing::warn;

use super::definition::EffectTarget;
// ===========================================================================
// Re-exports — passive effect events (canonical location: effects/<name>.rs)
// ===========================================================================
pub(crate) use super::effects::attraction::AttractionApplied;
// ===========================================================================
// Re-exports — triggered effect events (canonical location: effects/<name>.rs)
// ===========================================================================
pub(crate) use super::effects::chain_bolt::ChainBoltFired;
pub(crate) use super::effects::{
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
    spawn_bolt::{SpawnBoltFired, SpawnBoltsFired},
    spawn_phantom::SpawnPhantomFired,
    speed_boost::SpeedBoostFired,
    tilt_control_boost::TiltControlApplied,
    time_penalty::TimePenaltyFired,
};

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
        Effect::SpeedBoost { multiplier } => {
            commands.trigger(SpeedBoostFired {
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
        Effect::SpeedBoost { multiplier } => {
            commands.trigger(SpeedBoostApplied {
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
        Effect::SizeBoost(per_stack) => {
            commands.trigger(SizeBoostApplied {
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
        Effect::RampingDamage { bonus_per_hit } => {
            commands.trigger(RampingDamageApplied {
                bonus_per_hit,
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
            multiplier: 1.3,
            targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
            source_chip: None,
        };
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
    // M10: fire_typed_event dispatches all Effect variants via observers
    // =========================================================================

    #[derive(Resource, Default)]
    struct CapturedChainBolt(Vec<ChainBoltFired>);

    fn capture_chain_bolt(
        trigger: On<ChainBoltFired>,
        mut captured: ResMut<CapturedChainBolt>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedMultiBolt(Vec<MultiBoltFired>);

    fn capture_multi_bolt(
        trigger: On<MultiBoltFired>,
        mut captured: ResMut<CapturedMultiBolt>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedShield(Vec<ShieldFired>);

    fn capture_shield(trigger: On<ShieldFired>, mut captured: ResMut<CapturedShield>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedLoseLife(Vec<LoseLifeFired>);

    fn capture_lose_life(trigger: On<LoseLifeFired>, mut captured: ResMut<CapturedLoseLife>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedTimePenalty(Vec<TimePenaltyFired>);

    fn capture_time_penalty(
        trigger: On<TimePenaltyFired>,
        mut captured: ResMut<CapturedTimePenalty>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSpawnBolts(Vec<SpawnBoltsFired>);

    fn capture_spawn_bolts(
        trigger: On<SpawnBoltsFired>,
        mut captured: ResMut<CapturedSpawnBolts>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    /// M10: fire_typed_event dispatches ChainBolt into ChainBoltFired observer.
    #[test]
    fn fire_typed_event_dispatches_chain_bolt() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedChainBolt>()
            .add_observer(capture_chain_bolt);

        let effect = Effect::ChainBolt {
            tether_distance: 100.0,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, vec![], None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedChainBolt>();
        assert_eq!(captured.0.len(), 1, "ChainBolt should dispatch ChainBoltFired");
        assert!(
            (captured.0[0].tether_distance - 100.0).abs() < f32::EPSILON,
            "tether_distance should be 100.0"
        );
    }

    /// M10: fire_typed_event dispatches MultiBolt into MultiBoltFired observer.
    #[test]
    fn fire_typed_event_dispatches_multi_bolt() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedMultiBolt>()
            .add_observer(capture_multi_bolt);

        let effect = Effect::MultiBolt {
            base_count: 2,
            count_per_level: 0,
            stacks: 1,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, vec![], None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedMultiBolt>();
        assert_eq!(captured.0.len(), 1, "MultiBolt should dispatch MultiBoltFired");
        assert_eq!(captured.0[0].base_count, 2);
        assert_eq!(captured.0[0].count_per_level, 0);
        assert_eq!(captured.0[0].stacks, 1);
    }

    /// M10: fire_typed_event dispatches Shield into ShieldFired observer.
    #[test]
    fn fire_typed_event_dispatches_shield() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedShield>()
            .add_observer(capture_shield);

        let effect = Effect::Shield {
            base_duration: 3.0,
            duration_per_level: 0.0,
            stacks: 1,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, vec![], None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedShield>();
        assert_eq!(captured.0.len(), 1, "Shield should dispatch ShieldFired");
        assert!((captured.0[0].base_duration - 3.0).abs() < f32::EPSILON);
        assert_eq!(captured.0[0].stacks, 1);
    }

    /// M10: fire_typed_event dispatches LoseLife into LoseLifeFired observer.
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
            fire_typed_event(effect, vec![], None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedLoseLife>();
        assert_eq!(captured.0.len(), 1, "LoseLife should dispatch LoseLifeFired");
    }

    /// M10: fire_typed_event dispatches TimePenalty into TimePenaltyFired observer.
    #[test]
    fn fire_typed_event_dispatches_time_penalty() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedTimePenalty>()
            .add_observer(capture_time_penalty);

        let effect = Effect::TimePenalty { seconds: 5.0 };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, vec![], None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedTimePenalty>();
        assert_eq!(
            captured.0.len(),
            1,
            "TimePenalty should dispatch TimePenaltyFired"
        );
        assert!((captured.0[0].seconds - 5.0).abs() < f32::EPSILON);
    }

    /// M10: fire_typed_event dispatches SpawnBolts into SpawnBoltsFired observer.
    #[test]
    fn fire_typed_event_dispatches_spawn_bolts() {
        use super::super::definition::Effect;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CapturedSpawnBolts>()
            .add_observer(capture_spawn_bolts);

        let effect = Effect::SpawnBolts {
            count: 1,
            lifespan: None,
            inherit: false,
        };
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            fire_typed_event(effect, vec![], None, &mut commands);
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpawnBolts>();
        assert_eq!(
            captured.0.len(),
            1,
            "SpawnBolts should dispatch SpawnBoltsFired"
        );
        assert_eq!(captured.0[0].count, 1);
        assert_eq!(captured.0[0].lifespan, None);
        assert!(!captured.0[0].inherit);
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
