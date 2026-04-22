//! Sympathy hazard — damage to a cell heals its neighbours.
//!
//! Design doc: `docs/design/hazards/sympathy.md`.
//!
//! Owns [`SympathyConfig`], the [`SYMPATHY_SENTINEL`] source-tag constant,
//! the [`activate`] entry point called from `hazards::activate`, and the
//! [`register`] entry point called from `hazards::register`. The runtime
//! system [`sympathy_heal_adjacent`] reads `DamageDealt<Cell>` and writes
//! `HealDealt<Cell>` with `HealCap::Starting` for every neighbour of each
//! damaged cell, attenuated per BFS ring.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
