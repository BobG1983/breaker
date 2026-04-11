//! Effect type dispatch — routes EffectType/ReversibleEffectType to config methods.

mod fire_dispatch;
mod reverse_dispatch;

pub use fire_dispatch::fire_dispatch;
pub use reverse_dispatch::reverse_dispatch;
