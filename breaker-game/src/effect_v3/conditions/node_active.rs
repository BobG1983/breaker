//! `NodeActive` condition evaluator.

use bevy::prelude::*;

use crate::state::types::NodeState;

/// Evaluate whether the `NodeActive` condition is currently true.
///
/// Returns true while a node is playing or paused.
pub fn is_node_active(world: &World) -> bool {
    world
        .get_resource::<State<NodeState>>()
        .is_some_and(|state| *state.get() == NodeState::Playing)
}
