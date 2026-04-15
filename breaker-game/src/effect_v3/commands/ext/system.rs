//! `EffectCommandsExt` — Commands extension trait for effect operations.

use bevy::prelude::*;

use super::super::{
    FireEffectCommand, RemoveEffectCommand, RemoveStagedEffectCommand, ReverseEffectCommand,
    RouteEffectCommand, StageEffectCommand, StampEffectCommand, TrackArmedFireCommand,
};
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

    /// Remove the FIRST `StagedEffects` entry on `entity` whose source
    /// name equals `name` AND whose tree equals `tree`. Removes exactly
    /// one entry — not a name sweep. Does NOT touch `BoundEffects`.
    ///
    /// Used by `walk_staged_effects` to consume a staged entry that just
    /// matched its trigger by entry-specific `(name, Tree)` tuple
    /// identity, preserving any fresh same-name stages queued later in
    /// the same command flush.
    fn remove_staged_effect(&mut self, entity: Entity, name: String, tree: Tree);

    /// Record `participant` as having received an armed On's fired effect
    /// under `armed_source` on the `owner`'s `ArmedFiredParticipants`
    /// component. Used by `evaluate_on` when firing through an armed
    /// Shape-D scoped tree so the disarm path can reverse effects on the
    /// exact participants they were fired on.
    fn track_armed_fire(&mut self, owner: Entity, armed_source: String, participant: Entity);
}

impl EffectCommandsExt for Commands<'_, '_> {
    fn fire_effect(&mut self, entity: Entity, effect: EffectType, source: String) {
        self.queue(FireEffectCommand {
            entity,
            effect,
            source,
        });
    }

    fn reverse_effect(&mut self, entity: Entity, effect: ReversibleEffectType, source: String) {
        self.queue(ReverseEffectCommand {
            entity,
            effect,
            source,
        });
    }

    fn route_effect(&mut self, entity: Entity, name: String, tree: Tree, route_type: RouteType) {
        self.queue(RouteEffectCommand {
            entity,
            name,
            tree,
            route_type,
        });
    }

    fn stamp_effect(&mut self, entity: Entity, name: String, tree: Tree) {
        self.queue(StampEffectCommand { entity, name, tree });
    }

    fn stage_effect(&mut self, entity: Entity, name: String, tree: Tree) {
        self.queue(StageEffectCommand { entity, name, tree });
    }

    fn remove_effect(&mut self, entity: Entity, name: &str) {
        self.queue(RemoveEffectCommand {
            entity,
            name: name.to_owned(),
        });
    }

    fn remove_staged_effect(&mut self, entity: Entity, name: String, tree: Tree) {
        self.queue(RemoveStagedEffectCommand { entity, name, tree });
    }

    fn track_armed_fire(&mut self, owner: Entity, armed_source: String, participant: Entity) {
        self.queue(TrackArmedFireCommand {
            owner,
            armed_source,
            participant,
        });
    }
}
