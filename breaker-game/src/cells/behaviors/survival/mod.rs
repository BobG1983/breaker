//! Survival turret behavior — cells that periodically fire projectile
//! salvos at bolts. Bolt-immune, bump-vulnerable.

pub(crate) mod components;
pub(crate) mod systems;

#[cfg(test)]
mod tests;
