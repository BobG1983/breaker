//! Stamp effect command — sugar for `route_effect` with `RouteType::Bound`.

use bevy::prelude::*;

use crate::effect_v3::types::Tree;

/// Deferred command that stamps (permanently installs) a tree on an entity.
/// Sugar for `RouteEffectCommand` with `RouteType::Bound`.
pub struct StampEffectCommand {
    /// The entity to install the tree on.
    pub entity: Entity,
    /// The name identifying the source of the tree.
    pub name: String,
    /// The tree to install.
    pub tree: Tree,
}

impl Command for StampEffectCommand {
    fn apply(self, _world: &mut World) {
        todo!()
    }
}
