//! Chip registry — `HashMap` pool of all loaded chip definitions.

pub use data::ChipCatalog;
pub(crate) use data::*;

mod data;

#[cfg(test)]
mod tests;
