//! Stage effect command — sugar for `route_effect` with `RouteType::Staged`.

use bevy::prelude::*;

use crate::effect_v3::{storage::StagedEffects, types::Tree};

/// Deferred command that stages (one-shot installs) a tree on an entity.
/// Sugar for `RouteEffectCommand` with `RouteType::Staged`.
pub struct StageEffectCommand {
    /// The entity to install the tree on.
    pub entity: Entity,
    /// The name identifying the source of the tree.
    pub name: String,
    /// The tree to install.
    pub tree: Tree,
}

impl Command for StageEffectCommand {
    fn apply(self, world: &mut World) {
        let has_staged = world.get::<StagedEffects>(self.entity).is_some();
        if !has_staged {
            world
                .entity_mut(self.entity)
                .insert(StagedEffects::default());
        }
        if let Some(mut staged) = world.get_mut::<StagedEffects>(self.entity) {
            staged.0.push((self.name, self.tree));
        }
    }
}
