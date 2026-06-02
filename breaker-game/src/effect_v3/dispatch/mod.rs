//! Effect type dispatch — routes EffectType/ReversibleEffectType to config methods.

mod fire_dispatch;
mod reverse_dispatch;

pub use fire_dispatch::fire_dispatch;
pub(in crate::effect_v3) use fire_dispatch::fire_dispatch_with_rng;
pub use reverse_dispatch::{
    fire_reversible_dispatch, reverse_all_by_source_dispatch, reverse_dispatch,
};
