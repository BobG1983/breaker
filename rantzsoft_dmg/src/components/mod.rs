//! Damage-pipeline components.

mod dead;
mod heal_cap;
mod hp;
mod invulnerable;
mod killed_by;

pub use dead::Dead;
pub use heal_cap::HealCap;
pub use hp::Hp;
pub use invulnerable::Invulnerable;
pub use killed_by::KilledBy;
