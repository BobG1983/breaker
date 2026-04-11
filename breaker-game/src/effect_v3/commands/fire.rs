//! Fire effect command — deferred execution of `Fireable::fire`.

use bevy::prelude::*;

use crate::effect_v3::types::EffectType;

/// Deferred command that fires an effect on an entity.
pub struct FireEffectCommand {
    /// The entity to apply the effect to.
    pub entity: Entity,
    /// The effect to fire.
    pub effect: EffectType,
    /// The chip or definition name that originated this effect.
    pub source: String,
}

impl Command for FireEffectCommand {
    fn apply(self, _world: &mut World) {
        todo!()
    }
}
