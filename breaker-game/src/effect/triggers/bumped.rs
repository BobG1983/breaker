//! Bridge for `Trigger::Bumped` — evaluates ONLY the specific bolt's chains on any bump.

use bevy::prelude::*;

use crate::{
    breaker::messages::BumpPerformed,
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::bridge_targeted_bumped_inner,
    },
};

/// Bridge for `Trigger::Bumped` — reads `BumpPerformed` (any grade) and
/// evaluates ONLY the specific bolt entity's `EffectChains` and
/// `ArmedEffects` for `Trigger::Bumped`.
pub(crate) fn bridge_bumped(
    mut reader: MessageReader<BumpPerformed>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    bridge_targeted_bumped_inner(
        &mut reader,
        &mut chains_query,
        &mut armed_query,
        &mut commands,
        None,
        Trigger::Bumped,
    );
}

/// Registers bridge systems for this trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{breaker::BreakerSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_bumped
            .after(BreakerSystems::GradeBump)
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::{
        breaker::messages::BumpGrade,
        effect::definition::{Effect, EffectNode, Trigger},
    };

    // --- Test infrastructure ---

    #[derive(Resource)]
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn bumped_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bumped).chain());
        app
    }

    // --- Tests ---

    #[test]
    fn bridge_bumped_evaluates_specific_bolt_only() {
        let mut app = bumped_test_app();

        // Bolt A has When(Bumped) chain
        let bolt_a = app
            .world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::Bumped,
                Effect::test_shockwave(64.0),
            )])))
            .id();

        // Bolt B has When(Bumped) chain
        let _bolt_b = app
            .world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::Bumped,
                Effect::test_shockwave(32.0),
            )])))
            .id();

        // BumpPerformed targets bolt A only
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_a),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_bumped should only evaluate the specific bolt's EffectChains"
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "should fire bolt A's chain (range 64.0), not bolt B's (range 32.0)"
        );
    }

    #[test]
    fn bridge_bumped_none_bolt_is_noop() {
        let mut app = bumped_test_app();

        // Bolt with When(Bumped) chain
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::Bumped,
                Effect::test_shockwave(64.0),
            )])));

        // BumpPerformed with bolt: None
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "bridge_bumped should be a noop when bolt is None"
        );
    }
}
