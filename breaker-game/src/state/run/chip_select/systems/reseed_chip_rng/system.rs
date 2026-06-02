//! System to reseed `ChipRng` from the canonical formula on chip-select entry.

use bevy::prelude::*;

use crate::{
    prelude::*,
    shared::rng::{ChipRng, ChipSelectCount, derive_seed, derive_seed_named},
    state::run::resources::RunStats,
};

/// Seeds `ChipRng` from `derive_seed(derive_seed_named(run_stats.seed, "chip"), count.0 as u64)`
/// on `OnEnter(ChipSelectState::Selecting)`, before `generate_chip_offerings`.
pub(crate) fn reseed_chip_rng(
    run_stats: Res<RunStats>,
    count: Res<ChipSelectCount>,
    mut commands: Commands,
) {
    let seed = derive_seed(
        derive_seed_named(run_stats.seed, "chip"),
        u64::from(count.0),
    );
    commands.insert_resource(ChipRng::from_seed(seed));
}
