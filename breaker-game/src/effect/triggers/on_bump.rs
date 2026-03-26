//! Bridge for `BumpPerformed` and `BumpWhiffed` — evaluates chains on bump events.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::Breaker,
        messages::{BumpGrade, BumpPerformed, BumpWhiffed},
    },
    effect::{
        active::ActiveEffects,
        armed::ArmedEffects,
        definition::{EffectChains, EffectNode, EffectTarget, Trigger},
        evaluate::{NodeEvalResult, evaluate_node},
        helpers::*,
        typed_events::fire_typed_event,
    },
};

/// Bridge for `BumpPerformed` — evaluates chains on bump.
///
/// For each bump message, evaluates two trigger kinds:
/// 1. Grade-specific: Perfect->`PerfectBump`, Early->`EarlyBump`, Late->`LateBump`
/// 2. `Bump`: all non-whiff bumps evaluate `Bump` chains.
///
/// Also evaluates breaker entity `EffectChains`.
pub(crate) fn bridge_bump(
    mut reader: MessageReader<BumpPerformed>,
    active: Res<ActiveEffects>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut bolt_chains_query: Query<&mut EffectChains, Without<Breaker>>,
    mut commands: Commands,
) {
    for performed in reader.read() {
        let grade_trigger = match performed.grade {
            BumpGrade::Perfect => Trigger::PerfectBump,
            BumpGrade::Early => Trigger::EarlyBump,
            BumpGrade::Late => Trigger::LateBump,
        };

        // Bolt-targeted evaluation requires a real bolt entity
        if let Some(bolt_entity) = performed.bolt {
            let targets = vec![EffectTarget::Entity(bolt_entity)];

            for (chip_name, chain) in &active.0 {
                // Grade-specific evaluation
                for result in evaluate_node(grade_trigger, chain) {
                    match result {
                        NodeEvalResult::Fire(effect) => {
                            fire_typed_event(
                                effect,
                                targets.clone(),
                                chip_name.clone(),
                                &mut commands,
                            );
                        }
                        NodeEvalResult::Arm(remaining) => {
                            arm_bolt(
                                &mut armed_query,
                                &mut commands,
                                bolt_entity,
                                chip_name.clone(),
                                remaining,
                            );
                        }
                        NodeEvalResult::NoMatch => {}
                    }
                }
                // BumpSuccess evaluation (all grades)
                for result in evaluate_node(Trigger::Bump, chain) {
                    match result {
                        NodeEvalResult::Fire(effect) => {
                            fire_typed_event(
                                effect,
                                targets.clone(),
                                chip_name.clone(),
                                &mut commands,
                            );
                        }
                        NodeEvalResult::Arm(remaining) => {
                            arm_bolt(
                                &mut armed_query,
                                &mut commands,
                                bolt_entity,
                                chip_name.clone(),
                                remaining,
                            );
                        }
                        NodeEvalResult::NoMatch => {}
                    }
                }
            }

            // Evaluate armed triggers on the specific bolt
            evaluate_armed(&mut armed_query, &mut commands, bolt_entity, grade_trigger);
            evaluate_armed(&mut armed_query, &mut commands, bolt_entity, Trigger::Bump);

            // Evaluate breaker entity EffectChains (bolt-targeted)
            for mut chains in &mut breaker_query {
                evaluate_entity_chains(&mut chains, grade_trigger, targets.clone(), &mut commands);
                evaluate_entity_chains(&mut chains, Trigger::Bump, targets.clone(), &mut commands);
            }

            // Evaluate bolt entity EffectChains with Bumped triggers
            if let Ok(mut bolt_chains) = bolt_chains_query.get_mut(bolt_entity) {
                let bumped_grade = match performed.grade {
                    BumpGrade::Perfect => Trigger::PerfectBumped,
                    BumpGrade::Early => Trigger::EarlyBumped,
                    BumpGrade::Late => Trigger::LateBumped,
                };
                evaluate_entity_chains(
                    &mut bolt_chains,
                    bumped_grade,
                    targets.clone(),
                    &mut commands,
                );
                evaluate_entity_chains(&mut bolt_chains, Trigger::Bumped, targets, &mut commands);
            }
        } else {
            // No bolt — still evaluate breaker entity EffectChains with empty targets
            for mut chains in &mut breaker_query {
                evaluate_entity_chains(&mut chains, grade_trigger, vec![], &mut commands);
                evaluate_entity_chains(&mut chains, Trigger::Bump, vec![], &mut commands);
            }
        }
    }
}

/// Bridge for `BumpWhiffed` — evaluates chains when a bump whiffs.
///
/// Global trigger: evaluates active chains once per frame and evaluates
/// armed triggers on ALL bolt entities.
pub(crate) fn bridge_bump_whiff(
    mut reader: MessageReader<BumpWhiffed>,
    active: Res<ActiveEffects>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = Trigger::BumpWhiff;
    evaluate_active_chains(&active, trigger_kind, vec![], &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        breaker::messages::BumpGrade,
        effect::{
            active::ActiveEffects,
            armed::ArmedEffects,
            definition::{Effect, EffectNode, ImpactTarget, Trigger},
            typed_events::*,
        },
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedShockwaveFired(Vec<ShockwaveFired>);

    fn capture_shockwave_fired(
        trigger: On<ShockwaveFired>,
        mut captured: ResMut<CapturedShockwaveFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedLoseLifeFired(Vec<LoseLifeFired>);

    fn capture_lose_life_fired(
        trigger: On<LoseLifeFired>,
        mut captured: ResMut<CapturedLoseLifeFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedShieldFired(Vec<ShieldFired>);

    fn capture_shield_fired(trigger: On<ShieldFired>, mut captured: ResMut<CapturedShieldFired>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedTimePenaltyFired(Vec<TimePenaltyFired>);

    fn capture_time_penalty_fired(
        trigger: On<TimePenaltyFired>,
        mut captured: ResMut<CapturedTimePenaltyFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSpeedBoostFired(Vec<SpeedBoostFired>);

    fn capture_speed_boost_fired(
        trigger: On<SpeedBoostFired>,
        mut captured: ResMut<CapturedSpeedBoostFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource)]
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBumpWhiffFlag(bool);

    fn send_bump_whiff(flag: Res<SendBumpWhiffFlag>, mut writer: MessageWriter<BumpWhiffed>) {
        if flag.0 {
            writer.write(BumpWhiffed);
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Wraps a list of `EffectNode`s as `(None, node)` tuples for `ActiveEffects`.
    fn wrap_chains(chains: Vec<EffectNode>) -> Vec<(Option<String>, EffectNode)> {
        chains.into_iter().map(|c| (None, c)).collect()
    }

    fn bump_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedShieldFired>()
            .init_resource::<CapturedLoseLifeFired>()
            .init_resource::<CapturedTimePenaltyFired>()
            .add_observer(capture_shockwave_fired)
            .add_observer(capture_shield_fired)
            .add_observer(capture_lose_life_fired)
            .add_observer(capture_time_penalty_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());
        app
    }

    fn bump_whiff_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpWhiffed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBumpWhiffFlag(false))
            .init_resource::<CapturedLoseLifeFired>()
            .add_observer(capture_lose_life_fired)
            .add_systems(FixedUpdate, (send_bump_whiff, bridge_bump_whiff).chain());
        app
    }

    // --- Bump bridge tests ---

    #[test]
    fn perfect_bump_fires_on_perfect_bump_chain() {
        let chain = EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::test_shockwave(64.0));
        let mut app = bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
        assert_eq!(captured.0[0].targets, vec![EffectTarget::Entity(bolt)]);
    }

    #[test]
    fn perfect_bump_fires_both_on_perfect_bump_and_on_bump_success() {
        let chains = vec![
            EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::test_shockwave(64.0)),
            EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shield(3.0)),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        let shields = app.world().resource::<CapturedShieldFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "perfect bump should fire ShockwaveFired from PerfectBump chain"
        );
        assert_eq!(
            shields.0.len(),
            1,
            "perfect bump should fire ShieldFired from Bump chain"
        );
    }

    #[test]
    fn early_bump_fires_on_early_bump_and_on_bump_success_but_not_on_perfect_bump() {
        let chains = vec![
            EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::test_shockwave(64.0)),
            EffectNode::trigger_leaf(Trigger::EarlyBump, Effect::LoseLife),
            EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shield(3.0)),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let lose_life = app.world().resource::<CapturedLoseLifeFired>();
        let shields = app.world().resource::<CapturedShieldFired>();
        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            lose_life.0.len(),
            1,
            "early bump should fire LoseLifeFired from EarlyBump chain"
        );
        assert_eq!(
            shields.0.len(),
            1,
            "early bump should fire ShieldFired from Bump chain"
        );
        assert!(
            shockwaves.0.is_empty(),
            "early bump should NOT fire ShockwaveFired from PerfectBump chain"
        );
    }

    #[test]
    fn late_bump_fires_on_late_bump_and_on_bump_success() {
        let chains = vec![
            EffectNode::trigger_leaf(Trigger::LateBump, Effect::test_time_penalty(3.0)),
            EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shield(3.0)),
        ];
        let mut app = bump_test_app(chains);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Late,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let time_penalty = app.world().resource::<CapturedTimePenaltyFired>();
        let shields = app.world().resource::<CapturedShieldFired>();
        assert_eq!(time_penalty.0.len(), 1);
        assert!((time_penalty.0[0].seconds - 3.0).abs() < f32::EPSILON);
        assert_eq!(shields.0.len(), 1);
    }

    #[test]
    fn perfect_bump_with_non_leaf_arms_bolt() {
        let chain = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }],
        };
        let mut app = bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(captured.0.is_empty(), "non-leaf inner should arm, not fire");

        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert_eq!(armed.0.len(), 1);
        assert_eq!(
            armed.0[0].1,
            EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))]
            }
        );
    }

    // --- BumpWhiff bridge tests ---

    #[test]
    fn bump_whiff_fires_on_bump_whiff_chain() {
        let chain = EffectNode::trigger_leaf(Trigger::BumpWhiff, Effect::LoseLife);
        let mut app = bump_whiff_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBumpWhiffFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert_eq!(captured.0.len(), 1);
        assert!(captured.0[0].targets.is_empty());
    }

    #[test]
    fn bump_whiff_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(Trigger::BumpWhiff, Effect::LoseLife);
        let mut app = bump_whiff_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert!(captured.0.is_empty());
    }

    // --- Behavior 28: bridge_bump evaluates breaker entity EffectChains ---

    #[test]
    fn bridge_bump_evaluates_breaker_effect_chains() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        target: crate::effect::definition::Target::Bolt,
                        multiplier: 1.2,
                    })],
                }]),
            ))
            .id();

        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "breaker EffectChains should be evaluated by bridge_bump"
        );
        assert!((captured.0[0].multiplier - 1.2).abs() < f32::EPSILON);
    }

    // --- Bumped split in bridge_bump tests ---

    #[test]
    fn bridge_bump_fires_perfect_bumped_on_bolt_entity_effect_chains() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Bolt entity with PerfectBumped EffectChain
        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]))
            .id();

        // Spawn breaker entity so breaker_query has something
        app.world_mut().spawn((Breaker, EffectChains::default()));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "PerfectBumped chain on bolt entity should fire on Perfect bump"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_bump_fires_generic_bumped_on_bolt_entity_for_any_grade() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Bolt entity with generic Bumped EffectChain
        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]))
            .id();

        // Spawn breaker entity so breaker_query has something
        app.world_mut().spawn((Breaker, EffectChains::default()));

        // Send Early bump — Bumped should match any non-whiff grade
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "Bumped chain on bolt entity should fire for Early grade (matches any non-whiff)"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_bump_perfect_bumped_does_not_fire_on_breaker_entity() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Breaker entity with PerfectBumped EffectChain — should NOT fire
        // (PerfectBumped is bolt-perspective only)
        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]),
        ));

        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "PerfectBumped on breaker entity should NOT fire — Bumped is bolt-perspective only"
        );
    }

    #[test]
    fn bridge_bump_perfect_bump_still_fires_on_breaker_entity() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());

        // Breaker entity with PerfectBump (breaker-perspective) EffectChain
        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]),
        ));

        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "PerfectBump (breaker-perspective) on breaker entity should still fire"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }
}
