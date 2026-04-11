//! Death confirmed message — sent after the domain kill handler confirms the kill.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::game_entity::GameEntity;

/// Sent after the domain kill handler confirms the kill.
///
/// The entity is still alive when `Destroyed` is sent. It survives through
/// trigger evaluation and death animation. Despawn happens later via
/// `DespawnEntity` in `PostFixedUpdate`.
#[derive(Message, Clone, Debug)]
pub struct Destroyed<T: GameEntity> {
    /// The entity that died (still alive at this point).
    pub victim: Entity,
    /// The entity that caused the death (`None` for environmental deaths).
    pub killer: Option<Entity>,
    /// World position of the victim at time of death.
    pub victim_pos: Vec2,
    /// World position of the killer, if it exists. Used for directional VFX.
    pub killer_pos: Option<Vec2>,
    /// Marker for the victim entity type.
    pub _marker: PhantomData<T>,
}
