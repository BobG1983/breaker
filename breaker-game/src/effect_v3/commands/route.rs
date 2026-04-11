//! Route effect command — installs a tree on a target entity.

use bevy::prelude::*;

use crate::effect_v3::types::{RouteType, Tree};

/// Deferred command that routes a tree to an entity.
pub struct RouteEffectCommand {
    /// The entity to install the tree on.
    pub entity: Entity,
    /// The name identifying the source of the tree.
    pub name: String,
    /// The tree to install.
    pub tree: Tree,
    /// Whether the tree is permanent (Bound) or one-shot (Staged).
    pub route_type: RouteType,
}

impl Command for RouteEffectCommand {
    fn apply(self, _world: &mut World) {
        todo!()
    }
}
