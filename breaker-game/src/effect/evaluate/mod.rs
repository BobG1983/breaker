//! Pure evaluation function — maps a runtime `Trigger` + `EffectNode` to matched children.

mod core;

pub(crate) use core::*;

#[cfg(test)]
mod tests;
