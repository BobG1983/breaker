//! Bridge for `Trigger::NoBump` — fires when bolt hits breaker without any bump.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltHitBreaker,
    breaker::messages::BumpPerformed,
    effect::{
        definition::{EffectChains, Trigger},
        helpers::evaluate_entity_chains,
    },
};

/// Bridge for `Trigger::NoBump` — reads `BoltHitBreaker` and `BumpPerformed`.
/// If hits > 0 and bumps == 0, sweeps ALL entities with `EffectChains`
/// for `Trigger::NoBump`.
pub(crate) fn bridge_no_bump(
    mut hit_reader: MessageReader<BoltHitBreaker>,
    mut bump_reader: MessageReader<BumpPerformed>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let hits = hit_reader.read().count();
    let bumps = bump_reader.read().count();

    if hits > 0 && bumps == 0 {
        for mut chains in &mut chains_query {
            evaluate_entity_chains(&mut chains, Trigger::NoBump, vec![], &mut commands);
        }
    }
}

/// Registers bridge systems for this trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_no_bump
            .after(super::impact::bridge_breaker_impact)
            .after(super::bump::bridge_bump)
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::{
        breaker::{components::Breaker, messages::BumpGrade},
        effect::definition::{Effect, EffectNode, Trigger},
    };

    // --- Test infrastructure ---

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

    fn no_bump_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitBreaker>()
            .add_message::<BumpPerformed>()
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

    // --- Tests ---

    #[test]
    fn bridge_no_bump_sweeps_all_entities() {
        let mut app = no_bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        // Breaker entity with NoBump chain
        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::NoBump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        // Non-breaker entity with NoBump chain
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::NoBump,
                Effect::test_shockwave(32.0),
            )])));

        // BoltHitBreaker present, NO BumpPerformed
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "bridge_no_bump should sweep ALL entities with NoBump chains when bolt hits without bump"
        );
    }

    #[test]
    fn bridge_no_bump_noop_when_bump_performed() {
        let mut app = no_bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::NoBump,
                Effect::test_shockwave(64.0),
            )])),
        ));

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
            "bridge_no_bump should NOT fire when BumpPerformed is also present"
        );
    }

    /// M1: NoBump bridge does NOT evaluate ArmedEffects — bolt with armed NoBump
    /// chain does not fire when BoltHitBreaker arrives without BumpPerformed.
    #[test]
    fn bridge_no_bump_does_not_evaluate_armed_effects() {
        use crate::effect::armed::ArmedEffects;

        let mut app = no_bump_test_app();

        // Bolt entity with ArmedEffects containing When(NoBump, [Do(Shockwave(64.0))])
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::NoBump, Effect::test_shockwave(64.0)),
            )]))
            .id();

        // BoltHitBreaker present, NO BumpPerformed — NoBump condition is met
        app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "bridge_no_bump should NOT evaluate ArmedEffects — only EffectChains are swept"
        );

        // ArmedEffects should remain unchanged
        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert_eq!(
            armed.0.len(),
            1,
            "ArmedEffects should be untouched by bridge_no_bump"
        );
    }

    #[test]
    fn bridge_no_bump_noop_when_no_hit() {
        let mut app = no_bump_test_app();

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::NoBump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        // No BoltHitBreaker, no BumpPerformed
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "bridge_no_bump should NOT fire when no BoltHitBreaker arrives"
        );
    }
}
