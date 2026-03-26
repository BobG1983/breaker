//! Bridge for `NoBump` — fires when `BoltHitBreaker` arrives without any `BumpPerformed`.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltHitBreaker,
    breaker::{
        components::Breaker,
        messages::BumpPerformed,
    },
    effect::{
        active::ActiveEffects,
        definition::{EffectChains, Trigger},
        helpers::{evaluate_active_chains, evaluate_entity_chains},
    },
};

/// Bridge for `NoBump` — fires when `BoltHitBreaker` arrives without
/// any `BumpPerformed` in the same frame.
///
/// Evaluates active chains and breaker entity `EffectChains` with
/// `Trigger::NoBump`.
pub(crate) fn bridge_no_bump(
    mut hit_reader: MessageReader<BoltHitBreaker>,
    mut bump_reader: MessageReader<BumpPerformed>,
    active: Res<ActiveEffects>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut commands: Commands,
) {
    let hit_count = hit_reader.read().count();
    let bump_count = bump_reader.read().count();

    if hit_count == 0 || bump_count > 0 {
        return;
    }

    evaluate_active_chains(&active, Trigger::NoBump, vec![], &mut commands);

    for mut chains in &mut breaker_query {
        evaluate_entity_chains(&mut chains, Trigger::NoBump, vec![], &mut commands);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::messages::BoltHitBreaker,
        breaker::messages::{BumpGrade, BumpPerformed},
        effect::{
            active::ActiveEffects,
            definition::{Effect, EffectNode, Trigger},
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
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
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

    /// Wraps a list of `EffectNode`s as `(None, node)` tuples for `ActiveEffects`.
    fn wrap_chains(chains: Vec<EffectNode>) -> Vec<(Option<String>, EffectNode)> {
        chains.into_iter().map(|c| (None, c)).collect()
    }

    fn no_bump_test_app(active_chains: Vec<EffectNode>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(active_chains)))
            .insert_resource(SendBoltHitBreaker(None))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, send_bump, bridge_no_bump).chain(),
            );
        app
    }

    // --- NoBump bridge tests ---

    #[test]
    fn bridge_no_bump_fires_when_bolt_hits_breaker_without_bump() {
        let chain = EffectNode::trigger_leaf(Trigger::NoBump, Effect::test_shockwave(64.0));
        let mut app = no_bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();

        // BoltHitBreaker present, NO BumpPerformed
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "NoBump chain should fire when BoltHitBreaker arrives without BumpPerformed"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_no_bump_does_not_fire_when_bump_performed_also_arrives() {
        let chain = EffectNode::trigger_leaf(Trigger::NoBump, Effect::test_shockwave(64.0));
        let mut app = no_bump_test_app(vec![chain]);
        let bolt = app.world_mut().spawn_empty().id();

        // Both BoltHitBreaker AND BumpPerformed present
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "NoBump chain should NOT fire when BumpPerformed is also present"
        );
    }

    #[test]
    fn bridge_no_bump_does_not_fire_when_no_bolt_hit_breaker() {
        let chain = EffectNode::trigger_leaf(Trigger::NoBump, Effect::test_shockwave(64.0));
        let mut app = no_bump_test_app(vec![chain]);

        // Neither BoltHitBreaker nor BumpPerformed
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "NoBump chain should NOT fire when no BoltHitBreaker arrives"
        );
    }

    #[test]
    fn bridge_no_bump_evaluates_breaker_entity_effect_chains() {
        use crate::{breaker::components::Breaker, effect::definition::EffectChains};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .add_message::<BumpPerformed>()
            .insert_resource(ActiveEffects(wrap_chains(vec![])))
            .insert_resource(SendBoltHitBreaker(None))
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (send_bolt_hit_breaker, send_bump, bridge_no_bump).chain(),
            );

        // Breaker entity with NoBump EffectChain
        let _breaker = app
            .world_mut()
            .spawn((
                Breaker,
                EffectChains(vec![EffectNode::When {
                    trigger: Trigger::NoBump,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }]),
            ))
            .id();

        let bolt = app.world_mut().spawn_empty().id();

        // BoltHitBreaker present, NO BumpPerformed
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_no_bump should evaluate breaker entity EffectChains with Trigger::NoBump"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }
}
