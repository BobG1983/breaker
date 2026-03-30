//! Scenario definition types loaded from RON files.
//!
//! Types here are pure data -- no Bevy components or resources. They are
//! deserialized from `.scenario.ron` files and consumed by the runner.

mod definitions;

pub use definitions::*;

#[cfg(test)]
mod tests;
