use bevy::prelude::*;

use super::{super::system::reseed_hazard_rng, helpers::make_full_hazard_registry};
use crate::{
    prelude::*,
    shared::rng::{HazardRng, derive_seed, derive_seed_named},
    state::run::resources::{NodeOutcome, RunStats},
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

// ── Group G, Behavior 35 — end-to-end determinism ───────────────────────

#[test]
fn end_to_end_hazard_offering_is_deterministic_for_seed_42_tier_5() {
    use crate::{
        mutators::hazards::{
            definition::{HazardDefinition, HazardKind},
            resources::{ActiveHazards, HazardOffers},
        },
        state::run::hazard_select::systems::generate_hazard_offerings,
    };

    let build = || -> App {
        TestAppBuilder::new()
            .insert_resource(RunStats {
                seed: 42,
                ..default()
            })
            .insert_resource(NodeOutcome {
                tier: 5,
                ..default()
            })
            .insert_resource(HazardRng::from_seed(SENTINEL))
            .insert_resource(ActiveHazards::default())
            .insert_resource(HazardOffers::default())
            .insert_resource(make_full_hazard_registry())
            .with_system(
                Update,
                (reseed_hazard_rng, generate_hazard_offerings).chain(),
            )
            .build()
    };

    let mut app_a = build();
    let mut app_b = build();
    app_a.update();
    app_b.update();

    let offers_a: Vec<HazardKind> = app_a
        .world()
        .resource::<HazardOffers>()
        .0
        .iter()
        .map(HazardDefinition::kind)
        .collect();
    let offers_b: Vec<HazardKind> = app_b
        .world()
        .resource::<HazardOffers>()
        .0
        .iter()
        .map(HazardDefinition::kind)
        .collect();

    assert_eq!(
        offers_a, offers_b,
        "end-to-end: (seed=42, tier=5) must produce the same HazardOffers in independent apps"
    );
}

#[test]
fn end_to_end_different_tier_produces_different_channel_seeds() {
    // Edge case A: tier=0 vs tier=5 must have distinct channel seeds (prerequisite
    // for different outputs). The full pipeline test would need an app; seed
    // distinctness is sufficient to prove the formula routes to different streams.
    let seed_0 = derive_seed(derive_seed_named(42, "hazard"), 0);
    let seed_5 = derive_seed(derive_seed_named(42, "hazard"), 5);
    assert_ne!(
        seed_0, seed_5,
        "tier=0 and tier=5 must produce distinct hazard channel seeds (prerequisite)"
    );
}

#[test]
fn end_to_end_different_run_seed_produces_different_channel_seeds() {
    // Edge case B: seed=42 vs seed=100 (tier=5) — channel seeds must differ.
    let seed_42 = derive_seed(derive_seed_named(42, "hazard"), 5);
    let seed_100 = derive_seed(derive_seed_named(100, "hazard"), 5);
    assert_ne!(
        seed_42, seed_100,
        "run_seed=42 and run_seed=100 must produce distinct hazard channel seeds for tier=5"
    );
}
