//! `NodeActive` condition evaluator.

use bevy::prelude::*;

use crate::prelude::*;

/// Evaluate whether the `NodeActive` condition is currently true.
///
/// Returns true while the current node is in the `Playing` state.
pub fn is_node_active(world: &World) -> bool {
    world
        .get_resource::<State<NodeState>>()
        .is_some_and(|state| *state.get() == NodeState::Playing)
}
