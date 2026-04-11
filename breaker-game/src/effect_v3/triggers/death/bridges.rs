//! Death trigger bridge systems.
//!
//! Each bridge reads `Destroyed<T>` messages and dispatches `Died`, `Killed`, and
//! `DeathOccurred` triggers to entities with bound effects.

use bevy::prelude::*;

use crate::effect_v3::types::{Trigger, TriggerContext};

/// Bridge: reads `Destroyed<T>` messages and dispatches death triggers.
///
/// For each destroyed entity:
/// - `Died` on the victim entity (local)
/// - `Killed(EntityKind)` on the killer entity (local, if killer exists)
/// - `DeathOccurred(EntityKind)` on all entities (global)
///
/// Generic over the entity marker type — monomorphized for Cell, Bolt, Wall, Breaker.
pub fn on_destroyed() {
    todo!()
}
