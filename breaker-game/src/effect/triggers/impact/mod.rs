//! Bridge systems for impact triggers — sweeps ALL entities with `EffectChains`
//! for `Trigger::Impact(Cell/Wall/Breaker)`, evaluates `ArmedEffects` on bolt,
//! and evaluates `Until` children.

mod bridge;

#[cfg(test)]
mod tests;

pub(crate) use bridge::*;
