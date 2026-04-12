//! Route effect command — installs a tree on a target entity.

use bevy::prelude::*;

use crate::effect_v3::{
    storage::{BoundEffects, StagedEffects},
    types::{RouteType, Tree},
};

/// Deferred command that routes a tree to an entity.
pub struct RouteEffectCommand {
    /// The entity to install the tree on.
    pub entity:     Entity,
    /// The name identifying the source of the tree.
    pub name:       String,
    /// The tree to install.
    pub tree:       Tree,
    /// Whether the tree is permanent (Bound) or one-shot (Staged).
    pub route_type: RouteType,
}

impl Command for RouteEffectCommand {
    fn apply(self, world: &mut World) {
        match self.route_type {
            RouteType::Bound => {
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
            RouteType::Staged => {
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
    }
}
