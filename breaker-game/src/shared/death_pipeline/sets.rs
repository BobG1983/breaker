//! System sets for the death pipeline.

use bevy::prelude::*;

/// System sets for death pipeline ordering.
///
/// `ApplyDamage` runs first (process damage messages, decrement Hp, set
/// `KilledBy`), then `DetectDeaths` (detect Hp <= 0, send `KillYourself`),
/// then `HandleKill` (consume `KillYourself<T>`, mark `Dead`, emit
/// `Destroyed<T>`, enqueue `DespawnEntity`), and finally `ApplyHeal`
/// (process heal messages, increment Hp bounded by the per-message
/// `HealCap` selector — runs last so `Dead` is already in place to gate
/// in-tick revival attempts).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum DeathPipelineSystems {
    /// Process damage messages, decrement Hp, set `KilledBy`.
    ApplyDamage,
    /// Detect Hp <= 0, send `KillYourself`.
    DetectDeaths,
    /// Consume `KillYourself<T>`, mark `Dead`, emit `Destroyed<T>`, enqueue
    /// `DespawnEntity`.
    HandleKill,
    /// Process heal messages, increment Hp clamped to `max`/`starting`. Runs
    /// AFTER `HandleKill` so that `Dead` (inserted by `handle_kill<T>`) is
    /// visible to `apply_heal<T>`'s `Without<Dead>` filter — this is what
    /// prevents same-tick heal-revives of a damage-killed entity.
    ApplyHeal,
}
