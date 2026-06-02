//! System to generate 3 random hazard offerings before the selection screen.

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::seq::SliceRandom;

use crate::{
    mutators::hazards::{
        definition::{HazardDefinition, HazardKind},
        resources::{HazardOffers, HazardRegistry},
    },
    prelude::*,
    shared::rng::HazardRng,
};

/// Bundled parameters for hazard offering generation.
#[derive(SystemParam)]
pub(crate) struct HazardOfferingParams<'w, 's> {
    /// Command queue that writes the `HazardOffers` resource.
    commands: Commands<'w, 's>,
    /// Registry holding every hazard definition keyed by kind.
    registry: Res<'w, HazardRegistry>,
    /// Run-seeded RNG used for the shuffle.
    rng:      ResMut<'w, HazardRng>,
}

/// Generates hazard offerings by shuffling the registry and taking up to 3.
///
/// Runs `OnEnter(HazardSelectState::Selecting)` before `spawn_hazard_select`.
/// Writes `HazardOffers` via `Commands`; the chained `ApplyDeferred` barrier
/// flushes the insertion before the spawn system reads the resource.
pub(crate) fn generate_hazard_offerings(mut params: HazardOfferingParams) {
    // Iterate `HazardKind::ALL` for a deterministic input order — registry
    // storage is a `HashMap` and its iteration order is not seed-stable.
    let mut pool: Vec<HazardDefinition> = HazardKind::ALL
        .iter()
        .filter_map(|kind| params.registry.get(*kind).cloned())
        .collect();

    pool.shuffle(&mut params.rng.0);

    let offers: Vec<HazardDefinition> = pool.into_iter().take(3).collect();

    params.commands.insert_resource(HazardOffers(offers));
}

#[cfg(test)]
mod tests;
