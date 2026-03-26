//! Bridge for `BoltLost` — sweeps ALL entities with `EffectChains` for
//! `Trigger::BoltLost`, evaluates `ArmedEffects` on all bolt entities.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltLost,
    effect::{
        armed::ArmedEffects,
        definition::{EffectChains, Trigger},
        helpers::{evaluate_armed_all, evaluate_entity_chains},
    },
};

/// Bridge for `BoltLost` — sweeps ALL entities with `EffectChains` for
/// `Trigger::BoltLost`. Also evaluates `ArmedEffects` on all bolt entities.
///
/// FIX from old bridge: was breaker-only; now sweeps all entities.
pub(crate) fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    armed_query: Query<(Entity, &mut ArmedEffects)>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    let trigger_kind = Trigger::BoltLost;

    for mut chains in &mut chains_query {
        evaluate_entity_chains(&mut chains, trigger_kind, vec![], &mut commands);
    }

    evaluate_armed_all(armed_query, trigger_kind, &mut commands);
}

/// Registers bridge systems for bolt lost trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{bolt::BoltSystems, effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_bolt_lost
            .after(BoltSystems::BoltLost)
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

    #[derive(Resource)]
    struct SendBoltLostFlag(bool);

    fn send_bolt_lost(flag: Res<SendBoltLostFlag>, mut writer: MessageWriter<BoltLost>) {
        if flag.0 {
            writer.write(BoltLost);
        }
    }

    fn bolt_lost_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .insert_resource(SendBoltLostFlag(false))
            .init_resource::<CapturedLoseLifeFired>()
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_lose_life_fired)
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, (send_bolt_lost, bridge_bolt_lost).chain());
        app
    }

    // --- Bolt lost bridge tests ---

    /// Breaker + non-breaker both have `When(BoltLost)` — both should fire.
    #[test]
    fn bridge_bolt_lost_sweeps_all_entities() {
        let mut app = bolt_lost_test_app();

        // Breaker entity with BoltLost chain
        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife),
            )]),
        ));

        // Non-breaker entity with BoltLost chain
        app.world_mut().spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::BoltLost, Effect::test_shockwave(64.0)),
        )]));

        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let lose_life = app.world().resource::<CapturedLoseLifeFired>();
        let shockwave = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            lose_life.0.len() + shockwave.0.len(),
            2,
            "both breaker and non-breaker entities should fire — got lose_life={}, shockwave={}",
            lose_life.0.len(),
            shockwave.0.len()
        );
    }

    /// `ArmedEffects` on all bolt entities should fire on `BoltLost`.
    #[test]
    fn bridge_bolt_lost_evaluates_armed_on_all() {
        let mut app = bolt_lost_test_app();

        // Bolt 1 with armed BoltLost chain
        app.world_mut().spawn(ArmedEffects(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::BoltLost, Effect::test_shockwave(64.0)),
        )]));

        // Bolt 2 with armed BoltLost chain
        app.world_mut().spawn(ArmedEffects(vec![(
            None,
            EffectNode::trigger_leaf(Trigger::BoltLost, Effect::test_shockwave(32.0)),
        )]));

        app.world_mut().resource_mut::<SendBoltLostFlag>().0 = true;
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "armed effects on both bolts should fire — got {}",
            captured.0.len()
        );
    }

    /// No `BoltLost` message means no firing.
    #[test]
    fn bridge_bolt_lost_noop_without_message() {
        let mut app = bolt_lost_test_app();

        app.world_mut().spawn((
            Breaker,
            EffectChains(vec![(
                None,
                EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife),
            )]),
        ));

        // Do NOT set SendBoltLostFlag
        tick(&mut app);

        let captured = app.world().resource::<CapturedLoseLifeFired>();
        assert!(
            captured.0.is_empty(),
            "no BoltLost message — nothing should fire"
        );
    }
}
