//! Route node evaluator — install a tree on another entity.

use bevy::prelude::*;

use crate::effect_v3::types::{RouteType, Tree, TriggerContext};

/// Evaluate a `Terminal::Route` node: install a tree on the target entity
/// with the given permanence.
pub fn evaluate_route(
    _entity: Entity,
    _route_type: RouteType,
    _tree: &Tree,
    _context: &TriggerContext,
    _source: &str,
    _commands: &mut Commands,
) {
    todo!()
}
