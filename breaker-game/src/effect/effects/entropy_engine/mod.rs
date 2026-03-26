//! Entropy engine evolution effect — counts cell destructions and fires
//! a random effect from the pool when the threshold is reached.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::*;
