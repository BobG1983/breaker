//! Explode effect — area damage at entity position.

pub mod config;
pub(crate) mod messages;
pub(crate) mod systems;

#[cfg(test)]
mod tests;

pub use config::ExplodeConfig;
