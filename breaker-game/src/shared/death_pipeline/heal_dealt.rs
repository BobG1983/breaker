//! Generic heal message — one Bevy message queue per target type T.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::game_entity::GameEntity;

/// Per-message ceiling selector for `apply_heal<T>`.
///
/// Different senders want different clamps:
/// - Hazard neighbour-heals (Cascade, Sympathy) should cap at `Starting` so they
///   cannot push a cell past its pristine value.
/// - Chip / protocol buffs cap at `Max` to exploit an elevated `Hp.max` set
///   elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HealCap {
    /// Cap `Hp.current` at `Hp.starting` (ignores `Hp.max`).
    Starting,
    /// Cap `Hp.current` at `Hp.max.unwrap_or(Hp.starting)`.
    Max,
}

/// Generic heal message. One Bevy message queue per target type `T`.
///
/// Sent by: any domain that wants to heal a `GameEntity` (Cascade, Renewal,
/// other future restorative effects). Consumed by `apply_heal<T>` in the
/// unified death/heal pipeline.
#[derive(Message, Debug)]
pub(crate) struct HealDealt<T: GameEntity> {
    /// The entity that originated this heal (for future attribution / UI).
    pub healer:  Option<Entity>,
    /// The entity receiving the heal.
    pub target:  Entity,
    /// Pre-calculated heal amount. Values `<= 0.0` are ignored by `apply_heal<T>`.
    pub amount:  f32,
    /// Free-form label describing where the heal came from (chip name, effect
    /// name, etc.) — used by UI/stats.
    pub source:  Option<String>,
    /// Ceiling selector. Picked by the sender per-message.
    pub cap:     HealCap,
    /// Marker for the target entity type.
    pub _marker: PhantomData<T>,
}

// Manual Clone impl avoids requiring T: Clone (PhantomData is always Clone).
impl<T: GameEntity> Clone for HealDealt<T> {
    fn clone(&self) -> Self {
        Self {
            healer:  self.healer,
            target:  self.target,
            amount:  self.amount,
            source:  self.source.clone(),
            cap:     self.cap,
            _marker: PhantomData,
        }
    }
}
