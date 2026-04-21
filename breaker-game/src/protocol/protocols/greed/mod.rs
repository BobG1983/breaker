//! Greed protocol — skipping chip offers boosts rare-rarity chip weight next visit.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/greed.md`.
//!
//! Owns the `GreedConfig` resource (per-run tuning), `GreedStacks` resource
//! (per-run counter, cleared by `reset_run_state`), the `greed_on_skip` system
//! that reads `ChipOfferSkipped`, and the `apply_greed_boost` helper invoked
//! by the chip-select domain's `generate_chip_offerings`.

pub mod system;

#[cfg(test)]
mod tests;

pub use system::{GreedConfig, GreedStacks};
pub(crate) use system::{activate, apply_greed_boost, register};
