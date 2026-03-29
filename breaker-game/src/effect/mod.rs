//! Effect system — data-driven trigger→effect pipeline.

/// Commands extension for firing and reversing effects.
pub mod commands;
/// Core types: triggers, targets, effect nodes, effect kinds, components.
pub mod core;
/// Per-effect modules with `fire()`, `reverse()`, `register()`.
pub mod effects;
/// `EffectPlugin` — registers all effect and trigger systems.
pub mod plugin;
/// System sets for cross-domain ordering.
pub mod sets;
/// Trigger bridge systems and evaluation helpers.
pub mod triggers;

pub use core::*;

pub use commands::EffectCommandsExt;
pub use effects::{
    bump_force::EffectiveBumpForce, damage_boost::EffectiveDamageMultiplier,
    piercing::EffectivePiercing, quick_stop::EffectiveQuickStop,
    size_boost::EffectiveSizeMultiplier, speed_boost::EffectiveSpeedMultiplier,
};
pub use plugin::EffectPlugin;
pub use sets::EffectSystems;
