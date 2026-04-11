//! Effect storage — `BoundEffects`, `StagedEffects`, `SpawnStampRegistry`.

mod bound_effects;
mod spawn_stamp_registry;
mod staged_effects;

pub use bound_effects::BoundEffects;
pub use spawn_stamp_registry::SpawnStampRegistry;
pub use staged_effects::StagedEffects;
