//! `fire_dispatch` — match `EffectType` variant to `config.fire()` call.

use bevy::prelude::*;

use crate::effect_v3::types::EffectType;

/// Dispatch an `EffectType` to the appropriate config's `fire()` method.
///
/// This is a mechanical match — each arm unwraps the config and calls
/// `config.fire(entity, source, world)`.
pub fn fire_dispatch(_effect: &EffectType, _entity: Entity, _source: &str, _world: &mut World) {
    todo!()
}
