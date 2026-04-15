//! Effect stack component — per-type stack storage.
pub(crate) mod component;

#[cfg(test)]
mod tests;

pub use component::EffectStack;
