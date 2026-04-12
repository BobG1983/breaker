//! `GameEntity` marker trait for entity types that participate in the death pipeline.

use bevy::prelude::*;

/// Marker trait for entity types that participate in the death pipeline.
///
/// Each `impl` creates a separate Bevy message queue — `DamageDealt<Cell>` and
/// `DamageDealt<Bolt>` are independent message types.
pub trait GameEntity: Component {}

impl GameEntity for crate::cells::components::Cell {}
impl GameEntity for crate::bolt::components::Bolt {}
impl GameEntity for crate::walls::components::Wall {}
impl GameEntity for crate::breaker::components::Breaker {}
