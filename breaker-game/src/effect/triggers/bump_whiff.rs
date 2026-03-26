//! Bridge for `Trigger::BumpWhiff` — evaluates chains when a bump whiffs.

use bevy::prelude::*;

use crate::{
    breaker::messages::BumpWhiffed,
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::{evaluate_armed_all, evaluate_entity_chains},
    },
};

/// Bridge for `Trigger::BumpWhiff` — reads `BumpWhiffed` and sweeps ALL
/// entities with `EffectChains` for `Trigger::BumpWhiff`. Also evaluates
/// `ArmedEffects` on all bolt entities.
pub(crate) fn bridge_bump_whiff(
    mut reader: MessageReader<BumpWhiffed>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }

    for mut chains in &mut chains_query {
        evaluate_entity_chains(&mut chains, Trigger::BumpWhiff, vec![], &mut commands);
    }

    evaluate_armed_all(armed_query, Trigger::BumpWhiff, &mut commands);
}

/// Registers bridge systems for this trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{breaker::BreakerSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_bump_whiff
            .after(BreakerSystems::GradeBump)
            .in_set(EffectSystems::Bridge)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::*, *};
    use crate::{
        breaker::components::Breaker,
        effect::{
            armed::ArmedEffects,
            definition::{Effect, EffectChains, EffectNode, Trigger},
        },
    };

    // --- Test infrastructure ---

    #[derive(Resource)]
    struct SendBumpWhiffFlag(bool);

    fn send_bump_whiff(flag: Res<SendBumpWhiffFlag>, mut writer: MessageWriter<BumpWhiffed>) {
        if flag.0 {
            writer.write(BumpWhiffed);
        }
    }

    fn bump_whiff_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpWhiffed>()
            .insert_resource(SendBumpWhiffFlag(false))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bump_whiff, bridge_bump_whiff).chain());
        app
    }

    // --- Tests ---

    #[test]
    fn bridge_bump_whiff_sweeps_all_entities() {
        let mut app = bump_whiff_test_app();

        // Breaker entity with BumpWhiff chain
        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::BumpWhiff,
                Effect::test_shockwave(64.0),
            )])),
        ));

        // Non-breaker entity with BumpWhiff chain
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::BumpWhiff,
                Effect::test_shockwave(32.0),
            )])));

        app.world_mut().resource_mut::<SendBumpWhiffFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "bridge_bump_whiff should sweep ALL entities with BumpWhiff chains"
        );
    }

    /// M2: BumpWhiff passes empty targets vec to ShockwaveFired.
    #[test]
    fn bridge_bump_whiff_passes_empty_targets() {
        let mut app = bump_whiff_test_app();

        // Entity with BumpWhiff chain
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![EffectNode::trigger_leaf(
                Trigger::BumpWhiff,
                Effect::test_shockwave(64.0),
            )])));

        app.world_mut().resource_mut::<SendBumpWhiffFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1, "one shockwave should fire");
        assert!(
            captured.0[0].targets.is_empty(),
            "BumpWhiff bridge should pass empty targets vec — got {:?}",
            captured.0[0].targets
        );
    }

    #[test]
    fn bridge_bump_whiff_evaluates_armed_on_all_bolts() {
        let mut app = bump_whiff_test_app();

        // Two bolt entities with ArmedEffects containing BumpWhiff chains
        app.world_mut().spawn(ArmedEffects(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::BumpWhiff, Effect::test_shockwave(64.0)),
        )]));

        app.world_mut().spawn(ArmedEffects(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::BumpWhiff, Effect::test_shockwave(32.0)),
        )]));

        app.world_mut().resource_mut::<SendBumpWhiffFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "bridge_bump_whiff should evaluate ArmedEffects on all bolt entities"
        );
    }
}
