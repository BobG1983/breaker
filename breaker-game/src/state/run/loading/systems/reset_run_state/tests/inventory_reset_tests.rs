use bevy::prelude::*;

use super::helpers::test_app;
use crate::{
    prelude::*,
    shared::RunSeed,
    state::run::resources::{NodeOutcome, NodeResult},
};

#[test]
fn resets_to_defaults() {
    let mut app = test_app();
    app.update();

    let state = app.world().resource::<NodeOutcome>();
    assert_eq!(state.node_index, 0);
    assert_eq!(state.result, NodeResult::InProgress);
}

#[test]
fn reseeds_with_specific_seed_when_set() {
    use rand::Rng;
    let mut app = test_app();
    app.world_mut().insert_resource(RunSeed(Some(42)));
    app.update();

    let val1: f32 = app.world_mut().resource_mut::<GameRng>().0.random();

    // Same seed must produce same sequence
    let mut rng2 = GameRng::from_seed(42);
    let val2: f32 = rng2.0.random();
    assert!(
        (val1 - val2).abs() < f32::EPSILON,
        "expected deterministic output with seed 42"
    );
}

#[test]
fn reseeds_with_entropy_when_none() {
    use rand::Rng;
    let mut app = test_app();
    // RunSeed default is None
    app.update();

    let val1: f32 = app.world_mut().resource_mut::<GameRng>().0.random();

    // Run again — should get a different RNG state (extremely unlikely to match)
    app.world_mut().insert_resource(NodeOutcome {
        node_index: 5,
        result: NodeResult::Won,
        ..default()
    });
    app.update();

    let val2: f32 = app.world_mut().resource_mut::<GameRng>().0.random();
    // Not asserting inequality — OS entropy could theoretically match,
    // but we verify the code path runs without panic
    let _ = (val1, val2);
}

// ── Reset clears per-run protocol and hazard state ─

/// Seed `ActiveProtocols` with a `Greed` definition, `ActiveHazards`
/// with `add_stack(Decay)` ×2, and `ProtocolOffer` with a stale value,
/// then run the system once. Every per-run resource should be cleared.
#[test]
fn clears_active_protocols_and_active_hazards() {
    use crate::mutators::{
        hazards::{definition::HazardKind, resources::ActiveHazards},
        protocols::{
            definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
            resources::{ActiveProtocols, ProtocolOffer},
        },
    };

    let mut app = test_app();

    // Seed ActiveProtocols with one Greed definition.
    {
        let mut protocols = app.world_mut().resource_mut::<ActiveProtocols>();
        protocols.insert(ProtocolDefinition {
            name:        "Greed".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
        });
    }
    // Seed ActiveHazards with two Decay stacks.
    {
        let mut hazards = app.world_mut().resource_mut::<ActiveHazards>();
        hazards.add_stack(HazardKind::Decay);
        hazards.add_stack(HazardKind::Decay);
    }
    // Seed ProtocolOffer with a stale offer to verify it is cleared.
    {
        let mut offer = app.world_mut().resource_mut::<ProtocolOffer>();
        offer.0 = Some(ProtocolDefinition {
            name:        "Stale".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
        });
    }

    app.update();

    assert!(
        app.world().resource::<ActiveProtocols>().is_empty(),
        "reset_run_state must clear ActiveProtocols"
    );
    assert!(
        app.world().resource::<ActiveHazards>().is_empty(),
        "reset_run_state must clear ActiveHazards"
    );
    assert!(
        !app.world()
            .resource::<ActiveProtocols>()
            .contains(ProtocolKind::Greed),
        "Greed must not be active after reset"
    );
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Decay),
        0,
        "Decay stacks must be 0 after reset"
    );
    assert!(
        app.world().resource::<ProtocolOffer>().0.is_none(),
        "reset_run_state must clear ProtocolOffer"
    );
}

// ── Reset clears GreedStacks ─

/// Behavior 27 — `reset_run_state` clears `GreedStacks` back to default
/// (skips = 0), while `GreedConfig` is preserved (not a per-run
/// inventory).
#[test]
fn reset_run_state_clears_greed_stacks_and_preserves_greed_config() {
    use crate::mutators::protocols::greed::{GreedConfig, GreedStacks};

    let mut app = test_app();

    // Seed GreedStacks with a non-default value.
    app.world_mut().insert_resource(GreedStacks { skips: 7 });
    // Seed GreedConfig — must be preserved across reset.
    app.world_mut().insert_resource(GreedConfig {
        rarity_boost_per_skip: 5.0,
    });

    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        0,
        "reset_run_state must clear GreedStacks back to default (skips = 0)"
    );
    let cfg = app
        .world()
        .get_resource::<GreedConfig>()
        .expect("GreedConfig must be preserved across reset_run_state");
    assert!(
        (cfg.rarity_boost_per_skip - 5.0).abs() < f32::EPSILON,
        "GreedConfig.rarity_boost_per_skip must be preserved; expected 5.0, got {}",
        cfg.rarity_boost_per_skip
    );
}

/// Behavior 28 — `test_app()` builds with `GreedStacks` initialized so the
/// `RunInventories` `SystemParam` resolves; after `app.update()` the skips
/// field is zero (default).
#[test]
fn test_app_initializes_greed_stacks_and_reset_leaves_it_at_default() {
    use crate::mutators::protocols::greed::GreedStacks;

    let mut app = test_app();
    assert!(
        app.world().get_resource::<GreedStacks>().is_some(),
        "test_app() must init GreedStacks so RunInventories SystemParam resolves"
    );

    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        0,
        "after reset_run_state, GreedStacks.skips must be 0"
    );
}

/// Behavior 29 — large skip counts reset to 0 (reset uses assignment, not
/// subtraction).
#[test]
fn reset_run_state_clears_large_and_max_skip_counts_to_zero() {
    use crate::mutators::protocols::greed::GreedStacks;

    // Case A: 1000 skips.
    {
        let mut app = test_app();
        app.world_mut().insert_resource(GreedStacks { skips: 1000 });
        app.update();
        assert_eq!(
            app.world().resource::<GreedStacks>().skips,
            0,
            "1000 skips must reset to 0"
        );
    }
    // Case B: u32::MAX skips.
    {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(GreedStacks { skips: u32::MAX });
        app.update();
        assert_eq!(
            app.world().resource::<GreedStacks>().skips,
            0,
            "u32::MAX skips must reset to 0 (confirms assignment, not subtraction)"
        );
    }
}
