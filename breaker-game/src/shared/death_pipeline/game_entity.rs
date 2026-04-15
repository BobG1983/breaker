//! `GameEntity` marker trait for entity types that participate in the death pipeline.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt, breaker::components::Breaker, cells::components::Cell,
    walls::components::Wall,
};

/// Marker trait for entity types that participate in the death pipeline.
///
/// Each `impl` creates a separate Bevy message queue — `DamageDealt<Cell>` and
/// `DamageDealt<Bolt>` are independent message types.
pub(crate) trait GameEntity: Component {}

impl GameEntity for Cell {}
impl GameEntity for Bolt {}
impl GameEntity for Wall {}
impl GameEntity for Breaker {}
