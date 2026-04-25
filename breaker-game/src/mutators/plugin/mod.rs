//! Mutators plugin — protocols + hazards consolidated.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::MutatorsPlugin;
#[cfg(test)]
pub(crate) use system::wire_damage_chain;
