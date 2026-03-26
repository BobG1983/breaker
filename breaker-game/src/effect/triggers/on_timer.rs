//! Bridge for `NodeTimer` — fires threshold-based chains when timer ratio drops.

use bevy::prelude::*;

use crate::{
    effect::{
        active::ActiveEffects,
        definition::{EffectNode, Trigger},
        typed_events::fire_typed_event,
    },
    run::node::resources::NodeTimer,
};

/// Bridge for `NodeTimer` — fires `When(NodeTimerThreshold(t))` chains
/// when the timer ratio crosses below the threshold. Fires once only.
pub(crate) fn bridge_timer_threshold(
    timer: Res<NodeTimer>,
    mut active: ResMut<ActiveEffects>,
    mut commands: Commands,
) {
    // Early return if no threshold chains exist
    let has_threshold = active.0.iter().any(|(_, chain)| {
        matches!(
            chain,
            EffectNode::When {
                trigger: Trigger::NodeTimerThreshold(_),
                ..
            }
        )
    });
    if !has_threshold {
        return;
    }

    let ratio = if timer.total == 0.0 {
        0.0
    } else {
        timer.remaining / timer.total
    };

    // Find and fire matching threshold chains, then remove them (fire-once).
    let mut indices_to_remove = Vec::new();
    for (i, (_chip_name, chain)) in active.0.iter().enumerate() {
        if let EffectNode::When {
            trigger: Trigger::NodeTimerThreshold(threshold),
            then,
        } = chain
            && ratio < *threshold
        {
            // Fire children
            for child in then {
                if let EffectNode::Do(effect) = child {
                    fire_typed_event(effect.clone(), vec![], None, &mut commands);
                }
                // Non-leaf children from timer threshold — skip for now
            }
            indices_to_remove.push(i);
        }
    }

    // Remove fired chains in reverse order to preserve indices
    for &i in indices_to_remove.iter().rev() {
        active.0.remove(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::{
            active::ActiveEffects,
            definition::{Effect, EffectNode, Target, Trigger},
            typed_events::*,
        },
        run::node::resources::NodeTimer,
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

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Wraps a list of `EffectNode`s as `(None, node)` tuples for `ActiveEffects`.
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
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 14.9,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

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
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 30.0,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

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
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 12.0,
                total: 60.0,
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

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
            .insert_resource(ActiveEffects(wrap_chains(vec![chain])))
            .insert_resource(NodeTimer {
                remaining: 10.0,
                total: 0.0, // Edge case: total is zero
            })
            .init_resource::<CapturedSpeedBoostFired>()
            .add_observer(capture_speed_boost_fired)
            .add_systems(FixedUpdate, bridge_timer_threshold);

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpeedBoostFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "total == 0.0 should treat ratio as 0.0, which is below 0.5"
        );
    }
}
