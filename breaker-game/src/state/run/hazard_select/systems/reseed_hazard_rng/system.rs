//! Reseed `HazardRng` on each hazard-select visit using the canonical
//! `derive_seed(derive_seed_named(run_seed, "hazard"), tier_index)` formula.

use bevy::prelude::*;

use crate::{
    shared::rng::{HazardRng, derive_seed, derive_seed_named},
    state::run::resources::{NodeOutcome, RunStats},
};

/// Seeds `HazardRng` from the canonical formula.
///
/// Schedule: registered in `OnEnter(HazardSelectState::Selecting)` under
/// `HazardSelectSystems::Reseed`, ordered before `HazardSelectSystems::GenerateOfferings`.
/// The discriminator is `NodeOutcome.tier` — no separate counter.
pub(crate) fn reseed_hazard_rng(
    run_stats: Res<RunStats>,
    mut rng: ResMut<HazardRng>,
    node_outcome: Res<NodeOutcome>,
) {
    let seed = derive_seed(
        derive_seed_named(run_stats.seed, "hazard"),
        u64::from(node_outcome.tier),
    );
    *rng = HazardRng::from_seed(seed);
}
