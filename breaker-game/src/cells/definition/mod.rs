//! Cell type definition — RON-deserialized data for a single cell type.

pub(crate) mod data;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use data::GuardedBehavior;
pub(crate) use data::{CellBehavior, CellTypeDefinition};
