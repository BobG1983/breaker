//! New effect domain (v3) — data-driven trigger->effect pipeline.
//!
//! Built alongside old `effect/` domain. Both coexist until migration is complete.

pub mod commands;
pub mod components;
pub mod conditions;
pub mod dispatch;
pub mod effects;
mod plugin;
pub mod sets;
pub mod stacking;
pub mod storage;
pub mod traits;
pub mod triggers;
pub mod types;
pub mod walking;

pub use plugin::EffectV3Plugin;
pub use sets::EffectV3Systems;
