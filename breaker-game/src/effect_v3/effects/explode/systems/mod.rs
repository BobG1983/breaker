//! Explode systems — request consumer that emits per-cell damage.
pub(in crate::effect_v3) mod apply_explode;

#[cfg(test)]
mod tests;

pub(in crate::effect_v3) use apply_explode::apply_explode_damage;
