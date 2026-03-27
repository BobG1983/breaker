//! Chip registry — `HashMap` pool of all loaded chip definitions.

pub(crate) use data::*;

mod data;

#[cfg(test)]
mod tests;
