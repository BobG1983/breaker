//! `PiercingBeam` systems — request consumer that emits per-cell damage.
pub(in crate::effect_v3) mod apply_piercing_beam;

#[cfg(test)]
mod tests;

pub(in crate::effect_v3) use apply_piercing_beam::apply_piercing_beam_damage;
