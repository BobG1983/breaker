use bevy::prelude::*;
use rand::Rng;

use super::super::system::reseed_hazard_rng;
use crate::{
    prelude::*,
    shared::rng::{HazardRng, derive_seed, derive_seed_named},
    state::run::resources::{NodeOutcome, RunStats},
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

// ── Group D, Behavior 15 — overwrites pre-existing HazardRng ────────────

#[test]
fn reseed_hazard_rng_overwrites_preexisting_rng_resource() {
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

    app.update();

    let expected_seed = derive_seed(derive_seed_named(42, "hazard"), 3);
    let mut expected_rng = HazardRng::from_seed(expected_seed);
    let expected_draw: u64 = expected_rng.0.random();
    let actual_draw: u64 = app.world_mut().resource_mut::<HazardRng>().0.random();
    assert_eq!(
        actual_draw, expected_draw,
        "reseed must overwrite SENTINEL-seeded HazardRng with formula result for tier=3"
    );

    let sentinel_draw: u64 = HazardRng::from_seed(SENTINEL).0.random();
    assert_ne!(
        actual_draw, sentinel_draw,
        "actual draw must not equal SENTINEL stream"
    );
}

#[test]
fn reseed_hazard_rng_idempotent_under_same_inputs() {
    // Edge case for Behavior 15: same (seed, tier) produces identical draws.
    let run_once = |seed: u64, tier: u32| -> u64 {
        let mut app = TestAppBuilder::new()
            .insert_resource(RunStats { seed, ..default() })
            .insert_resource(NodeOutcome { tier, ..default() })
            .insert_resource(HazardRng::from_seed(SENTINEL))
            .with_system(Update, reseed_hazard_rng)
            .build();
        app.update();
        app.world_mut().resource_mut::<HazardRng>().0.random()
    };

    assert_eq!(
        run_once(42, 3),
        run_once(42, 3),
        "same (seed=42, tier=3) must produce identical draws across independent runs"
    );
}
