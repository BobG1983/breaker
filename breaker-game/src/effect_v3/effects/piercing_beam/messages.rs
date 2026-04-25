//! `PiercingBeam` emitter request — written by `PiercingBeamConfig::fire`,
//! read by `apply_piercing_beam_damage` in `DmgSystems::EmitDamage`.
//!
//! Stub created during W7 RED phase. The struct shape and field names match
//! `.claude/specs/w7-fireable-damage-refactor-code.md` §2.a so the failing
//! tests compile. Production wiring is the GREEN phase's responsibility.

use bevy::prelude::*;

use crate::prelude::SourceId;

/// Internal request emitted by `PiercingBeamConfig::fire` and consumed by
/// `apply_piercing_beam_damage` one tick later.
#[derive(Message, Debug, Clone)]
pub(in crate::effect_v3) struct PiercingBeamEmissionRequested {
    /// World-space origin of the beam.
    pub(in crate::effect_v3) origin:      Vec2,
    /// Unit-length forward direction of the beam.
    pub(in crate::effect_v3) direction:   Vec2,
    /// Half-width of the beam rectangle (`width / 2.0`).
    pub(in crate::effect_v3) half_width:  f32,
    /// Raw, un-multiplied damage per cell hit.
    pub(in crate::effect_v3) base_damage: f32,
    /// The firing entity. Becomes `DamageDealt::dealer`.
    pub(in crate::effect_v3) dealer:      Option<Entity>,
    /// Optional opaque caller-supplied `SourceId`.
    pub(in crate::effect_v3) source:      Option<SourceId>,
}
