//! Reversible trait — the reverse contract for reversible effects.

use bevy::prelude::*;

use super::Fireable;

/// The contract for undoing an effect on an entity.
///
/// Config structs in `ReversibleEffectType` implement both `Fireable`
/// and `Reversible`. Reversible extends Fireable — every reversible
/// config can also be fired.
pub trait Reversible: Fireable {
    /// Reverse (undo) this effect on the given entity.
    ///
    /// - `entity`: The entity to reverse the effect on.
    /// - `source`: Must match the source used in the original fire call.
    /// - `world`: Exclusive world access.
    fn reverse(&self, entity: Entity, source: &str, world: &mut World);

    /// Reverse ALL instances of this effect that were fired from the given source.
    ///
    /// For singleton/owner-indexed effects (`FlashStep`, `Pulse`, `Shield`, etc.)
    /// this defaults to a single `reverse()` call — there is at most one active
    /// instance. Stack-based passives (`SpeedBoost`, `Piercing`, etc.) override to
    /// remove
    /// all matching entries via `EffectStack::retain_by_source`.
    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        self.reverse(entity, source, world);
    }
}
