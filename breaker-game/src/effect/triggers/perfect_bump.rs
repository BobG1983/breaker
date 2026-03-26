//! Bridge for `Trigger::PerfectBump` — evaluates chains on perfect-grade bumps only.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::bridge_global_bump_inner,
    },
};

/// Bridge for `Trigger::PerfectBump` — reads `BumpPerformed`, filters
/// `Perfect` grade, then sweeps ALL entities with `EffectChains` for
/// `Trigger::PerfectBump`.
pub(crate) fn bridge_perfect_bump(
    mut reader: MessageReader<BumpPerformed>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    bridge_global_bump_inner(
        &mut reader,
        &mut chains_query,
        &mut armed_query,
        &mut commands,
        Some(BumpGrade::Perfect),
        Trigger::PerfectBump,
    );
}

/// Registers bridge systems for this trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{breaker::BreakerSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_perfect_bump
            .after(BreakerSystems::GradeBump)
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
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn perfect_bump_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_perfect_bump).chain());
        app
    }

    // --- Tests ---

    #[test]
    fn bridge_perfect_bump_sweeps_all_entities() {
        let mut app = perfect_bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        // Breaker entity with PerfectBump chain
        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::PerfectBump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        // Non-breaker entity with PerfectBump chain
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::PerfectBump,
                Effect::test_shockwave(32.0),
            )])));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "bridge_perfect_bump should sweep ALL entities with PerfectBump chains on Perfect grade"
        );
    }

    #[test]
    fn bridge_perfect_bump_no_fire_on_early() {
        let mut app = perfect_bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::PerfectBump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "bridge_perfect_bump should NOT fire on Early grade"
        );
    }
}
