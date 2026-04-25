//! `PiercingBeam` systems — request consumer that emits per-cell damage.
pub(crate) mod apply_piercing_beam;

#[cfg(test)]
mod tests;

pub(crate) use apply_piercing_beam::apply_piercing_beam_damage;
