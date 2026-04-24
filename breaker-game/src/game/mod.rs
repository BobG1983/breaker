//! Game plugin group — wires together all domain plugins.

pub(crate) mod sets;
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use sets::PostApplyRipple;
pub use system::Game;
