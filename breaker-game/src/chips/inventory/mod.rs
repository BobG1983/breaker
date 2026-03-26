//! Chip inventory — tracks the player's chip build during a run.

mod data;

pub use data::{ChipEntry, ChipInventory};

#[cfg(test)]
mod tests;
