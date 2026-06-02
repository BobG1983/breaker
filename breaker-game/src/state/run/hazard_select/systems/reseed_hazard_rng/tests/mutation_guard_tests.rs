use bevy::prelude::*;

use super::super::system::reseed_hazard_rng;
use crate::{
    prelude::*,
    shared::rng::HazardRng,
    state::run::resources::{NodeOutcome, NodeResult, RunStats},
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

// ── Group D, Behavior 18 — does NOT mutate NodeOutcome ──────────────────

#[test]
fn reseed_hazard_rng_does_not_mutate_node_outcome() {
    let initial = NodeOutcome {
        tier:               3,
        node_index:         12,
        result:             NodeResult::Won,
        cleared_this_frame: true,
        position_in_tier:   4,
    };

    let mut app = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 42,
            ..default()
        })
        .insert_resource(initial)
        .insert_resource(HazardRng::from_seed(SENTINEL))
        .with_system(Update, reseed_hazard_rng)
        .build();

    app.update();

    let outcome = app.world().resource::<NodeOutcome>();
    assert_eq!(outcome.tier, 3, "NodeOutcome.tier must be unchanged");
    assert_eq!(
        outcome.node_index, 12,
        "NodeOutcome.node_index must be unchanged"
    );
    assert_eq!(
        outcome.result,
        NodeResult::Won,
        "NodeOutcome.result must be unchanged"
    );
    assert!(
        outcome.cleared_this_frame,
        "NodeOutcome.cleared_this_frame must be unchanged"
    );
    assert_eq!(
        outcome.position_in_tier, 4,
        "NodeOutcome.position_in_tier must be unchanged"
    );
}

#[test]
fn reseed_hazard_rng_does_not_mutate_node_outcome_across_five_ticks() {
    // Edge case for Behavior 18: five ticks, tier stays at 3.
    let mut app = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 42,
            ..default()
        })
        .insert_resource(NodeOutcome {
            tier: 3,
            ..default()
        })
        .insert_resource(HazardRng::from_seed(SENTINEL))
        .with_system(Update, reseed_hazard_rng)
        .build();

    for _ in 0..5 {
        app.update();
    }

    assert_eq!(
        app.world().resource::<NodeOutcome>().tier,
        3,
        "NodeOutcome.tier must remain 3 after 5 ticks"
    );
}
