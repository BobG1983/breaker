//! System sets for the death pipeline.

use bevy::prelude::*;

/// System sets for death pipeline ordering.
///
/// `ApplyDamage` runs first (process damage messages, decrement Hp, set
/// `KilledBy`), then `DetectDeaths` (detect Hp <= 0, send `KillYourself`),
/// then `HandleKill` (consume `KillYourself<T>`, mark `Dead`, emit
/// `Destroyed<T>`, enqueue `DespawnEntity`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum DeathPipelineSystems {
    /// Process damage messages, decrement Hp, set `KilledBy`.
    ApplyDamage,
    /// Detect Hp <= 0, send `KillYourself`.
    DetectDeaths,
    /// Consume `KillYourself<T>`, mark `Dead`, emit `Destroyed<T>`, enqueue
    /// `DespawnEntity`.
    HandleKill,
}
