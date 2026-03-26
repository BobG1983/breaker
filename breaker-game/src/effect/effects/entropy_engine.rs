//! Entropy engine evolution effect — counts cell destructions and fires
//! a random effect from the pool when the threshold is reached.
//!
//! Observes [`EntropyEngineFired`] and maintains [`EntropyEngineCounter`] resource.
//! When the counter reaches the threshold, a weighted random entry from the pool
//! is fired (for leaves) or armed (for non-leaf chains), then the counter resets.

use bevy::prelude::*;
use rand::Rng;

use crate::{
    effect::{
        definition::{EffectNode, EffectTarget},
        typed_events::fire_typed_event,
    },
    shared::GameRng,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when an entropy engine effect needs counting and potential resolution.
#[derive(Event, Clone, Debug)]
pub(crate) struct EntropyEngineFired {
    /// Number of cell destructions needed before firing.
    pub threshold: u32,
    /// Weighted pool of `EffectNode` entries to select from on trigger.
    pub pool: Vec<(f32, EffectNode)>,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the cumulative cell destruction count for the entropy engine.
#[derive(Resource, Debug, Default)]
pub(crate) struct EntropyEngineCounter {
    /// Number of cell destructions counted since last reset.
    pub count: u32,
}

// ---------------------------------------------------------------------------
// Observer — handles entropy engine counting and firing
// ---------------------------------------------------------------------------

/// Observer: increments the cell destruction counter and fires a random
/// effect from the pool when the threshold is reached.
pub(crate) fn handle_entropy_engine(
    trigger: On<EntropyEngineFired>,
    mut counter: Option<ResMut<EntropyEngineCounter>>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Get or init the counter
    let new_count = if let Some(ref mut counter) = counter {
        counter.count += 1;
        counter.count
    } else {
        // Counter doesn't exist yet — will insert via commands
        1
    };

    if new_count >= event.threshold {
        // Reset counter
        if let Some(ref mut counter) = counter {
            counter.count = 0;
        } else {
            commands.insert_resource(EntropyEngineCounter { count: 0 });
        }

        // Weighted random selection from pool
        let pool = &event.pool;
        if pool.is_empty() {
            return;
        }

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

        match node {
            EffectNode::Do(effect) => {
                fire_typed_event(
                    effect.clone(),
                    event.targets.clone(),
                    event.source_chip.clone(),
                    &mut commands,
                );
            }
            _ => {
                // TODO(C7W2): arm non-leaf chains on bolt entity
                warn!(
                    "EntropyEngine selected non-leaf chain — arming not yet supported in this handler"
                );
            }
        }
    } else if counter.is_none() {
        // Counter didn't exist, insert with new count
        commands.insert_resource(EntropyEngineCounter { count: new_count });
    }
}

/// Registers all observers and systems for the entropy engine effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_entropy_engine);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::{
            definition::{Effect, EffectNode, EffectTarget},
            typed_events::{EntropyEngineFired, SpawnBoltFired},
        },
        shared::GameRng,
    };

    // --- Test infrastructure ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_entropy_engine);
        app
    }

    // --- Capture resources ---

    #[derive(Resource, Default)]
    struct CapturedSpawnBolt(Vec<SpawnBoltFired>);

    fn capture_spawn_bolt(trigger: On<SpawnBoltFired>, mut captured: ResMut<CapturedSpawnBolt>) {
        captured.0.push(trigger.event().clone());
    }

    fn spawn_bolts_node() -> EffectNode {
        EffectNode::Do(Effect::SpawnBolts {
            count: 1,
            lifespan: None,
            inherit: false,
        })
    }

    // =========================================================================
    // Behavior 6: below threshold does not fire
    // =========================================================================

    #[test]
    fn handle_entropy_engine_below_threshold_does_not_fire() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.insert_resource(EntropyEngineCounter { count: 0 });
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        app.world_mut().commands().trigger(EntropyEngineFired {
            threshold: 5,
            pool: vec![(1.0, spawn_bolts_node())],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            0,
            "counter at 0 + 1 increment = 1, below threshold 5 — should not fire"
        );

        let counter = app.world().resource::<EntropyEngineCounter>();
        assert_eq!(counter.count, 1, "counter should be incremented to 1");
    }

    // =========================================================================
    // Behavior 7: fires random effect when counter reaches threshold
    // =========================================================================

    #[test]
    fn handle_entropy_engine_fires_at_threshold() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.insert_resource(EntropyEngineCounter { count: 4 });
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        app.world_mut().commands().trigger(EntropyEngineFired {
            threshold: 5,
            pool: vec![(1.0, spawn_bolts_node())],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            1,
            "counter at 4 + 1 = 5, equals threshold — should fire SpawnBoltsFired"
        );
    }

    // =========================================================================
    // Behavior 8: counter resets to 0 after firing
    // =========================================================================

    #[test]
    fn handle_entropy_engine_resets_counter_after_firing() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.insert_resource(EntropyEngineCounter { count: 4 });
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        app.world_mut().commands().trigger(EntropyEngineFired {
            threshold: 5,
            pool: vec![(1.0, spawn_bolts_node())],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        });
        app.world_mut().flush();

        let counter = app.world().resource::<EntropyEngineCounter>();
        assert_eq!(
            counter.count, 0,
            "counter should reset to 0 after reaching threshold"
        );
    }

    #[test]
    fn handle_entropy_engine_no_double_fire_after_reset() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.insert_resource(EntropyEngineCounter { count: 4 });
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        // First trigger — reaches threshold, fires, resets
        app.world_mut().commands().trigger(EntropyEngineFired {
            threshold: 5,
            pool: vec![(1.0, spawn_bolts_node())],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        });
        app.world_mut().flush();

        // Second trigger — counter is at 0, increments to 1, does not fire
        app.world_mut().commands().trigger(EntropyEngineFired {
            threshold: 5,
            pool: vec![(1.0, spawn_bolts_node())],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the first trigger should fire — second should increment to 1"
        );

        let counter = app.world().resource::<EntropyEngineCounter>();
        assert_eq!(
            counter.count, 1,
            "counter should be 1 after reset and one more increment"
        );
    }

    // =========================================================================
    // Behavior 9: fires every time with threshold 1
    // =========================================================================

    #[test]
    fn handle_entropy_engine_fires_every_trigger_with_threshold_one() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        app.insert_resource(EntropyEngineCounter { count: 0 });
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        app.world_mut().commands().trigger(EntropyEngineFired {
            threshold: 1,
            pool: vec![(1.0, spawn_bolts_node())],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        });
        app.world_mut().flush();

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(
            captured.0.len(),
            1,
            "threshold 1 should fire on every trigger"
        );

        let counter = app.world().resource::<EntropyEngineCounter>();
        assert_eq!(
            counter.count, 0,
            "counter should be reset to 0 after firing"
        );
    }

    // =========================================================================
    // Behavior 10: initializes counter resource if missing
    // =========================================================================

    #[test]
    fn handle_entropy_engine_initializes_counter_if_missing() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        // No EntropyEngineCounter resource inserted
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        app.world_mut().commands().trigger(EntropyEngineFired {
            threshold: 5,
            pool: vec![(1.0, spawn_bolts_node())],
            targets: vec![],
            source_chip: Some("Entropy Engine".to_owned()),
        });
        app.world_mut().flush();

        // Counter should be initialized and incremented to 1
        let counter = app
            .world()
            .get_resource::<EntropyEngineCounter>()
            .expect("EntropyEngineCounter should be initialized");
        assert_eq!(
            counter.count, 1,
            "counter should be initialized at 0, then incremented to 1"
        );

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(captured.0.len(), 0, "1 < 5, should not fire");
    }

    #[test]
    fn handle_entropy_engine_counter_persists_across_triggers() {
        let mut app = test_app();
        app.insert_resource(GameRng::from_seed(42));
        // No initial counter
        app.init_resource::<CapturedSpawnBolt>()
            .add_observer(capture_spawn_bolt);

        // Trigger 3 times (threshold 5)
        for _ in 0..3 {
            app.world_mut().commands().trigger(EntropyEngineFired {
                threshold: 5,
                pool: vec![(1.0, spawn_bolts_node())],
                targets: vec![],
                source_chip: Some("Entropy Engine".to_owned()),
            });
            app.world_mut().flush();
        }

        let counter = app.world().resource::<EntropyEngineCounter>();
        assert_eq!(
            counter.count, 3,
            "counter should persist at 3 after 3 triggers"
        );

        let captured = app.world().resource::<CapturedSpawnBolt>();
        assert_eq!(captured.0.len(), 0, "3 < 5, should not fire");
    }
}
