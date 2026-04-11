//! `EffectCommandsExt` — Commands extension trait for effect operations.

use bevy::prelude::*;

use crate::effect_v3::types::{EffectType, ReversibleEffectType, RouteType, Tree};

/// Extension trait for `Commands` providing effect operations.
///
/// All methods queue deferred commands that execute during the next
/// command flush.
pub trait EffectCommandsExt {
    /// Fire an effect on the given entity.
    fn fire_effect(&mut self, entity: Entity, effect: EffectType, source: String);

    /// Reverse a reversible effect on the given entity.
    fn reverse_effect(&mut self, entity: Entity, effect: ReversibleEffectType, source: String);

    /// Route a tree to an entity with the given permanence.
    fn route_effect(&mut self, entity: Entity, name: String, tree: Tree, route_type: RouteType);

    /// Sugar for `route_effect` with `RouteType::Bound`.
    fn stamp_effect(&mut self, entity: Entity, name: String, tree: Tree);

    /// Sugar for `route_effect` with `RouteType::Staged`.
    fn stage_effect(&mut self, entity: Entity, name: String, tree: Tree);

    /// Remove all effect trees matching the given name from an entity.
    fn remove_effect(&mut self, entity: Entity, name: &str);
}

impl EffectCommandsExt for Commands<'_, '_> {
    fn fire_effect(&mut self, _entity: Entity, _effect: EffectType, _source: String) {
        todo!()
    }

    fn reverse_effect(&mut self, _entity: Entity, _effect: ReversibleEffectType, _source: String) {
        todo!()
    }

    fn route_effect(
        &mut self,
        _entity: Entity,
        _name: String,
        _tree: Tree,
        _route_type: RouteType,
    ) {
        todo!()
    }

    fn stamp_effect(&mut self, _entity: Entity, _name: String, _tree: Tree) {
        todo!()
    }

    fn stage_effect(&mut self, _entity: Entity, _name: String, _tree: Tree) {
        todo!()
    }

    fn remove_effect(&mut self, _entity: Entity, _name: &str) {
        todo!()
    }
}
