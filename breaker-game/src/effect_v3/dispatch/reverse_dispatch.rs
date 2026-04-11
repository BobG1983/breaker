//! `reverse_dispatch` — match `ReversibleEffectType` variant to `config.reverse()` call.

use bevy::prelude::*;

use crate::effect_v3::types::ReversibleEffectType;

/// Dispatch a `ReversibleEffectType` to the appropriate config's `reverse()` method.
///
/// This is a mechanical match — each arm unwraps the config and calls
/// `config.reverse(entity, source, world)`.
pub fn reverse_dispatch(
    _effect: &ReversibleEffectType,
    _entity: Entity,
    _source: &str,
    _world: &mut World,
) {
    todo!()
}
