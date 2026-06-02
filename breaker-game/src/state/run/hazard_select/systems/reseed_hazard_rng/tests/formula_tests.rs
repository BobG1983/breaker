use bevy::prelude::*;
use rand::Rng;

use super::super::system::reseed_hazard_rng;
use crate::{
    prelude::*,
    shared::rng::{HazardRng, derive_seed, derive_seed_named},
    state::run::resources::{NodeOutcome, RunStats},
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

// ── Group D, Behavior 13 — formula for tier_index = 0 ───────────────────

#[test]
fn reseed_hazard_rng_uses_canonical_formula_for_tier_zero() {
    let mut app = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 42,
            ..default()
        })
        .insert_resource(NodeOutcome {
            tier: 0,
            ..default()
        })
        .insert_resource(HazardRng::from_seed(SENTINEL))
        .with_system(Update, reseed_hazard_rng)
        .build();

    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "hazard"), 0);
    let mut expected_rng = HazardRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();
    let actual_draw: u64 = app.world_mut().resource_mut::<HazardRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "HazardRng first draw must match derive_seed(derive_seed_named(42, \"hazard\"), 0)"
    );
}

#[test]
fn reseed_hazard_rng_formula_with_zero_seed_no_panic() {
    // Edge case for Behavior 13: seed=0, tier=0 must not panic.
    let mut app = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 0,
            ..default()
        })
        .insert_resource(NodeOutcome {
            tier: 0,
            ..default()
        })
        .insert_resource(HazardRng::from_seed(SENTINEL))
        .with_system(Update, reseed_hazard_rng)
        .build();

    app.update();

    let expected_seed = derive_seed(derive_seed_named(0, "hazard"), 0);
    let mut expected_rng = HazardRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();
    let actual_draw: u64 = app.world_mut().resource_mut::<HazardRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "seed=0 must produce derive_seed(derive_seed_named(0, \"hazard\"), 0)"
    );
}

// ── Group D, Behavior 14 — uses NodeOutcome.tier as discriminator ────────

#[test]
fn reseed_hazard_rng_uses_tier_5_as_discriminator() {
    let mut app = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 42,
            ..default()
        })
        .insert_resource(NodeOutcome {
            tier: 5,
            ..default()
        })
        .insert_resource(HazardRng::from_seed(SENTINEL))
        .with_system(Update, reseed_hazard_rng)
        .build();

    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "hazard"), 5);
    let mut expected_rng = HazardRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();
    let actual_draw: u64 = app.world_mut().resource_mut::<HazardRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "HazardRng with tier=5 must use discriminator 5"
    );
}

#[test]
fn different_tiers_produce_different_streams() {
    // Edge case for Behavior 14: tier=0 vs tier=5 must differ.
    let draw_with_tier = |tier: u32| -> u64 {
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(NodeOutcome { tier, ..default() })
            .insert_resource(HazardRng::from_seed(SENTINEL))
            .with_system(Update, reseed_hazard_rng)
            .build();
        app.update();
        app.world_mut().resource_mut::<HazardRng>().0.random()
    };

    assert_ne!(
        draw_with_tier(0),
        draw_with_tier(5),
        "tier=0 and tier=5 must produce different HazardRng streams"
    );
}
