use bevy::prelude::*;
use rand::Rng;

use super::super::system::reseed_hazard_rng;
use crate::{
    prelude::*,
    shared::{
        RunSeed,
        rng::{HazardRng, derive_seed, derive_seed_named},
    },
    state::run::resources::{NodeOutcome, RunStats},
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

// ── Group D, Behavior 16 — determinism across independent apps ───────────

#[test]
fn reseed_hazard_rng_deterministic_across_independent_apps() {
    let mut app_a = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 9999,
            ..default()
        })
        .insert_resource(NodeOutcome {
            tier: 9,
            ..default()
        })
        .insert_resource(HazardRng::from_seed(SENTINEL))
        .with_system(Update, reseed_hazard_rng)
        .build();

    let mut app_b = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 9999,
            ..default()
        })
        .insert_resource(NodeOutcome {
            tier: 9,
            ..default()
        })
        .insert_resource(HazardRng::from_seed(SENTINEL))
        .with_system(Update, reseed_hazard_rng)
        .build();

    app_a.update();
    app_b.update();

    let draw_a: u64 = app_a.world_mut().resource_mut::<HazardRng>().0.random();
    let draw_b: u64 = app_b.world_mut().resource_mut::<HazardRng>().0.random();
    assert_eq!(
        draw_a, draw_b,
        "two independent apps with (seed=9999, tier=9) must produce identical draws"
    );
}

#[test]
fn reseed_hazard_rng_deterministic_zero_inputs() {
    // Edge case for Behavior 16: (seed=0, tier=0) stable across two apps.
    let mut app_a = TestAppBuilder::new()
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

    let mut app_b = TestAppBuilder::new()
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

    app_a.update();
    app_b.update();

    let draw_a: u64 = app_a.world_mut().resource_mut::<HazardRng>().0.random();
    let draw_b: u64 = app_b.world_mut().resource_mut::<HazardRng>().0.random();
    assert_eq!(
        draw_a, draw_b,
        "(seed=0, tier=0) must be stable across independent apps"
    );
}

// ── Group D, Behavior 17 — reads RunStats.seed, not RunSeed ─────────────

#[test]
fn reseed_hazard_rng_reads_run_stats_seed_not_run_seed() {
    let mut app = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 42,
            ..default()
        })
        .insert_resource(RunSeed(Some(999)))
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
        "reseed_hazard_rng must use RunStats.seed (42), not RunSeed (999)"
    );

    let wrong_seed = derive_seed(derive_seed_named(999, "hazard"), 0);
    let mut wrong_rng = HazardRng::from_seed(wrong_seed);
    let wrong_draw: u64 = wrong_rng.0.random();
    assert_ne!(
        actual_draw, wrong_draw,
        "draw must not match seed derived from RunSeed value 999"
    );
}

#[test]
fn reseed_hazard_rng_reads_run_stats_seed_when_run_seed_is_none() {
    // Edge case for Behavior 17: RunSeed(None) — still uses RunStats.seed=42.
    let mut app = TestAppBuilder::new()
        .insert_resource(RunStats {
            seed: 42,
            ..default()
        })
        .insert_resource(RunSeed(None))
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
        "RunSeed(None) must not affect reseed — RunStats.seed=42 governs"
    );
}
