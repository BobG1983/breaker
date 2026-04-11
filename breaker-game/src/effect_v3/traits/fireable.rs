//! Fireable trait — the fire contract for all effects.

use bevy::prelude::*;

/// The contract for executing an effect on an entity.
///
/// Every config struct in `EffectType` implements this trait.
/// The enum is the dispatch layer; the config is the implementation.
pub trait Fireable {
    /// Execute this effect on the given entity.
    ///
    /// - `entity`: The entity to apply the effect to (the Owner, or a
    ///   participant if redirected via On).
    /// - `source`: The chip or definition name that originated this effect.
    /// - `world`: Exclusive world access for reading/writing components,
    ///   spawning entities, and sending messages.
    fn fire(&self, entity: Entity, source: &str, world: &mut World);

    /// Register runtime systems, components, or resources for this effect.
    ///
    /// Called by `EffectV3Plugin::build` for every config struct. The default
    /// implementation is a no-op. Override when the effect has tick systems,
    /// cleanup systems, or reset systems.
    fn register(_app: &mut App) {}
}
