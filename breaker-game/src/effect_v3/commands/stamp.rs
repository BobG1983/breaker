//! Stamp effect command — sugar for `route_effect` with `RouteType::Bound`.

use bevy::prelude::*;

use crate::effect_v3::{storage::BoundEffects, types::Tree};

/// Deferred command that stamps (permanently installs) a tree on an entity.
/// Sugar for `RouteEffectCommand` with `RouteType::Bound`.
pub struct StampEffectCommand {
    /// The entity to install the tree on.
    pub entity: Entity,
    /// The name identifying the source of the tree.
    pub name:   String,
    /// The tree to install.
    pub tree:   Tree,
}

impl Command for StampEffectCommand {
    fn apply(self, world: &mut World) {
        let has_bound = world.get::<BoundEffects>(self.entity).is_some();
        if !has_bound {
            world
                .entity_mut(self.entity)
                .insert(BoundEffects::default());
        }
        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
            bound.0.push((self.name, self.tree));
        }
    }
}
