//! Effect trait definitions.

mod fireable;
mod passive_effect;
mod reversible;

pub use fireable::Fireable;
pub use passive_effect::PassiveEffect;
pub use reversible::Reversible;
