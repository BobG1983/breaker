//! Generic damage message — one Bevy message queue per victim type T.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::game_entity::GameEntity;

/// Generic damage message. One Bevy message queue per victim type `T`.
///
/// Sent by: bolt collision, shockwave fire, chain lightning fire, explode fire,
/// piercing beam fire, tether beam tick, or any effect that deals damage.
#[derive(Message, Debug)]
pub(crate) struct DamageDealt<T: GameEntity> {
    /// The entity that originated this damage (for kill attribution).
    pub dealer:      Option<Entity>,
    /// The entity taking the damage.
    pub target:      Entity,
    /// Pre-calculated damage amount (includes any multipliers from the sender).
    pub amount:      f32,
    /// Which chip originated this damage chain, for UI/stats.
    pub source_chip: Option<String>,
    /// Marker for the victim entity type.
    pub _marker:     PhantomData<T>,
}

// Manual Clone impl avoids requiring T: Clone (PhantomData is always Clone).
impl<T: GameEntity> Clone for DamageDealt<T> {
    fn clone(&self) -> Self {
        Self {
            dealer:      self.dealer,
            target:      self.target,
            amount:      self.amount,
            source_chip: self.source_chip.clone(),
            _marker:     PhantomData,
        }
    }
}
