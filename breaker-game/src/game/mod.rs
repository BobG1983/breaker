//! Game plugin group — wires together all domain plugins.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::Game;
