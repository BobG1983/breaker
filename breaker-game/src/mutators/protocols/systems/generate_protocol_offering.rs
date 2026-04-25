//! System to generate a protocol offering on entry to chip-select.
//!
//! Picks a uniformly-random protocol from the set of unlocked, not-yet-active
//! protocols that exist in the registry, and inserts it as
//! `ProtocolOffer(Some(def))`. If the eligible pool is empty, inserts
//! `ProtocolOffer(None)`.

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::Rng;

use crate::{
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind},
        resources::{ActiveProtocols, ProtocolOffer, ProtocolRegistry, UnlockedProtocols},
    },
    prelude::*,
};

/// Bundled parameters for [`generate_protocol_offering`].
///
/// Mirrors `ChipOfferingParams` in `generate_chip_offerings.rs`. The
/// `commands` path (rather than `ResMut<ProtocolOffer>`) is required because
/// the downstream `spawn_chip_select` reads the offer via `Res<ProtocolOffer>`
/// and the `ApplyDeferred` barrier in the `OnEnter` chain flushes the insert.
#[derive(SystemParam)]
pub(crate) struct ProtocolOfferingParams<'w, 's> {
    /// Command queue that writes the `ProtocolOffer` resource.
    commands: Commands<'w, 's>,
    /// Registry holding every protocol definition keyed by kind.
    registry: Res<'w, ProtocolRegistry>,
    /// Protocols already taken earlier in the run.
    active:   Res<'w, ActiveProtocols>,
    /// Kinds the player has unlocked for offering.
    unlocked: Res<'w, UnlockedProtocols>,
    /// Run-seeded RNG used for the uniform random pick.
    rng:      ResMut<'w, GameRng>,
}

/// Generate a protocol offering at chip-select entry.
///
/// Writes `ProtocolOffer` via `Commands` so the downstream
/// `spawn_chip_select` system can read the flushed value through the
/// `ApplyDeferred` barrier the plugin inserts between these systems.
pub(crate) fn generate_protocol_offering(mut params: ProtocolOfferingParams) {
    // Iterate the canonical ALL slice for deterministic order; keep kinds that
    // are unlocked, not yet active, and present in the registry.
    let mut pool: Vec<ProtocolDefinition> = ProtocolKind::ALL
        .iter()
        .filter_map(|kind| {
            if params.unlocked.contains(*kind) && !params.active.contains(*kind) {
                params.registry.get(*kind).cloned()
            } else {
                None
            }
        })
        .collect();

    if pool.is_empty() {
        params.commands.insert_resource(ProtocolOffer(None));
        return;
    }

    let idx = params.rng.0.random_range(0..pool.len());
    let chosen = pool.swap_remove(idx);
    params.commands.insert_resource(ProtocolOffer(Some(chosen)));
}

#[cfg(test)]
mod tests;
