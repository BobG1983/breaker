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
pub(crate) struct PiercingBeamEmissionRequested {
    /// World-space origin of the beam.
    pub(crate) origin:      Vec2,
    /// Unit-length forward direction of the beam.
    pub(crate) direction:   Vec2,
    /// Half-width of the beam rectangle (`width / 2.0`).
    pub(crate) half_width:  f32,
    /// Raw, un-multiplied damage per cell hit.
    pub(crate) base_damage: f32,
    /// The firing entity. Becomes `DamageDealt::dealer`.
    pub(crate) dealer:      Option<Entity>,
    /// Optional opaque caller-supplied `SourceId`.
    pub(crate) source:      Option<SourceId>,
}
