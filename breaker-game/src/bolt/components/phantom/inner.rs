//! Phantom-bolt component vocabulary and canonical switch functions.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    shared::{Lifespan, PhantomFlicker},
};

/// Marker identifying an entity as a phantom bolt.
///
/// Single source of truth; the old declaration at
/// `effect_v3/effects/phantom_bolt/components.rs` is a `pub use` re-export of
/// this type.
#[derive(Component, Debug, Clone, Copy)]
pub struct PhantomBolt;

/// Identifies the origin of a phantom bolt for dedup and attribution.
///
/// NOT `Copy` because the `Chip` variant owns a `String`.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhantomDedupKey {
    /// Afterimage phantom — spawned from a Perfect Bump on this real bolt.
    Bolt(Entity),
    /// Chip-effect phantom — spawned by `SpawnPhantomConfig::fire`.
    Chip {
        /// Name of the chip effect that created this phantom.
        chip:       String,
        /// Entity the chip effect fired from.
        fired_from: Entity,
    },
}

/// Per-phantom-lifetime set of cell entities already damaged by this phantom.
///
/// Inserted empty by `Bolt::become_phantom`. Mutated by `bolt_cell_collision`
/// in Wave 3B. Removed by `PhantomBolt::become_normal`.
#[derive(Component, Debug, Clone, Default)]
pub struct PhantomDamagedCells(pub HashSet<Entity>);

/// Decides what happens when `Lifespan` reaches zero.
///
/// Read by `tick_bolt_lifespan` in Wave 3A.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifetimeEndBehavior {
    /// Emit `DespawnEntity` to the death pipeline.
    Despawn,
    /// Strip the phantom component set; bolt continues as normal.
    RevertToNormalBolt,
}

/// Parameter bundle threaded through the bolt builder's `.phantom(...)` method.
///
/// NOT a `Component` — the dedup key flows out as the actual
/// `PhantomDedupKey` component via `Bolt::become_phantom`.
#[derive(Debug, Clone)]
pub struct PhantomParams {
    /// Identifier used to dedup phantom spawns per source.
    pub dedup_key: PhantomDedupKey,
}

impl Bolt {
    /// Stamps an entity as a phantom bolt.
    ///
    /// Inserts `PhantomBolt`, the supplied `PhantomDedupKey`, and an empty
    /// `PhantomDamagedCells` on `bolt`. No other components are touched.
    pub fn become_phantom(commands: &mut Commands, bolt: Entity, dedup_key: PhantomDedupKey) {
        commands
            .entity(bolt)
            .insert((PhantomBolt, dedup_key, PhantomDamagedCells::default()));
    }
}

impl PhantomBolt {
    /// Reverts a phantom bolt to a normal bolt.
    ///
    /// Removes `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`,
    /// `PhantomFlicker`, `Lifespan`, and `LifetimeEndBehavior` from `phantom`.
    /// No other components are touched. Safe to call on entities that lack
    /// some or all of the six components.
    pub fn become_normal(commands: &mut Commands, phantom: Entity) {
        commands.entity(phantom).remove::<(
            Self,
            PhantomDedupKey,
            PhantomDamagedCells,
            PhantomFlicker,
            Lifespan,
            LifetimeEndBehavior,
        )>();
    }
}
