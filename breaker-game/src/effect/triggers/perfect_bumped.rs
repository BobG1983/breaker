//! Bridge for `Trigger::PerfectBumped` — evaluates specific bolt's chains on perfect bump.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::bridge_targeted_bumped_inner,
    },
};

/// Bridge for `Trigger::PerfectBumped` — reads `BumpPerformed`, filters
/// `Perfect` grade, then evaluates ONLY the specific bolt entity's
/// `EffectChains` and `ArmedEffects` for `Trigger::PerfectBumped`.
pub(crate) fn bridge_perfect_bumped(
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
        Some(BumpGrade::Perfect),
        Trigger::PerfectBumped,
    );
}

/// Registers bridge systems for this trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{breaker::BreakerSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_perfect_bumped
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

    fn perfect_bumped_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_perfect_bumped).chain());
        app
    }

    // --- Tests ---

    #[test]
    fn bridge_perfect_bumped_fires_on_specific_bolt() {
        let mut app = perfect_bumped_test_app();

        // Bolt with PerfectBumped chain
        let bolt = app
            .world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::PerfectBumped,
                Effect::test_shockwave(64.0),
            )])))
            .id();

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_perfect_bumped should fire PerfectBumped on the specific bolt"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_perfect_bumped_no_fire_on_early() {
        let mut app = perfect_bumped_test_app();

        let bolt = app
            .world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::PerfectBumped,
                Effect::test_shockwave(64.0),
            )])))
            .id();

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "bridge_perfect_bumped should NOT fire on Early grade"
        );
    }
}
