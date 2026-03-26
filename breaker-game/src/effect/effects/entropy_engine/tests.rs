use bevy::prelude::*;

use super::system::*;
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
// M8: Basic test with single-entry shockwave pool, threshold 1
// =========================================================================

#[derive(Resource, Default)]
struct CapturedShockwaveEE(Vec<crate::effect::typed_events::ShockwaveFired>);

fn capture_shockwave_ee(
    trigger: On<crate::effect::typed_events::ShockwaveFired>,
    mut captured: ResMut<CapturedShockwaveEE>,
) {
    captured.0.push(trigger.event().clone());
}

/// M8: `EntropyEngine` with threshold 1 and single-entry shockwave pool fires
/// ShockwaveFired(64.0) on first trigger. Counter reaches threshold immediately.
#[test]
fn handle_entropy_engine_fires_shockwave_at_threshold_one() {
    let mut app = test_app();
    app.insert_resource(GameRng::from_seed(42));
    app.insert_resource(EntropyEngineCounter { count: 0 });
    app.init_resource::<CapturedShockwaveEE>()
        .add_observer(capture_shockwave_ee);

    app.world_mut().commands().trigger(EntropyEngineFired {
        threshold: 1,
        pool: vec![(1.0, EffectNode::Do(Effect::test_shockwave(64.0)))],
        targets: vec![],
        source_chip: Some("Entropy Engine".to_owned()),
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedShockwaveEE>();
    assert_eq!(
        captured.0.len(),
        1,
        "threshold 1 with count 0: counter increments to 1, reaches threshold — should fire"
    );
    assert!(
        (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
        "ShockwaveFired base_range should be 64.0"
    );

    let counter = app.world().resource::<EntropyEngineCounter>();
    assert_eq!(counter.count, 0, "counter should reset to 0 after firing");
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
