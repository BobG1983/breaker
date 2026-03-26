//! `apply_once_nodes` — processes `Once` node wrappers at chip selection time.
//!
//! Moved from `on_death.rs` to its own file. Fires bare `Do` children inside
//! `Once` wrappers and removes the consumed `Once` from `EffectChains`.

use bevy::prelude::*;

use crate::effect::{
    definition::{EffectChains, EffectNode},
    typed_events::fire_typed_event,
};

/// Processes `Once` nodes wrapping bare `Do` children at chip selection time.
/// Fires the effect and removes the `Once` wrapper from `EffectChains`.
/// Once nodes wrapping `When` nodes are left for bridge evaluation.
pub(crate) fn apply_once_nodes(mut query: Query<&mut EffectChains>, mut commands: Commands) {
    for mut chains in &mut query {
        chains.0.retain(|(chip_name, node)| {
            if let EffectNode::Once(children) = node {
                // Check if all children are bare Do nodes
                let all_bare_do = children.iter().all(|c| matches!(c, EffectNode::Do(_)));
                if all_bare_do && !children.is_empty() {
                    // Fire all bare Do children
                    for child in children {
                        if let EffectNode::Do(effect) = child {
                            fire_typed_event(
                                effect.clone(),
                                vec![],
                                chip_name.clone(),
                                &mut commands,
                            );
                        }
                    }
                    return false; // Remove the Once node
                }
            }
            true // Keep non-Once nodes and Once nodes wrapping When
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_helpers::tick, *};
    use crate::effect::{
        definition::{Effect, EffectNode, Trigger},
        typed_events::*,
    };

    // --- Once node evaluation tests ---

    /// Bare `Do` inside `Once` fires at chip selection time and is consumed.
    #[test]
    fn apply_once_nodes_fires_bare_do_and_removes_once_wrapper() {
        #[derive(Resource, Default)]
        struct CapturedSpawnBoltsFired(Vec<SpawnBoltsFired>);

        fn capture_spawn_bolts(
            trigger: On<SpawnBoltsFired>,
            mut captured: ResMut<CapturedSpawnBoltsFired>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedSpawnBoltsFired>()
            .add_observer(capture_spawn_bolts)
            .add_systems(FixedUpdate, apply_once_nodes);

        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::Once(vec![EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                })]),
            )]))
            .id();

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBoltsFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "bare Do inside Once should fire at chip selection time"
        );

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert!(
            chains.0.is_empty(),
            "Once node should be removed from EffectChains after firing"
        );
    }

    /// Empty `EffectChains` (Once already consumed) should not fire again.
    #[test]
    fn once_already_consumed_does_not_fire_again() {
        #[derive(Resource, Default)]
        struct CapturedSpawnBoltsFired2(Vec<SpawnBoltsFired>);

        fn capture_spawn_bolts2(
            trigger: On<SpawnBoltsFired>,
            mut captured: ResMut<CapturedSpawnBoltsFired2>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedSpawnBoltsFired2>()
            .add_observer(capture_spawn_bolts2)
            .add_systems(FixedUpdate, apply_once_nodes);

        // Empty EffectChains — Once was already consumed
        app.world_mut().spawn(EffectChains::default());

        tick(&mut app);

        let captured = app.world().resource::<CapturedSpawnBoltsFired2>();
        assert!(
            captured.0.is_empty(),
            "empty EffectChains should not fire anything"
        );
    }

    /// M5: Multiple Once nodes — consuming bare Do Once does not affect
    /// Once wrapping When. Bare Do fires and is removed; Once(When) is preserved.
    #[test]
    fn apply_once_nodes_consumes_bare_do_preserves_once_when() {
        #[derive(Resource, Default)]
        struct CapturedShockwaveMulti(Vec<ShockwaveFired>);

        fn capture_shockwave_multi(
            trigger: On<ShockwaveFired>,
            mut captured: ResMut<CapturedShockwaveMulti>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedShockwaveMulti>()
            .add_observer(capture_shockwave_multi)
            .add_systems(FixedUpdate, apply_once_nodes);

        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![
                // First entry: Once wrapping bare Do(Shockwave(64.0)) — should fire and be consumed
                (
                    None,
                    EffectNode::Once(vec![EffectNode::Do(Effect::test_shockwave(64.0))]),
                ),
                // Second entry: Once wrapping When(BoltLost, [Do(Shockwave(32.0))]) — should be preserved
                (
                    None,
                    EffectNode::Once(vec![EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(Effect::test_shockwave(32.0))],
                    }]),
                ),
            ]))
            .id();

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveMulti>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the bare Do Once should fire — got {}",
            captured.0.len()
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "the fired shockwave should be the 64.0 one from bare Do"
        );

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert_eq!(
            chains.0.len(),
            1,
            "EffectChains should have 1 remaining entry (Once wrapping When preserved)"
        );
        assert!(
            matches!(&chains.0[0].1, EffectNode::Once(children) if children.len() == 1),
            "remaining entry should be Once wrapping When"
        );
    }

    /// `Once` wrapping `When` nodes should NOT fire at chip selection time.
    #[test]
    fn apply_once_nodes_preserves_once_wrapping_when() {
        #[derive(Resource, Default)]
        struct CapturedShockwaveFired(Vec<ShockwaveFired>);

        fn capture_shockwave(
            trigger: On<ShockwaveFired>,
            mut captured: ResMut<CapturedShockwaveFired>,
        ) {
            captured.0.push(trigger.event().clone());
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave)
            .add_systems(FixedUpdate, apply_once_nodes);

        let entity = app
            .world_mut()
            .spawn(EffectChains(vec![(
                None,
                EffectNode::Once(vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }]),
            )]))
            .id();

        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "Once wrapping When should NOT fire at chip selection time"
        );

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert_eq!(
            chains.0.len(),
            1,
            "Once wrapping When should be preserved for bridge evaluation"
        );
    }
}
