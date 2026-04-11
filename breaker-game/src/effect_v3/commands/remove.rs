//! Remove effect command — removes matching effect trees from an entity.

use bevy::prelude::*;

/// Deferred command that removes all effect trees matching a given name
/// from an entity's `BoundEffects` and `StagedEffects`.
pub struct RemoveEffectCommand {
    /// The entity to remove effect trees from.
    pub entity: Entity,
    /// The name to match against.
    pub name: String,
}

impl Command for RemoveEffectCommand {
    fn apply(self, _world: &mut World) {
        todo!()
    }
}
