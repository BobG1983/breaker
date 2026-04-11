//! Unified death pipeline — generic damage, death detection, and despawn for all entity types.

pub mod damage_dealt;
pub mod dead;
pub mod despawn_entity;
pub mod destroyed;
pub mod game_entity;
pub mod hp;
pub mod kill_yourself;
pub mod killed_by;
pub mod plugin;
pub mod sets;
pub mod systems;

pub use damage_dealt::DamageDealt;
pub use dead::Dead;
pub use despawn_entity::DespawnEntity;
pub use destroyed::Destroyed;
pub use game_entity::GameEntity;
pub use hp::Hp;
pub use kill_yourself::KillYourself;
pub use killed_by::KilledBy;
pub use plugin::DeathPipelinePlugin;
pub use sets::DeathPipelineSystems;
