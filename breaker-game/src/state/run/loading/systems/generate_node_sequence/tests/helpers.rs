use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::state::run::{
    definition::{TierDefinition, TierNodeCount},
    resources::DifficultyCurve,
};

pub(super) fn make_tier(
    nodes: TierNodeCount,
    active_ratio: f32,
    timer_mult: f32,
) -> TierDefinition {
    TierDefinition {
        nodes,
        active_ratio,
        timer_mult,
        introduced_cells: vec![],
    }
}

pub(super) fn make_curve(
    tiers: Vec<TierDefinition>,
    timer_reduction_per_boss: f32,
) -> DifficultyCurve {
    DifficultyCurve {
        tiers,
        timer_reduction_per_boss,
    }
}

pub(super) fn rng_from_seed(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}
