//! Damage-pipeline components.

mod damage_boost_stack;
mod dead;
mod heal_cap;
mod hp;
mod invulnerable;
mod killed_by;
mod vulnerable_stack;

pub use damage_boost_stack::DamageBoostStack;
pub use dead::Dead;
pub use heal_cap::HealCap;
pub use hp::Hp;
pub use invulnerable::Invulnerable;
pub use killed_by::KilledBy;
pub use vulnerable_stack::VulnerableStack;
