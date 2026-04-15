//! Unified death pipeline — generic damage, death detection, and despawn for all entity types.

pub(crate) mod damage_dealt;
pub(crate) mod dead;
pub(crate) mod despawn_entity;
pub(crate) mod destroyed;
pub(crate) mod game_entity;
pub(crate) mod hp;
pub(crate) mod invulnerable;
pub(crate) mod kill_yourself;
pub(crate) mod killed_by;
pub(crate) mod plugin;
pub(crate) mod sets;
pub(crate) mod systems;

pub(crate) use damage_dealt::DamageDealt;
pub(crate) use dead::Dead;
pub(crate) use destroyed::Destroyed;
pub(crate) use game_entity::GameEntity;
pub(crate) use hp::Hp;
pub(crate) use invulnerable::Invulnerable;
pub(crate) use killed_by::KilledBy;
pub(crate) use plugin::DeathPipelinePlugin;
