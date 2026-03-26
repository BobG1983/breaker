//! Bridge for `BoltLost` — evaluates chains when a bolt is lost.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltLost,
    breaker::components::Breaker,
    effect::{
        active::ActiveEffects,
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::{evaluate_active_chains, evaluate_armed_all, evaluate_entity_chains},
    },
};

/// Bridge for `BoltLost` — evaluates chains when a bolt is lost.
///
/// Global trigger: evaluates active chains once per frame (not per message)
/// and evaluates armed triggers on ALL bolt entities.
/// Also evaluates breaker entity `EffectChains` (for `SecondWind` etc.).
pub(crate) fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    active: Res<ActiveEffects>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = Trigger::BoltLost;
    evaluate_active_chains(&active, trigger_kind, vec![], &mut commands);
    evaluate_armed_all(armed_query, trigger_kind, &mut commands);

    // Evaluate breaker entity EffectChains
    for mut chains in &mut breaker_query {
        evaluate_entity_chains(&mut chains, trigger_kind, vec![], &mut commands);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::messages::BoltLost,
        effect::{
            active::ActiveEffects,
            definition::{Effect, EffectNode, Trigger},
            typed_events::*,
        },
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedLoseLifeFired(Vec<LoseLifeFired>);

    fn capture_lose_life_fired(
        trigger: On<LoseLifeFired>,
        mut captured: ResMut<CapturedLoseLifeFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedShockwaveFired(Vec<ShockwaveFired>);

    fn capture_shockwave_fired(
        trigger: On<ShockwaveFired>,
        mut captured: ResMut<CapturedShockwaveFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource)]
    struct SendBoltLostFlag(bool);

    fn send_bolt_lost(flag: Res<SendBoltLostFlag>, mut writer: MessageWriter<BoltLost>) {
        if flag.0 {
            writer.write(BoltLost);
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

    fn bolt_lost_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedLoseLifeFired>()
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_lose_life_fired)
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());
        app
    }

    // --- Bolt lost bridge tests ---

    #[test]
    fn bolt_lost_fires_active_chains() {
        let chain = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        let mut app = bolt_lost_test_app(vec![chain]);
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert_eq!(captured.0.len(), 1);
        assert!(captured.0[0].targets.is_empty());
    }

    #[test]
    fn bolt_lost_no_message_no_fire() {
        let chain = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        let mut app = bolt_lost_test_app(vec![chain]);
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert!(captured.0.is_empty());
    }

    // --- Integration: bridge + effect observer ---

    #[test]
    fn bridge_bolt_lost_plus_life_lost_observer_decrements_lives() {
        use crate::{
            effect::effects::life_lost::{LivesCount, handle_life_lost},
            run::messages::RunLost,
        };

        let chain = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .add_message::<RunLost>()
            .insert_resource(ActiveEffects(vec![(None, chain)]))
            .insert_resource(SendBoltLostFlag(false))
            .add_observer(handle_life_lost)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());

        let entity = app.world_mut().spawn(LivesCount(3)).id();
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 2,
            "bolt lost should decrement LivesCount via unified bridge"
        );
    }

    // --- Behavior 25: bridge_bolt_lost evaluates breaker entity EffectChains ---

    #[test]
    fn bridge_bolt_lost_evaluates_breaker_effect_chains_once_consumed() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        #[derive(Resource, Default)]
        struct CapturedSecondWindFired(Vec<SecondWindFired>);

        fn capture_second_wind(
            trigger: On<SecondWindFired>,
            mut captured: ResMut<CapturedSecondWindFired>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedSecondWindFired>()
            .add_observer(capture_second_wind)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());

        // Breaker with Once([When(BoltLost, [Do(SecondWind)])])
        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![(
                    None,
                    EffectNode::Once(vec![EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(Effect::SecondWind { invuln_secs: 3.0 })],
                    }]),
                )]),
            ))
            .id();

        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedSecondWindFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "breaker EffectChains should be evaluated by bridge_bolt_lost"
        );
        assert!((captured.0[0].invuln_secs - 3.0).abs() < f32::EPSILON);
    }

    // --- Behavior 19/25: Once consumed prevents second firing ---

    #[test]
    fn bridge_bolt_lost_once_consumed_second_bolt_lost_does_not_fire() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        #[derive(Resource, Default)]
        struct CapturedSecondWindFired2(Vec<SecondWindFired>);

        fn capture_second_wind2(
            trigger: On<SecondWindFired>,
            mut captured: ResMut<CapturedSecondWindFired2>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedSecondWindFired2>()
            .add_observer(capture_second_wind2)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());

        // Breaker with Once([When(BoltLost, [Do(SecondWind { invuln_secs: 3.0 })])])
        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![(
                    None,
                    EffectNode::Once(vec![EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(Effect::SecondWind { invuln_secs: 3.0 })],
                    }]),
                )]),
            ))
            .id();

        // First bolt lost — should fire SecondWind
        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedSecondWindFired2>();
        assert_eq!(
            captured.0.len(),
            1,
            "first BoltLost should fire SecondWind from Once wrapper"
        );

        // Second bolt lost — should NOT fire again (Once consumed)
        tick(&mut app);

        let captured = app.world().resource::<CapturedSecondWindFired2>();
        assert_eq!(
            captured.0.len(),
            1,
            "second BoltLost should NOT fire SecondWind — Once was consumed"
        );
    }
}
