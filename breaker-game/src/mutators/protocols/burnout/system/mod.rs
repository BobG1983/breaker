//! Burnout protocol — heat-gauge rhythm mechanic.
//!
//! Design doc: `docs/design/protocols/burnout.md`.

pub(crate) mod amplify;
pub(crate) mod cleanup_node;
pub(crate) mod config;
pub(crate) mod on_bump;
pub(crate) mod register;
pub(crate) mod tick_speed_boost;
pub(crate) mod update_heat;

pub(crate) use config::activate;
pub use on_bump::BurnoutDamageBoost;
pub(crate) use register::register;
pub use update_heat::BurnoutHeat;
