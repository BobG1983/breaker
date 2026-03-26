//! Bridge for `NodeTimer` — fires threshold-based chains when timer ratio drops.

use bevy::prelude::*;

use crate::{
    breaker::components::Breaker,
    effect::definition::{EffectChains, EffectNode, Trigger},
    run::node::resources::NodeTimer,
};

/// Bridge for `NodeTimer` — fires `When(NodeTimerThreshold(t))` chains
/// when the timer ratio crosses below the threshold. Fires once only.
///
/// Threshold chains live on breaker entity `EffectChains`.
pub(crate) fn bridge_timer_threshold(
    timer: Res<NodeTimer>,
    mut breaker_query: Query<&mut EffectChains, With<Breaker>>,
    mut commands: Commands,
) {
    let ratio = if timer.total == 0.0 {
        0.0
    } else {
        timer.remaining / timer.total
    };

    for mut chains in &mut breaker_query {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::definition::{Effect, EffectNode, Trigger},
        run::node::resources::NodeTimer,
    };

    // --- Test infrastructure ---

    #[derive(Resource, Default)]
    struct CapturedSpeedBoostFired(Vec<crate::effect::typed_events::SpeedBoostFired>);

    fn capture_speed_boost_fired(
        trigger: On<crate::effect::typed_events::SpeedBoostFired>,
        mut captured: ResMut<CapturedSpeedBoostFired>,
    ) {
        captured.0.push(trigger.event().clone());
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Wraps a list of `EffectNode`s as `(None, node)` tuples for `EffectChains`.
    fn wrap_chains(chains: Vec<EffectNode>) -> Vec<(Option<String>, EffectNode)> {
        chains.into_iter().map(|c| (None, c)).collect()
    }

    // --- NodeTimerThreshold bridge tests ---

    #[test]
    fn bridge_timer_threshold_fires_when_ratio_crosses_below_threshold() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.25),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                multiplier: 2.0,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 14.9,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        // Place threshold chain on breaker entity EffectChains
        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![chain])),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "timer ratio 14.9/60.0 = 0.248 < 0.25 should fire"
        );
        assert!(
            (captured.0[0].multiplier - 2.0).abs() < f32::EPSILON,
            "fired effect should have multiplier 2.0"
        );
    }

    #[test]
    fn bridge_timer_threshold_no_fire_when_ratio_above_threshold() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.25),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                multiplier: 2.0,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 30.0,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![chain])),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert!(
            captured.0.is_empty(),
            "timer ratio 30/60 = 0.5 > 0.25 should NOT fire"
        );
    }

    #[test]
    fn bridge_timer_threshold_fires_once_even_if_ratio_stays_below() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.5),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                multiplier: 1.5,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 12.0,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![chain])),
        ));

        // First tick: fires
        tick(&mut app);
        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(captured.0.len(), 1, "first tick should fire");

        // Second tick: should NOT fire again (chain consumed)
        tick(&mut app);
        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "second tick should NOT fire again — chain should be consumed"
        );
    }

    #[test]
    fn bridge_timer_threshold_zero_total_treats_ratio_as_zero() {
        let chain = EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(0.5),
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                multiplier: 1.5,
            })],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeTimer {
                remaining: 10.0,
                total: 0.0, // Edge case: total is zero
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![chain])),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "total == 0.0 should treat ratio as 0.0, which is below 0.5"
        );
    }
}
