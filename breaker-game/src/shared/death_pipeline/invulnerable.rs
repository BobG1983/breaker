//! Generic invulnerability marker — filters an entity out of `apply_damage<T>`.

use bevy::prelude::*;

/// Marker component — any entity with this component is skipped by
/// `apply_damage<T>`'s victim query via a `Without<Invulnerable>` filter.
///
/// Generic across all `GameEntity` types. Producers (e.g., the cells domain's
/// `Locked ↔ Invulnerable` coupling) insert/remove it freely; the damage
/// pipeline only reads the marker's presence.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct Invulnerable;
