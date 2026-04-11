//! Reverse effect command — deferred execution of `Reversible::reverse`.

use bevy::prelude::*;

use crate::effect_v3::{dispatch::reverse_dispatch, types::ReversibleEffectType};

/// Deferred command that reverses a reversible effect on an entity.
pub struct ReverseEffectCommand {
    /// The entity to reverse the effect on.
    pub entity: Entity,
    /// The reversible effect to reverse.
    pub effect: ReversibleEffectType,
    /// The chip or definition name that originated this effect.
    pub source: String,
}

impl Command for ReverseEffectCommand {
    fn apply(self, world: &mut World) {
        reverse_dispatch(&self.effect, self.entity, &self.source, world);
    }
}
