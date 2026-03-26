//! Random effect chip handler (Flux).
//!
//! Observes [`RandomEffectFired`] and selects a weighted random entry from the
//! pool using [`GameRng`]. Leaf entries fire as typed events; non-leaf entries
//! are armed on the bolt entity via [`ArmedEffects`].

use bevy::prelude::*;
use rand::Rng;

use crate::{
    effect::{
        armed::ArmedEffects,
        definition::{EffectNode, EffectTarget},
        typed_events::fire_typed_event,
    },
    shared::GameRng,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a random effect pool needs to be resolved.
#[derive(Event, Clone, Debug)]
pub(crate) struct RandomEffectFired {
    /// Weighted pool of `EffectNode` entries to select from.
    pub pool: Vec<(f32, EffectNode)>,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

// ---------------------------------------------------------------------------
// Observer — handles random effect selection
// ---------------------------------------------------------------------------

/// Observer: selects a weighted random entry from the pool and either fires
/// the leaf effect or arms the bolt with the non-leaf chain.
pub(crate) fn handle_random_effect(
    trigger: On<RandomEffectFired>,
    mut rng: ResMut<GameRng>,
    mut armed_query: Query<&mut ArmedEffects>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let pool = &event.pool;
    if pool.is_empty() {
        return;
    }

    // Weighted random selection
    let total_weight: f32 = pool.iter().map(|(w, _)| *w).sum();
    let roll: f32 = rng.0.random::<f32>() * total_weight;
    let mut cumulative = 0.0;
    let mut selected_idx = pool.len() - 1;
    for (i, (weight, _)) in pool.iter().enumerate() {
        cumulative += weight;
        if roll < cumulative {
            selected_idx = i;
            break;
        }
    }

    let (_, node) = &pool[selected_idx];

    if let EffectNode::Do(effect) = node {
        fire_typed_event(
            effect.clone(),
            event.targets.clone(),
            event.source_chip.clone(),
            &mut commands,
        );
    } else {
        // Non-leaf: arm the bolt if there is one in targets
        let bolt_entity = event.targets.iter().find_map(|t| match t {
            EffectTarget::Entity(e) => Some(*e),
            EffectTarget::Location(_) => None,
        });
        if let Some(bolt_entity) = bolt_entity {
            if let Ok(mut armed) = armed_query.get_mut(bolt_entity) {
                armed.0.push((event.source_chip.clone(), node.clone()));
            } else {
                commands.entity(bolt_entity).insert(ArmedEffects(vec![(
                    event.source_chip.clone(),
                    node.clone(),
                )]));
            }
        } else {
            warn!("RandomEffect selected non-leaf chain but no bolt entity to arm");
        }
    }
}

/// Registers all observers and systems for the random effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_random_effect);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::{
            armed::ArmedEffects,
            definition::{Effect, EffectNode, EffectTarget, Trigger},
            typed_events::{RandomEffectFired, ShockwaveFired, SpawnBoltFired},
        },
        shared::GameRng,
    };

    // --- Test infrastructure ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_random_effect);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Capture resources ---

    #[derive(Resource, Default)]
    struct CapturedSpawnBolt(Vec<SpawnBoltFired>);

    fn capture_spawn_bolt(trigger: On<SpawnBoltFired>, mut captured: ResMut<CapturedSpawnBolt>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedShockwave(Vec<ShockwaveFired>);

    fn capture_shockwave(trigger: On<ShockwaveFired>, mut captured: ResMut<CapturedShockwave>) {
        captured.0.push(trigger.event().clone());
    }

    #[derive(Resource, Default)]
    struct CapturedSpawn(Vec<SpawnBoltFired>);

    fn capture_spawn(trigger: On<SpawnBoltFired>, mut cap: ResMut<CapturedSpawn>) {
        cap.0.push(trigger.event().clone());
    }

    // =========================================================================
    // Behavior 7: handle_random_effect selects from weighted pool and fires leaf
    // =========================================================================

    #[test]
    fn handle_random_effect_fires_spawn_bolt_from_single_entry_pool() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().commands().trigger(RandomEffectFired {
            pool: vec![(
                1.0,
                EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
            )],
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: Some("Flux".to_owned()),
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            1,
            "single-entry pool with weight 1.0 should always fire SpawnBoltsFired"
        );
        assert_eq!(captured.0[0].targets, vec![EffectTarget::Entity(bolt)]);
    }

    // =========================================================================
    // Behavior 8: handle_random_effect fires ShockwaveFired for Shockwave leaf
    // =========================================================================

    #[test]
    fn handle_random_effect_fires_shockwave_when_selected() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.init_resource::<CapturedShockwave>()
            .add_observer(capture_shockwave);

        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().commands().trigger(RandomEffectFired {
            pool: vec![(
                1.0,
                EffectNode::Do(Effect::Shockwave {
                    base_range: 32.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                }),
            )],
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: Some("Flux".to_owned()),
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedShockwave>();
        assert_eq!(
            captured.0.len(),
            1,
            "Shockwave leaf should fire ShockwaveFired"
        );
        assert!(
            (captured.0[0].base_range - 32.0).abs() < f32::EPSILON,
            "base_range should be 32.0"
        );
    }

    // =========================================================================
    // Behavior 9: handle_random_effect arms bolt for non-leaf trigger wrapper
    // =========================================================================

    #[test]
    fn handle_random_effect_arms_bolt_when_non_leaf_selected() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.init_resource::<CapturedShockwave>()
            .add_observer(capture_shockwave);

        let chain = EffectNode::When {
            trigger: Trigger::Impact(crate::effect::definition::ImpactTarget::Cell),
            then: vec![EffectNode::Do(Effect::test_shockwave(32.0))],
        };
        let bolt = app.world_mut().spawn(ArmedEffects::default()).id();

        app.world_mut().commands().trigger(RandomEffectFired {
            pool: vec![(1.0, chain)],
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: Some("Flux".to_owned()),
        });
        app.world_mut().flush();

        // Non-leaf should be armed, NOT fired
        let captured = app.world().resource::<CapturedShockwave>();
        assert_eq!(
            captured.0.len(),
            0,
            "non-leaf entry should be armed, not fired as ShockwaveFired"
        );

        let armed = app
            .world()
            .get::<ArmedEffects>(bolt)
            .expect("bolt should have ArmedEffects");
        assert!(
            !armed.0.is_empty(),
            "ArmedEffects should contain the armed chain"
        );
    }

    // =========================================================================
    // Behavior 10: handle_random_effect with empty pool is a no-op
    // =========================================================================

    #[test]
    fn handle_random_effect_empty_pool_no_op() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        let bolt = app.world_mut().spawn_empty().id();

        app.world_mut().commands().trigger(RandomEffectFired {
            pool: vec![],
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: Some("Flux".to_owned()),
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(captured.0.len(), 0, "empty pool should not fire any events");
    }

    // =========================================================================
    // Behavior 11: handle_random_effect respects seed determinism
    // =========================================================================

    #[test]
    fn handle_random_effect_deterministic_with_same_seed() {
        fn run_with_seed(seed: u64) -> usize {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
                .add_observer(handle_random_effect);
            app.insert_resource(GameRng::from_seed(seed));
            app.init_resource::<CapturedSpawn>()
                .add_observer(capture_spawn);

            let bolt = app.world_mut().spawn_empty().id();

            app.world_mut().commands().trigger(RandomEffectFired {
                pool: vec![
                    (
                        0.5,
                        EffectNode::Do(Effect::SpawnBolts {
                            count: 1,
                            lifespan: None,
                            inherit: false,
                        }),
                    ),
                    (0.5, EffectNode::Do(Effect::test_speed_boost(1.1))),
                ],
                targets: vec![EffectTarget::Entity(bolt)],
                source_chip: None,
            });
            app.world_mut().flush();

            app.world().resource::<CapturedSpawn>().0.len()
        }

        let result_a = run_with_seed(99);
        let result_b = run_with_seed(99);
        assert_eq!(
            result_a, result_b,
            "same seed should produce same selection: {result_a} vs {result_b}"
        );
    }
}
