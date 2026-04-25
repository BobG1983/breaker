//! Explode emitter request — written by `ExplodeConfig::fire`, read by
//! `apply_explode_damage` in `DmgSystems::EmitDamage`.
//!
//! Stub created during W7 RED phase. The struct shape and field names match
//! `.claude/specs/w7-fireable-damage-refactor-code.md` §1.a so the failing
//! tests compile. Production wiring (writer to `fire`, consumer system,
//! `app.add_message::<...>()` registration) is the GREEN phase's
//! responsibility.

use bevy::prelude::*;

use crate::prelude::SourceId;

/// Internal request emitted by `ExplodeConfig::fire` and consumed by
/// `apply_explode_damage` one tick later. Carries everything the consumer
/// needs to perform the spatial query and emit `DamageDealt<Cell>` per hit.
#[derive(Message, Debug, Clone)]
pub(in crate::effect_v3) struct ExplodeEmissionRequested {
    /// World-space center of the explosion.
    pub(in crate::effect_v3) center:      Vec2,
    /// Radius of the explosion in world units.
    pub(in crate::effect_v3) radius:      f32,
    /// Raw, un-multiplied damage per cell hit.
    pub(in crate::effect_v3) base_damage: f32,
    /// The firing entity (chip/effect owner).
    pub(in crate::effect_v3) dealer:      Option<Entity>,
    /// Optional builder-produced `SourceId`.
    pub(in crate::effect_v3) source:      Option<SourceId>,
}
