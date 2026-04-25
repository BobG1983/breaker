//! Explode systems — request consumer that emits per-cell damage.
pub(crate) mod apply_explode;

#[cfg(test)]
mod tests;

pub(crate) use apply_explode::apply_explode_damage;
