//! Remove effect command — removes matching effect trees from an entity.

use bevy::prelude::*;

use crate::effect_v3::storage::{BoundEffects, StagedEffects};

/// Deferred command that removes all effect trees matching a given name
/// from an entity's `BoundEffects` and `StagedEffects`.
pub struct RemoveEffectCommand {
    /// The entity to remove effect trees from.
    pub entity: Entity,
    /// The name to match against.
    pub name:   String,
}

impl Command for RemoveEffectCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
            bound.0.retain(|(name, _)| name != &self.name);
        }
        if let Some(mut staged) = world.get_mut::<StagedEffects>(self.entity) {
            staged.0.retain(|(name, _)| name != &self.name);
        }
    }
}
