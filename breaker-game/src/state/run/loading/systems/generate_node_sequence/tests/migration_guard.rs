//! Structural migration guards for Wave 2E.
//!
//! Behaviors 11 (negative), 15, 17, 18: checks that the old monolithic RNG
//! resource behaves correctly in the two harnesses and that the source files
//! no longer contain its identifier after migration.
//! Extracted here so that the identifier does not appear in the asserting file.

use bevy::prelude::*;
use rantzsoft_stateflow::RoutingTable;

use super::{
    super::system::generate_node_sequence_system,
    helpers::{make_curve, make_tier},
};
use crate::{
    prelude::*,
    shared::rng::NodeSequenceRng,
    state::{
        run::{RunPlugin, resources::NodeSequence},
        types::*,
    },
};

// The forbidden identifier is built at compile time to avoid a self-referential
// false-positive where include_str finds the guard string in this file.
const MONOLITHIC_RNG: &str = concat!("Game", "Rng");

fn schedule_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<RunStats>()
        .build();
    app.init_resource::<RoutingTable<NodeState>>();
    app.add_plugins(RunPlugin);
    app
}

// Behavior 11 (negative side): old monolithic RNG is absent from the MinimalPlugins
// bare-app harness — the system no longer demands it after migration.
#[test]
fn bare_app_harness_does_not_contain_old_monolithic_rng() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(3), 0.5, 1.0)], 0.0);
    app.insert_resource(curve);
    app.insert_resource(NodeSequenceRng::from_seed(42));
    app.insert_resource(RunStats::default());
    app.add_systems(Update, generate_node_sequence_system);
    app.update();

    assert!(
        !app.world().contains_resource::<GameRng>(),
        "the MinimalPlugins bare-app harness must not contain the old monolithic RNG; \
         the system is migrated to NodeSequenceRng"
    );
    assert!(
        app.world().get_resource::<NodeSequence>().is_some(),
        "system must insert NodeSequence even without the old monolithic RNG present"
    );
}

// Behavior 15: old monolithic RNG remains registered in RunPlugin (negative guard against
// over-aggressive migration — other consumers not yet migrated still depend on it).
#[test]
fn run_plugin_still_registers_old_monolithic_rng() {
    let app = schedule_app();
    assert!(
        app.world().contains_resource::<GameRng>(),
        "the old monolithic RNG resource must remain registered in RunPlugin; \
         capture_run_seed and other un-migrated consumers still depend on it"
    );
}

// Behavior 17: generate_node_sequence/system.rs contains no old monolithic RNG reference.
#[test]
fn system_rs_does_not_reference_old_monolithic_rng_after_migration() {
    let source = include_str!("../system.rs");
    assert!(
        !source.contains(MONOLITHIC_RNG),
        "generate_node_sequence/system.rs must not reference the old monolithic RNG after Wave 2E; \
         use NodeSequenceRng instead"
    );
    assert!(
        source.contains("NodeSequenceRng"),
        "generate_node_sequence/system.rs must reference NodeSequenceRng after migration"
    );
}

// Behavior 18: ecs_wrapper.rs (the harness) contains no old monolithic RNG reference.
#[test]
fn ecs_wrapper_rs_does_not_reference_old_monolithic_rng_after_migration() {
    let source = include_str!("ecs_wrapper.rs");
    assert!(
        !source.contains(MONOLITHIC_RNG),
        "generate_node_sequence/tests/ecs_wrapper.rs must not reference the old monolithic RNG \
         after Wave 2E harness migration; use NodeSequenceRng instead"
    );
    assert!(
        source.contains("NodeSequenceRng"),
        "generate_node_sequence/tests/ecs_wrapper.rs must reference NodeSequenceRng after migration"
    );
}
