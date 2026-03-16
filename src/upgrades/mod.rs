//! Upgrades domain plugin — Amps, Augments, and Overclocks system.

mod definition;
mod plugin;
mod resources;

pub use definition::{UpgradeDefinition, UpgradeKind};
pub use plugin::UpgradesPlugin;
pub use resources::UpgradeRegistry;
