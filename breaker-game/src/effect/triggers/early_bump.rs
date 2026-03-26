//! Bridge for `Trigger::EarlyBump` — evaluates chains on early-grade bumps only.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::bridge_global_bump_inner,
    },
};

/// Bridge for `Trigger::EarlyBump` — reads `BumpPerformed`, filters
/// `Early` grade, then sweeps ALL entities with `EffectChains` for
/// `Trigger::EarlyBump`.
pub(crate) fn bridge_early_bump(
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
        Some(BumpGrade::Early),
        Trigger::EarlyBump,
    );
}

/// Registers bridge systems for this trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{breaker::BreakerSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_early_bump
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

    fn early_bump_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_early_bump).chain());
        app
    }

    // --- Tests ---

    #[test]
    fn bridge_early_bump_fires_on_early_grade() {
        let mut app = early_bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::EarlyBump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_early_bump should fire on Early grade"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bridge_early_bump_no_fire_on_perfect() {
        let mut app = early_bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::EarlyBump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "bridge_early_bump should NOT fire on Perfect grade"
        );
    }
}
