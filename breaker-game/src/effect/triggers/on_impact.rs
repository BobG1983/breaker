//! Bridge systems for impact events — cell, breaker, and wall impacts.

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, EffectTarget, ImpactTarget, Trigger},
        effect_nodes::until::{UntilTimers, UntilTriggers},
        helpers::*,
    },
};

/// Bridge for `BoltHitCell` — evaluates armed triggers on cell impact.
/// Also evaluates bolt entity `EffectChains` and `When` children inside
/// `UntilTimers`/`UntilTriggers`.
pub(crate) fn bridge_cell_impact(
    mut reader: MessageReader<BoltHitCell>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Cell),
        );

        // Evaluate bolt entity EffectChains
        if let Ok(mut chains) = chains_query.get_mut(bolt_entity) {
            evaluate_entity_chains(
                &mut chains,
                Trigger::Impact(ImpactTarget::Cell),
                targets.clone(),
                &mut commands,
            );
        }

        // Evaluate When children inside UntilTimers/UntilTriggers
        evaluate_until_children(
            &until_query,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Cell),
            &targets,
            &mut commands,
        );
    }
}

/// Bridge for `BoltHitBreaker` — evaluates armed triggers on
/// breaker impact. Also evaluates bolt entity `EffectChains`.
pub(crate) fn bridge_breaker_impact(
    mut reader: MessageReader<BoltHitBreaker>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Breaker),
        );

        // Evaluate bolt entity EffectChains
        if let Ok(mut chains) = chains_query.get_mut(bolt_entity) {
            evaluate_entity_chains(
                &mut chains,
                Trigger::Impact(ImpactTarget::Breaker),
                targets,
                &mut commands,
            );
        }
    }
}

/// Bridge for `BoltHitWall` — evaluates armed triggers on
/// wall impact. Also evaluates bolt entity `EffectChains` and `When`
/// children inside `UntilTimers`/`UntilTriggers`.
pub(crate) fn bridge_wall_impact(
    mut reader: MessageReader<BoltHitWall>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    until_query: Query<(Option<&UntilTimers>, Option<&UntilTriggers>)>,
    mut commands: Commands,
) {
    for hit in reader.read() {
        let bolt_entity = hit.bolt;
        let targets = vec![EffectTarget::Entity(bolt_entity)];

        evaluate_armed(
            &mut armed_query,
            &mut commands,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Wall),
        );

        // Evaluate bolt entity EffectChains
        if let Ok(mut chains) = chains_query.get_mut(bolt_entity) {
            evaluate_entity_chains(
                &mut chains,
                Trigger::Impact(ImpactTarget::Wall),
                targets.clone(),
                &mut commands,
            );
        }

        // Evaluate When children inside UntilTimers/UntilTriggers
        evaluate_until_children(
            &until_query,
            bolt_entity,
            Trigger::Impact(ImpactTarget::Wall),
            &targets,
            &mut commands,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
        effect::{
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
    struct CapturedShieldFired(Vec<ShieldFired>);

    fn capture_shield_fired(trigger: On<ShieldFired>, mut captured: ResMut<CapturedShieldFired>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedMultiBoltFired(Vec<MultiBoltFired>);

    fn capture_multi_bolt_fired(
        trigger: On<MultiBoltFired>,
        mut captured: ResMut<CapturedMultiBoltFired>,
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
    struct SendBoltHitCell(Option<BoltHitCell>);

    fn send_bolt_hit_cell(msg: Res<SendBoltHitCell>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBoltHitBreaker(Option<BoltHitBreaker>);

    fn send_bolt_hit_breaker(
        msg: Res<SendBoltHitBreaker>,
        mut writer: MessageWriter<BoltHitBreaker>,
    ) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBoltHitWall(Option<BoltHitWall>);

    fn send_bolt_hit_wall(msg: Res<SendBoltHitWall>, mut writer: MessageWriter<BoltHitWall>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Wraps a list of `EffectNode`s as `(None, node)` tuples for `EffectChains`.
    fn wrap_chains(chains: Vec<EffectNode>) -> Vec<(Option<String>, EffectNode)> {
        chains.into_iter().map(|c| (None, c)).collect()
    }

    fn cell_impact_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );
        app
    }

    fn breaker_impact_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .insert_resource(SendBoltHitBreaker(None))
            .init_resource::<CapturedShieldFired>()
            .init_resource::<CapturedMultiBoltFired>()
            .add_observer(capture_shield_fired)
            .add_observer(capture_multi_bolt_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, bridge_breaker_impact).chain(),
            );
        app
    }

    fn wall_impact_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedShieldFired>()
            .add_observer(capture_shockwave_fired)
            .add_observer(capture_shield_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );
        app
    }

    // --- Cell impact bridge tests ---

    #[test]
    fn cell_impact_fires_entity_chain() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Cell),
            Effect::test_shockwave(64.0),
        );
        let mut app = cell_impact_test_app();
        // Place chain on bolt entity EffectChains
        let bolt = app.world_mut().spawn(EffectChains(vec![(None, chain)])).id();
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_impact_fires_armed_trigger() {
        let mut app = cell_impact_test_app();
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Cell),
                    Effect::test_shockwave(64.0),
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);

        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert!(armed.0.is_empty());
    }

    #[test]
    fn cell_impact_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Cell),
            Effect::test_shockwave(64.0),
        );
        let mut app = cell_impact_test_app();
        app.world_mut().spawn(EffectChains(vec![(None, chain)]));
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(captured.0.is_empty());
    }

    // --- Breaker impact bridge tests ---

    #[test]
    fn breaker_impact_fires_entity_chain() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Breaker),
            Effect::test_shield(5.0),
        );
        let mut app = breaker_impact_test_app();
        // Place chain on bolt entity EffectChains
        let bolt = app.world_mut().spawn(EffectChains(vec![(None, chain)])).id();
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShieldFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_duration - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn breaker_impact_fires_armed_trigger() {
        let mut app = breaker_impact_test_app();
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Breaker),
                    Effect::test_multi_bolt(2),
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedMultiBoltFired>();
        assert_eq!(captured.0.len(), 1);
        assert_eq!(captured.0[0].base_count, 2);
    }

    // --- Wall impact bridge tests ---

    #[test]
    fn wall_impact_fires_entity_chain() {
        let chain = EffectNode::trigger_leaf(
            Trigger::Impact(ImpactTarget::Wall),
            Effect::test_shockwave(32.0),
        );
        let mut app = wall_impact_test_app();
        // Place chain on bolt entity EffectChains
        let bolt = app.world_mut().spawn(EffectChains(vec![(None, chain)])).id();
        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 32.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wall_impact_fires_armed_trigger() {
        let mut app = wall_impact_test_app();
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(
                    Trigger::Impact(ImpactTarget::Wall),
                    Effect::test_shield(5.0),
                ),
            )]))
            .id();
        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShieldFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_duration - 5.0).abs() < f32::EPSILON);
    }

    // --- Behavior 23: bridge_cell_impact evaluates bolt entity EffectChains ---

    #[test]
    fn bridge_cell_impact_evaluates_bolt_effect_chains() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(None, EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })],
            })]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bolt entity EffectChains should be evaluated by bridge_cell_impact"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    // --- Behavior 26: bridge_breaker_impact evaluates bolt entity EffectChains ---

    #[test]
    fn bridge_breaker_impact_evaluates_bolt_effect_chains() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .insert_resource(SendBoltHitBreaker(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, bridge_breaker_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(None, EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Breaker),
                then: vec![EffectNode::Do(Effect::SpeedBoost {
                    multiplier: 1.5,
                })],
            })]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bolt entity EffectChains should be evaluated by bridge_breaker_impact"
        );
        assert!((captured.0[0].multiplier - 1.5).abs() < f32::EPSILON);
    }

    // --- Behavior 27: bridge_wall_impact evaluates bolt entity EffectChains ---

    #[test]
    fn bridge_wall_impact_evaluates_bolt_effect_chains() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(None, EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Wall),
                then: vec![EffectNode::Do(Effect::SpeedBoost {
                    multiplier: 1.3,
                })],
            })]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bolt entity EffectChains should be evaluated by bridge_wall_impact"
        );
        assert!((captured.0[0].multiplier - 1.3).abs() < f32::EPSILON);
    }

    // --- Once node bridge evaluation ---

    #[test]
    fn bridge_cell_impact_unwraps_once_when_inner_when_matches() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }]),
            )]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge should unwrap Once, evaluate inner When(Impact(Cell)), and fire Shockwave"
        );

        let chains = app.world().get::<EffectChains>(bolt).unwrap();
        assert!(
            chains.0.is_empty(),
            "Once node should be consumed (removed from EffectChains)"
        );
    }

    #[test]
    fn bridge_cell_impact_preserves_once_when_inner_when_does_not_match() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Wall), // Does NOT match CellImpact
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        multiplier: 2.0,
                    })],
                }]),
            )]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert!(
            captured.0.is_empty(),
            "inner When(Impact(Wall)) should NOT match CellImpact trigger"
        );

        let chains = app.world().get::<EffectChains>(bolt).unwrap();
        assert_eq!(
            chains.0.len(),
            1,
            "Once node should be preserved when inner When does not match"
        );
    }

    // --- Multiple chains on entity EffectChains both fire ---

    #[test]
    fn bridge_cell_impact_fires_multiple_entity_effect_chains() {
        use crate::effect::definition::EffectChains;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_shockwave_fired)
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt with two chains on entity EffectChains: Shockwave + SpeedBoost
        let bolt = app
            .world_mut()
            .spawn(EffectChains(vec![
                (Some("chip1".to_string()), EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }),
                (None, EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        multiplier: 1.5,
                    })],
                }),
            ]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        // Both chains on entity EffectChains should fire
        let shockwaves = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            shockwaves.0.len(),
            1,
            "Shockwave chain on entity EffectChains should fire on cell impact"
        );
        assert!((shockwaves.0[0].base_range - 64.0).abs() < f32::EPSILON);

        let speed_boosts = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            speed_boosts.0.len(),
            1,
            "SpeedBoost chain on entity EffectChains should also fire on cell impact"
        );
        assert!((speed_boosts.0[0].multiplier - 1.5).abs() < f32::EPSILON);
    }

    // --- UntilTimers/UntilTriggers bridge evaluation ---

    #[test]
    fn bridge_cell_impact_evaluates_until_timer_children() {
        use crate::effect::effect_nodes::until::{UntilTimerEntry, UntilTimers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt with UntilTimers containing a When(Impact(Cell)) child
        let bolt = app
            .world_mut()
            .spawn(UntilTimers(vec![UntilTimerEntry {
                remaining: 3.0, // Still active (not expired)
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_cell_impact should evaluate When children inside UntilTimers entries"
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "fired shockwave should have base_range 64.0, got {}",
            captured.0[0].base_range
        );
    }

    #[test]
    fn bridge_cell_impact_does_not_fire_expired_until_children() {
        // Bolt entity with NO UntilTimers component (all timers expired).
        // BoltHitCell message written.
        // No ShockwaveFired event expected.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt without UntilTimers — all timers have already expired
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "no ShockwaveFired should be emitted when Until has expired (no UntilTimers component)"
        );
    }

    #[test]
    fn bridge_cell_impact_evaluates_until_trigger_children() {
        use crate::effect::effect_nodes::until::{UntilTriggerEntry, UntilTriggers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_cell, bridge_cell_impact).chain(),
            );

        // Bolt with UntilTriggers entry (trigger: Impact(Breaker)) containing
        // When(Impact(Cell), [Do(Shockwave)]). The Until itself has NOT expired
        // (trigger is Impact(Breaker), and we're hitting a Cell), so nested
        // children should still be active and evaluated.
        let bolt = app
            .world_mut()
            .spawn(UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Breaker),
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 48.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_cell_impact should evaluate When children inside UntilTriggers entries"
        );
        assert!(
            (captured.0[0].base_range - 48.0).abs() < f32::EPSILON,
            "fired shockwave should have base_range 48.0, got {}",
            captured.0[0].base_range
        );
    }

    #[test]
    fn bridge_wall_impact_evaluates_until_timer_children() {
        use crate::effect::effect_nodes::until::{UntilTimerEntry, UntilTimers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitWall>()
            .insert_resource(SendBoltHitWall(None))
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_wall, bridge_wall_impact).chain(),
            );

        // Bolt with UntilTimers containing a When(Impact(Wall)) child
        let bolt = app
            .world_mut()
            .spawn(UntilTimers(vec![UntilTimerEntry {
                remaining: 3.0, // Still active
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Wall),
                    then: vec![EffectNode::Do(Effect::SpeedBoost {
                        multiplier: 1.3,
                    })],
                }],
            }]))
            .id();

        app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt });

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_wall_impact should evaluate When children inside UntilTimers entries"
        );
        assert!(
            (captured.0[0].multiplier - 1.3).abs() < f32::EPSILON,
            "fired SpeedBoost should have multiplier 1.3, got {}",
            captured.0[0].multiplier
        );
    }
}
