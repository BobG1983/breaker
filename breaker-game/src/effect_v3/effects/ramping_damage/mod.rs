//! Ramping damage passive — increases damage per consecutive hit.

pub mod components;
pub mod config;
pub mod systems;

pub use components::RampingDamageAccumulator;
pub use config::RampingDamageConfig;
