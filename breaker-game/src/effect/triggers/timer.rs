//! Bridge for `NodeTimer` — sweeps ALL entities with `EffectChains` for
//! `Trigger::NodeTimerThreshold(t)` when the timer ratio crosses below the threshold.

use bevy::prelude::*;

use crate::{
    effect::definition::{EffectChains, EffectNode, Trigger},
    run::node::resources::NodeTimer,
};

/// Bridge for `NodeTimer` — fires `When(NodeTimerThreshold(t))` chains when
/// the timer ratio crosses below the threshold. Fires once only (chain consumed).
///
/// FIX from old bridge: was breaker-only; now sweeps ALL entities.
pub(crate) fn bridge_timer_threshold(
    timer: Res<NodeTimer>,
    mut chains_query: Query<&mut EffectChains>,
    mut commands: Commands,
) {
    let ratio = if timer.total == 0.0 {
        0.0
    } else {
        timer.remaining / timer.total
    };

    for mut chains in &mut chains_query {
        let mut indices_to_remove = Vec::new();
        for (i, (_chip_name, chain)) in chains.0.iter().enumerate() {
            if let EffectNode::When {
                trigger: Trigger::NodeTimerThreshold(threshold),
                then,
            } = chain
                && ratio < *threshold
            {
                // Fire children
                for child in then {
                    if let EffectNode::Do(effect) = child {
                        crate::effect::typed_events::fire_typed_event(
                            effect.clone(),
                            vec![],
                            None,
                            &mut commands,
                        );
                    }
                }
                indices_to_remove.push(i);
            }
        }

        // Remove fired chains in reverse order to preserve indices
        for &i in indices_to_remove.iter().rev() {
            chains.0.remove(i);
        }
    }
}

/// Registers bridge systems for timer threshold trigger.
pub(crate) fn register(app: &mut App) {
    use crate::{effect::sets::EffectSystems, shared::PlayingState};
    app.add_systems(
        FixedUpdate,
        bridge_timer_threshold
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
            definition::{Effect, EffectNode, Trigger},
            typed_events::*,
        },
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedSpeedBoostFired(Vec<SpeedBoostFired>);

    fn capture_speed_boost_fired(
        trigger: On<SpeedBoostFired>,
        mut captured: ResMut<CapturedSpeedBoostFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    // --- Timer threshold bridge tests ---

    /// Breaker + non-breaker both have `When(NodeTimerThreshold(0.5))` — both fire
    /// when ratio < 0.5.
    #[test]
    fn bridge_timer_sweeps_all_entities() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.5),
            then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 12.0,
                total: 60.0, // ratio = 0.2, below 0.5
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        // Breaker entity with threshold chain
        app.world_mut()
            .spawn((Breaker, EffectChains(wrap_chains(vec![chain.clone()]))));

        // Non-breaker entity with threshold chain
        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![chain])));

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            2,
            "both breaker and non-breaker should fire when ratio < threshold — got {}",
            captured.0.len()
        );
    }

    /// Ratio above threshold should not fire.
    #[test]
    fn bridge_timer_no_fire_above_threshold() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.25),
            then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 30.0,
                total: 60.0, // ratio = 0.5, above 0.25
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![chain])));

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert!(
            captured.0.is_empty(),
            "ratio 0.5 > 0.25 should NOT fire — got {}",
            captured.0.len()
        );
    }

    /// M4: Ratio exactly at threshold (0.5) does NOT fire — strict less-than comparison.
    #[test]
    fn bridge_timer_no_fire_at_exact_threshold() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.5),
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 30.0,
                total: 60.0, // ratio = 0.5, exactly at threshold
            })
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![chain])));

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "ratio 0.5 is NOT less than threshold 0.5 — should NOT fire (strict less-than). Got {}",
            captured.0.len()
        );
    }

    /// After firing, the chain is consumed. Second tick should not fire again.
    #[test]
    fn bridge_timer_fires_once_only() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.5),
            then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 12.0,
                total: 60.0, // ratio = 0.2, below 0.5
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        app.world_mut()
            .spawn(EffectChains(wrap_chains(vec![chain])));

        // First tick: should fire
        tick(&mut app);
        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(captured.0.len(), 1, "first tick should fire");

        // Second tick: should NOT fire again (chain consumed)
        tick(&mut app);
        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "second tick should NOT fire again — chain should be consumed — got {}",
            captured.0.len()
        );
    }
}
