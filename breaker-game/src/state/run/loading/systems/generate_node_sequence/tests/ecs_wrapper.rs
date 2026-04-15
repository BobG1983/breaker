//! Tests for the ECS wrapper: `generate_node_sequence_system` inserts
//! `NodeSequence` resource and produces deterministic results via `GameRng`.

use bevy::prelude::*;

use super::{super::system::generate_node_sequence_system, helpers::*};
use crate::{
    prelude::*,
    state::run::resources::{NodeAssignment, NodeSequence},
};

// -- 16. ECS wrapper: generate_node_sequence_system --

#[test]
fn system_inserts_node_sequence_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(3), 0.5, 1.0)], 0.0);
    app.insert_resource(curve);
    app.insert_resource(GameRng::from_seed(42));
    app.add_systems(Update, generate_node_sequence_system);
    app.update();

    let seq = app
        .world()
        .get_resource::<NodeSequence>()
        .expect("system should insert NodeSequence resource");
    assert_eq!(
        seq.assignments.len(),
        4,
        "Fixed(3) + boss = 4 total assignments"
    );
}

#[test]
fn system_generates_deterministic_sequence_from_game_rng() {
    // Run the system twice with the same seed and curve, verify identical results.
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0),
            make_tier(TierNodeCount::Range(3, 5), 0.4, 0.9),
        ],
        0.1,
    );

    let run_system = |seed: u64| -> Vec<NodeAssignment> {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(curve.clone());
        app.insert_resource(GameRng::from_seed(seed));
        app.add_systems(Update, generate_node_sequence_system);
        app.update();
        app.world().resource::<NodeSequence>().assignments.clone()
    };

    let seq1 = run_system(42);
    let seq2 = run_system(42);
    assert_eq!(
        seq1, seq2,
        "same seed should produce identical node sequence via system"
    );

    // Verify it's non-empty
    assert!(
        !seq1.is_empty(),
        "system should produce non-empty sequence for non-empty curve"
    );
}
