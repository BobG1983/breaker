//! Bridge for `Trigger::Bump` — evaluates chains on any non-whiff bump.

use bevy::prelude::*;

use crate::{
    breaker::messages::BumpPerformed,
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::bridge_global_bump_inner,
    },
};

/// Bridge for `Trigger::Bump` — reads `BumpPerformed` (any grade) and sweeps
/// ALL entities with `EffectChains` for `Trigger::Bump`. Also evaluates
/// `ArmedEffects` on the specific bolt.
pub(crate) fn bridge_bump(
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
        None,
        Trigger::Bump,
    );
}

/// Registers bridge systems for this trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{breaker::BreakerSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_bump
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

    fn bump_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump, bridge_bump).chain());
        app
    }

    // --- Tests ---

    #[test]
    fn bridge_bump_sweeps_all_entities() {
        let mut app = bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        // Breaker entity with When(Bump) chain
        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::Bump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        // Non-breaker entity with When(Bump) chain
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::Bump,
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
            "bridge_bump should sweep ALL entities with EffectChains for Trigger::Bump"
        );
    }

    #[test]
    fn bridge_bump_evaluates_armed_on_bolt() {
        let mut app = bump_test_app();

        // Bolt entity with ArmedEffects containing When(Bump) chain
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shockwave(64.0)),
            )]))
            .id();

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Early,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_bump should evaluate ArmedEffects on the specific bolt for Trigger::Bump"
        );
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    /// M12: `chip_name` propagation through chains — `EffectChains` entry with
    /// Some("Surge") produces `ShockwaveFired` with `source_chip`: Some("Surge").
    #[test]
    fn bridge_bump_propagates_chip_name_to_shockwave_fired() {
        let mut app = bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        // Entity with EffectChains using chip_name "Surge"
        app.world_mut().spawn(EffectChains(vec![(
            Some("Surge".to_owned()),
            EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shockwave(64.0)),
        )]));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "one shockwave should fire — got {}",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].source_chip,
            Some("Surge".to_owned()),
            "ShockwaveFired.source_chip should carry the chip_name from EffectChains"
        );
    }

    #[test]
    fn bridge_bump_does_not_fire_grade_specific() {
        let mut app = bump_test_app();
        let bolt = app.world_mut().spawn_empty().id();

        // Entity with When(PerfectBump) chain — should NOT fire from bridge_bump
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::PerfectBump,
                Effect::test_shockwave(64.0),
            )])));

        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "bridge_bump should NOT fire grade-specific triggers like PerfectBump"
        );
    }

    #[test]
    fn bridge_bump_none_bolt_still_sweeps() {
        let mut app = bump_test_app();

        // Breaker entity with When(Bump) chain
        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::Bump,
                Effect::test_shockwave(64.0),
            )])),
        ));

        // BumpPerformed with bolt: None
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Late,
            bolt: None,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bridge_bump should still sweep EffectChains even when bolt is None"
        );
    }
}
