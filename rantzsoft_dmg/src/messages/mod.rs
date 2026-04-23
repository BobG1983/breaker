//! Damage-pipeline messages.
//!
//! Each generic message (`DamageDealt<T>`, `HealDealt<T>`, `KillYourself<T>`,
//! `Destroyed<T>`) carries a `PhantomData<T: Dmgable>` marker that selects one
//! `Messages<M<T>>` resource per `T`, isolating the damage pipeline across
//! entity kinds. `DespawnEntity` is the non-generic command for entity
//! removal — it runs once per tick for all types.

mod damage_dealt;
mod despawn_entity;
mod destroyed;
mod heal_dealt;
mod kill_yourself;

pub use damage_dealt::DamageDealt;
pub use despawn_entity::DespawnEntity;
pub use destroyed::Destroyed;
pub use heal_dealt::HealDealt;
pub use kill_yourself::KillYourself;

#[cfg(test)]
mod per_t_isolation;
