//! Death request message — sent when an entity should die.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::game_entity::GameEntity;

/// Sent when an entity should die. Consumed by per-domain kill handlers that
/// perform domain-specific death logic before confirming the kill via
/// `Destroyed<T>`.
///
/// The entity must stay alive through domain handling, trigger evaluation,
/// and death animation.
#[derive(Message, Debug)]
pub(crate) struct KillYourself<T: GameEntity> {
    /// The entity to kill.
    pub victim:  Entity,
    /// The entity that caused the death (from `KilledBy`).
    pub killer:  Option<Entity>,
    /// Marker for the victim entity type.
    pub _marker: PhantomData<T>,
}

// Manual Clone impl avoids requiring T: Clone (PhantomData is always Clone).
impl<T: GameEntity> Clone for KillYourself<T> {
    fn clone(&self) -> Self {
        Self {
            victim:  self.victim,
            killer:  self.killer,
            _marker: PhantomData,
        }
    }
}
