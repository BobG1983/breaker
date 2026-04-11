//! Route node evaluator — install a tree on another entity.

use bevy::prelude::*;

use crate::effect_v3::types::{RouteType, Tree, TriggerContext};

/// Evaluate a `Terminal::Route` node: install a tree on the target entity
/// with the given permanence.
///
/// NOTE: Route installation is deferred to a future wave. Route nodes
/// need the `RouteEffectCommand` to install the tree into the target's
/// `BoundEffects` or `StagedEffects`.
#[allow(
    clippy::missing_const_for_fn,
    reason = "deferred implementation will use &mut Commands"
)]
pub fn evaluate_route(
    _entity: Entity,
    _route_type: RouteType,
    _tree: &Tree,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    // TODO: implement Route tree installation.
    // Needs:
    // - Queue a RouteEffectCommand that installs the tree
    // - Bound: insert into BoundEffects
    // - Staged: insert into StagedEffects
}
