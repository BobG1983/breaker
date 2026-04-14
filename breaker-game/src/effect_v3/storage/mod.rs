//! Effect storage — `BoundEffects`, `StagedEffects`, `SpawnStampRegistry`,
//! `ArmedFiredParticipants`.

mod armed_fired_participants;
mod bound_effects;
mod spawn_stamp_registry;
mod staged_effects;

pub use armed_fired_participants::ArmedFiredParticipants;
pub use bound_effects::BoundEffects;
pub use spawn_stamp_registry::SpawnStampRegistry;
pub(crate) use spawn_stamp_registry::{
    stamp_spawned_bolts, stamp_spawned_breakers, stamp_spawned_cells, stamp_spawned_walls,
};
pub use staged_effects::StagedEffects;
