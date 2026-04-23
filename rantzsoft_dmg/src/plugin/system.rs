//! `RantzDmgPlugin` — the damage-pipeline plugin skeleton.
//!
//! The plugin registers the non-generic `DespawnEntity` message, configures
//! the 11-variant `DmgSystems` chain under `FixedUpdate`, and schedules a
//! no-op `process_despawn_requests` stub in `FixedPostUpdate`. Per-`T`
//! generic messages (`DamageDealt<T>`, `HealDealt<T>`, `KillYourself<T>`,
//! `Destroyed<T>`) and per-`T` generic systems are registered later by
//! `register_dmgable::<T>` in P6 via `RantzDmgAppExt` — NOT here.

use bevy::prelude::*;

use crate::{messages::DespawnEntity, sets::DmgSystems, systems::process_despawn_requests};

/// The damage-pipeline plugin. Consumers add this once to get the ordered
/// `DmgSystems` chain and the `DespawnEntity` message registered.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RantzDmgPlugin;

impl Plugin for RantzDmgPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DespawnEntity>();

        app.configure_sets(
            FixedUpdate,
            (
                DmgSystems::EmitDamage,
                DmgSystems::ApplyDamageBoosts,
                DmgSystems::MutateDamage,
                DmgSystems::ApplyVulnerable,
                DmgSystems::ApplyDamage,
                DmgSystems::EmitKill,
                DmgSystems::MutateKill,
                DmgSystems::ApplyKill,
                DmgSystems::EmitHeal,
                DmgSystems::MutateHeal,
                DmgSystems::ApplyHeal,
            )
                .chain(),
        );

        app.add_systems(FixedPostUpdate, process_despawn_requests);
    }
}
