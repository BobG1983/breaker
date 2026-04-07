//! Tests for arc entity spawning with `ChainLightningArc` marker.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial};
use rantzsoft_stateflow::CleanupOnExit;

use super::*;
use crate::state::types::NodeState;

// -- Behavior 15: Arc spawned with marker and chain transitions to ArcTraveling --

#[test]
fn arc_entity_has_chain_lightning_arc_marker_and_no_extra_fields() {
    let mut app = chain_lightning_damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 40.0, 0.0);

    tick(&mut app);

    let mut hit_set = HashSet::new();
    hit_set.insert(cell_a);

    spawn_chain(
        &mut app,
        SpawnChainConfig {
            source: Vec2::new(20.0, 0.0),
            remaining_jumps: 2,
            damage: 15.0,
            range: 25.0,
            arc_speed: 200.0,
            hit_set,
            state: ChainState::Idle,
            source_chip: None,
        },
    );

    tick(&mut app);

    let mut arc_query = app.world_mut().query::<(
        &ChainLightningArc,
        &Spatial,
        &Position2D,
        &CleanupOnExit<NodeState>,
    )>();
    let arcs: Vec<_> = arc_query.iter(app.world()).collect();
    assert_eq!(arcs.len(), 1, "expected one arc entity");

    let (_, _, position, _) = arcs[0];
    assert!(
        (position.0.x - 20.0).abs() < 0.01,
        "arc Position2D should be at source position"
    );
}
